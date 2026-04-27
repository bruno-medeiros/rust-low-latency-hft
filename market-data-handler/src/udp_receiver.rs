//! UDP receiver using `recvmmsg(2)` for batched packet ingestion.
//!
//! Pre-allocates a fixed array of receive buffers and a matching `mmsghdr` array so
//! the hot-path `recv_batch` call touches no heap. Each call to `recv_batch` issues a
//! single `recvmmsg` syscall and returns slices into the pre-allocated buffers.

use std::net::UdpSocket;
use std::os::fd::AsRawFd;

/// Number of datagrams to batch per `recvmmsg` call.
pub const BATCH_SIZE: usize = 64;

/// Per-datagram maximum payload (Ethernet MTU minus IP+UDP headers).
pub const BUF_SIZE: usize = 1472;

/// A filled receive buffer from one `recvmmsg` batch entry.
pub struct RecvBuf {
    pub data: [u8; BUF_SIZE],
    pub len: usize,
}

impl Default for RecvBuf {
    fn default() -> Self {
        Self { data: [0u8; BUF_SIZE], len: 0 }
    }
}

impl RecvBuf {
    pub fn as_slice(&self) -> &[u8] {
        &self.data[..self.len]
    }
}

/// Batched UDP receiver. Owns the socket and pre-allocated receive buffers.
///
/// `bufs` is heap-allocated as a `Vec` to avoid a ~95 KB stack temporary
/// (`BATCH_SIZE × BUF_SIZE`) that `Box::new([...; BATCH_SIZE])` would otherwise
/// construct on the stack before moving to the heap.
pub struct UdpReceiver {
    socket: UdpSocket,
    bufs: Vec<RecvBuf>,
}

impl UdpReceiver {
    pub fn new(socket: UdpSocket) -> Self {
        let bufs = (0..BATCH_SIZE).map(|_| RecvBuf::default()).collect();
        Self { socket, bufs }
    }

    /// Block until at least one datagram arrives. Returns filled entries from the
    /// pre-allocated buffer slice. No allocation on the hot path.
    pub fn recv_batch(&mut self) -> std::io::Result<&[RecvBuf]> {
        let fd = self.socket.as_raw_fd();
        let n = recv_mmsg_batch(fd, &mut self.bufs)?;
        Ok(&self.bufs[..n])
    }

    pub fn socket(&self) -> &UdpSocket {
        &self.socket
    }
}

/// Issue `recvmmsg(2)` against `fd`, filling `bufs[..n]` and returning `n`.
///
/// The `iovec` and `mmsghdr` arrays (~4.5 KB combined) are stack-allocated here;
/// they are small enough to be safe on the stack unlike the receive data buffers.
fn recv_mmsg_batch(fd: i32, bufs: &mut [RecvBuf]) -> std::io::Result<usize> {
    debug_assert_eq!(bufs.len(), BATCH_SIZE);
    // SAFETY: iovec and mmsghdr are POD structs; zero-init produces valid zeroed state.
    let mut iovecs: [libc::iovec; BATCH_SIZE] = unsafe { std::mem::zeroed() };
    let mut msgs: [libc::mmsghdr; BATCH_SIZE] = unsafe { std::mem::zeroed() };

    for i in 0..BATCH_SIZE {
        iovecs[i].iov_base = bufs[i].data.as_mut_ptr() as *mut libc::c_void;
        iovecs[i].iov_len = BUF_SIZE;
        msgs[i].msg_hdr.msg_iov = &mut iovecs[i];
        msgs[i].msg_hdr.msg_iovlen = 1;
    }

    // SAFETY: fd is valid, iovecs/msgs are correctly initialised above.
    let ret = unsafe {
        libc::recvmmsg(
            fd,
            msgs.as_mut_ptr(),
            BATCH_SIZE as libc::c_uint,
            0,
            std::ptr::null_mut(),
        )
    };

    if ret < 0 {
        return Err(std::io::Error::last_os_error());
    }

    let n = ret as usize;
    for i in 0..n {
        bufs[i].len = msgs[i].msg_len as usize;
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::UdpSocket;
    use std::time::Duration;

    fn rx_with_timeout() -> UdpSocket {
        let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
        // Short timeout so recv_batch unblocks after test packets are consumed.
        sock.set_read_timeout(Some(Duration::from_millis(200))).unwrap();
        sock
    }

    #[test]
    fn recv_single_datagram() {
        let rx_sock = rx_with_timeout();
        let addr = rx_sock.local_addr().unwrap();
        let tx_sock = UdpSocket::bind("127.0.0.1:0").unwrap();

        let payload = b"hello-recvmmsg";
        tx_sock.send_to(payload, addr).unwrap();

        let mut receiver = UdpReceiver::new(rx_sock);
        let batch = receiver.recv_batch().unwrap();
        assert_eq!(batch.len(), 1);
        assert_eq!(batch[0].as_slice(), payload);
    }

    #[test]
    fn recv_multiple_datagrams_in_one_batch() {
        let rx_sock = rx_with_timeout();
        let addr = rx_sock.local_addr().unwrap();
        let tx_sock = UdpSocket::bind("127.0.0.1:0").unwrap();

        for i in 0u8..4 {
            tx_sock.send_to(&[i; 8], addr).unwrap();
        }

        // Give the kernel a moment to buffer all four packets.
        std::thread::sleep(std::time::Duration::from_millis(5));

        let mut receiver = UdpReceiver::new(rx_sock);
        let batch = receiver.recv_batch().unwrap();
        assert_eq!(batch.len(), 4);
        for (i, buf) in batch.iter().enumerate() {
            assert_eq!(buf.as_slice(), &[i as u8; 8]);
        }
    }
}
