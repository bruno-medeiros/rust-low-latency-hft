window.BENCHMARK_DATA = {
  "lastUpdate": 1774291196407,
  "repoUrl": "https://github.com/bruno-medeiros/rust-low-latency-hft",
  "entries": {
    "Rust benchmarks (throughput)": [
      {
        "commit": {
          "author": {
            "email": "bruno.do.medeiros@gmail.com",
            "name": "Bruno Medeiros",
            "username": "bruno-medeiros"
          },
          "committer": {
            "email": "bruno.do.medeiros@gmail.com",
            "name": "Bruno Medeiros",
            "username": "bruno-medeiros"
          },
          "distinct": true,
          "id": "f32fd42984a0c6111f61c6f323a28c8c7a55cd3b",
          "message": "remove criterion",
          "timestamp": "2026-03-23T18:21:45Z",
          "tree_id": "098fb97038e8b78cc7c19d3c60b23c53e2915aa2",
          "url": "https://github.com/bruno-medeiros/rust-low-latency-hft/commit/f32fd42984a0c6111f61c6f323a28c8c7a55cd3b"
        },
        "date": 1774291195201,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "LOB v0 / Throughput (realistic mix) / Throughput (realistic mix)",
            "value": 17229581.034152564,
            "unit": "ops/s"
          },
          {
            "name": "LOB v1 / Throughput (realistic mix) / Throughput (realistic mix)",
            "value": 29340793.074140225,
            "unit": "ops/s"
          },
          {
            "name": "Pipeline / Pipeline (Lobster data)",
            "value": 7833133.308346142,
            "unit": "ops/s"
          }
        ]
      }
    ],
    "Rust benchmarks (latency)": [
      {
        "commit": {
          "author": {
            "email": "bruno.do.medeiros@gmail.com",
            "name": "Bruno Medeiros",
            "username": "bruno-medeiros"
          },
          "committer": {
            "email": "bruno.do.medeiros@gmail.com",
            "name": "Bruno Medeiros",
            "username": "bruno-medeiros"
          },
          "distinct": true,
          "id": "f32fd42984a0c6111f61c6f323a28c8c7a55cd3b",
          "message": "remove criterion",
          "timestamp": "2026-03-23T18:21:45Z",
          "tree_id": "098fb97038e8b78cc7c19d3c60b23c53e2915aa2",
          "url": "https://github.com/bruno-medeiros/rust-low-latency-hft/commit/f32fd42984a0c6111f61c6f323a28c8c7a55cd3b"
        },
        "date": 1774291196186,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "LOB v0 / Latency / Add (passive) (mean latency)",
            "value": 94.41625000000003,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Add (sweep 5 levels, 50 fills) (mean latency)",
            "value": 2488.67291,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Market (sweep 10 levels, 100 fills) (mean latency)",
            "value": 4827.769660000004,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Cancel (head of queue) (mean latency)",
            "value": 83.83221,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Cancel (tail of queue) (mean latency)",
            "value": 209.1778199999999,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Spread (BBO query) (mean latency)",
            "value": 14.33846,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Depth (top 5) (mean latency)",
            "value": 81.42784999999998,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Order lookup (hit) (mean latency)",
            "value": 35.15423000000004,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Realistic mix (per-op) (mean latency)",
            "value": 117.35393,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Add (passive) (mean latency)",
            "value": 82.02024,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Add (sweep 5 levels, 50 fills) (mean latency)",
            "value": 1462.7972600000023,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Market (sweep 10 levels, 100 fills) (mean latency)",
            "value": 2738.689630000003,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Cancel (head of queue) (mean latency)",
            "value": 104.37970999999996,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Cancel (tail of queue) (mean latency)",
            "value": 57.46499000000001,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Spread (BBO query) (mean latency)",
            "value": 11.423989999999998,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Depth (top 5) (mean latency)",
            "value": 268.58869000000016,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Order lookup (hit) (mean latency)",
            "value": 22.38195,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Realistic mix (per-op) (mean latency)",
            "value": 111.79368999999994,
            "unit": "ns"
          }
        ]
      }
    ]
  }
}