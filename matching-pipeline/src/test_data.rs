//! LOBSTER fixtures checked into the crate (integration tests, benches).

use std::sync::OnceLock;

use crate::{LobsterParser, OrderCommand};

/// Relative path from the crate root to the GOOG level-1 message CSV.
pub const GOOG_SAMPLE_MESSAGE_REL_PATH: &str =
    "LOBSTER_SampleFiles/GOOG_2012-06-21_34200000_57600000_message_1.csv";

/// Absolute path to [`GOOG_SAMPLE_MESSAGE_REL_PATH`].
pub fn goog_sample_message_path() -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), GOOG_SAMPLE_MESSAGE_REL_PATH)
}

/// Raw contents of the GOOG sample message file.
pub fn load_goog_sample_csv() -> String {
    let path = goog_sample_message_path();
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"))
}

/// Parsed order commands from the GOOG sample; computed once on first use.
pub fn goog_sample_commands() -> &'static [OrderCommand] {
    static COMMANDS: OnceLock<Vec<OrderCommand>> = OnceLock::new();
    COMMANDS
        .get_or_init(|| {
            let csv = load_goog_sample_csv();
            let parser = LobsterParser::new();
            let rows = parser.parse_messages(&csv).unwrap();
            parser.extract_commands(&rows)
        })
        .as_slice()
}
