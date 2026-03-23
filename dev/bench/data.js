window.BENCHMARK_DATA = {
  "lastUpdate": 1774287716822,
  "repoUrl": "https://github.com/bruno-medeiros/rust-low-latency-hft",
  "entries": {
    "Rust benchmarks (throughput)": [
      {
        "commit": {
          "author": {
            "email": "bruno.medeiros@cryptio.co",
            "name": "Bruno Medeiros"
          },
          "committer": {
            "email": "bruno.medeiros@cryptio.co",
            "name": "Bruno Medeiros"
          },
          "distinct": true,
          "id": "96f74a096e74fe0c64f25dbf93759b564bcbe6c9",
          "message": "remove mlockall",
          "timestamp": "2026-03-23T16:19:27Z",
          "tree_id": "1a6a10b2beac521357c6bcefe2022a48e8e902e3",
          "url": "https://github.com/bruno-medeiros/rust-low-latency-hft/commit/96f74a096e74fe0c64f25dbf93759b564bcbe6c9"
        },
        "date": 1774283926984,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "LOB v0 / Throughput (realistic mix) / Throughput (realistic mix)",
            "value": 13855750.76013197,
            "unit": "ops/s"
          },
          {
            "name": "LOB v1 / Throughput (realistic mix) / Throughput (realistic mix)",
            "value": 28903853.476317823,
            "unit": "ops/s"
          },
          {
            "name": "Pipeline / Pipeline (Lobster data)",
            "value": 8053693.506047206,
            "unit": "ops/s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "bruno.medeiros@cryptio.co",
            "name": "Bruno Medeiros"
          },
          "committer": {
            "email": "bruno.medeiros@cryptio.co",
            "name": "Bruno Medeiros"
          },
          "distinct": true,
          "id": "7eab970dcc25141c0592212ad22a971a0ed07946",
          "message": "remove caches",
          "timestamp": "2026-03-23T16:41:00Z",
          "tree_id": "6750e29e6a5e638033fabc2c9b9c2675238e22ee",
          "url": "https://github.com/bruno-medeiros/rust-low-latency-hft/commit/7eab970dcc25141c0592212ad22a971a0ed07946"
        },
        "date": 1774285205411,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "LOB v0 / Throughput (realistic mix) / Throughput (realistic mix)",
            "value": 16027732.882372769,
            "unit": "ops/s"
          },
          {
            "name": "LOB v1 / Throughput (realistic mix) / Throughput (realistic mix)",
            "value": 29307200.765404828,
            "unit": "ops/s"
          },
          {
            "name": "Pipeline / Pipeline (Lobster data)",
            "value": 7971885.105640924,
            "unit": "ops/s"
          }
        ]
      },
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
          "id": "3cda0940e7a352e8a4fe67bd715f4e7843812920",
          "message": "remove caches",
          "timestamp": "2026-03-23T16:41:00Z",
          "tree_id": "6750e29e6a5e638033fabc2c9b9c2675238e22ee",
          "url": "https://github.com/bruno-medeiros/rust-low-latency-hft/commit/3cda0940e7a352e8a4fe67bd715f4e7843812920"
        },
        "date": 1774287711411,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "LOB v0 / Throughput (realistic mix) / Throughput (realistic mix)",
            "value": 15530561.93290831,
            "unit": "ops/s"
          },
          {
            "name": "LOB v1 / Throughput (realistic mix) / Throughput (realistic mix)",
            "value": 29170103.620452262,
            "unit": "ops/s"
          },
          {
            "name": "Pipeline / Pipeline (Lobster data)",
            "value": 8094928.927629125,
            "unit": "ops/s"
          }
        ]
      }
    ],
    "Rust benchmarks (latency)": [
      {
        "commit": {
          "author": {
            "email": "bruno.medeiros@cryptio.co",
            "name": "Bruno Medeiros"
          },
          "committer": {
            "email": "bruno.medeiros@cryptio.co",
            "name": "Bruno Medeiros"
          },
          "distinct": true,
          "id": "96f74a096e74fe0c64f25dbf93759b564bcbe6c9",
          "message": "remove mlockall",
          "timestamp": "2026-03-23T16:19:27Z",
          "tree_id": "1a6a10b2beac521357c6bcefe2022a48e8e902e3",
          "url": "https://github.com/bruno-medeiros/rust-low-latency-hft/commit/96f74a096e74fe0c64f25dbf93759b564bcbe6c9"
        },
        "date": 1774283929152,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "LOB v0 / Latency / Add (passive) (mean latency)",
            "value": 97.74431,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Add (sweep 5 levels, 50 fills) (mean latency)",
            "value": 2484.15334,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Market (sweep 10 levels, 100 fills) (mean latency)",
            "value": 4849.696359999996,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Cancel (head of queue) (mean latency)",
            "value": 74.58350000000007,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Cancel (tail of queue) (mean latency)",
            "value": 207.70739999999984,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Spread (BBO query) (mean latency)",
            "value": 14.507359999999991,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Depth (top 5) (mean latency)",
            "value": 80.51931000000003,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Order lookup (hit) (mean latency)",
            "value": 39.84324,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Realistic mix (per-op) (mean latency)",
            "value": 111.78913999999996,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Add (passive) (mean latency)",
            "value": 90.0209,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Add (sweep 5 levels, 50 fills) (mean latency)",
            "value": 1457.3816899999997,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Market (sweep 10 levels, 100 fills) (mean latency)",
            "value": 2729.020309999996,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Cancel (head of queue) (mean latency)",
            "value": 82.13820999999999,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Cancel (tail of queue) (mean latency)",
            "value": 50.44391000000003,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Spread (BBO query) (mean latency)",
            "value": 11.586050000000004,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Depth (top 5) (mean latency)",
            "value": 271.47145000000006,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Order lookup (hit) (mean latency)",
            "value": 19.58863,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Realistic mix (per-op) (mean latency)",
            "value": 109.96696000000009,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "bruno.medeiros@cryptio.co",
            "name": "Bruno Medeiros"
          },
          "committer": {
            "email": "bruno.medeiros@cryptio.co",
            "name": "Bruno Medeiros"
          },
          "distinct": true,
          "id": "7eab970dcc25141c0592212ad22a971a0ed07946",
          "message": "remove caches",
          "timestamp": "2026-03-23T16:41:00Z",
          "tree_id": "6750e29e6a5e638033fabc2c9b9c2675238e22ee",
          "url": "https://github.com/bruno-medeiros/rust-low-latency-hft/commit/7eab970dcc25141c0592212ad22a971a0ed07946"
        },
        "date": 1774285207520,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "LOB v0 / Latency / Add (passive) (mean latency)",
            "value": 96.49027999999998,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Add (sweep 5 levels, 50 fills) (mean latency)",
            "value": 2448.2230399999985,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Market (sweep 10 levels, 100 fills) (mean latency)",
            "value": 4816.495139999999,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Cancel (head of queue) (mean latency)",
            "value": 86.82148,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Cancel (tail of queue) (mean latency)",
            "value": 208.07283000000004,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Spread (BBO query) (mean latency)",
            "value": 13.32558,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Depth (top 5) (mean latency)",
            "value": 81.87276999999996,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Order lookup (hit) (mean latency)",
            "value": 38.36272,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Realistic mix (per-op) (mean latency)",
            "value": 115.38941,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Add (passive) (mean latency)",
            "value": 81.81005000000005,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Add (sweep 5 levels, 50 fills) (mean latency)",
            "value": 1486.2923899999982,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Market (sweep 10 levels, 100 fills) (mean latency)",
            "value": 2766.3357500000025,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Cancel (head of queue) (mean latency)",
            "value": 119.31292999999997,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Cancel (tail of queue) (mean latency)",
            "value": 72.69739999999997,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Spread (BBO query) (mean latency)",
            "value": 11.352570000000004,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Depth (top 5) (mean latency)",
            "value": 271.7425200000004,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Order lookup (hit) (mean latency)",
            "value": 20.306220000000003,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Realistic mix (per-op) (mean latency)",
            "value": 113.43873000000004,
            "unit": "ns"
          }
        ]
      },
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
          "id": "3cda0940e7a352e8a4fe67bd715f4e7843812920",
          "message": "remove caches",
          "timestamp": "2026-03-23T16:41:00Z",
          "tree_id": "6750e29e6a5e638033fabc2c9b9c2675238e22ee",
          "url": "https://github.com/bruno-medeiros/rust-low-latency-hft/commit/3cda0940e7a352e8a4fe67bd715f4e7843812920"
        },
        "date": 1774287716233,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "LOB v0 / Latency / Add (passive) (mean latency)",
            "value": 93.23646000000004,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Add (sweep 5 levels, 50 fills) (mean latency)",
            "value": 2539.9104300000013,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Market (sweep 10 levels, 100 fills) (mean latency)",
            "value": 4995.022399999995,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Cancel (head of queue) (mean latency)",
            "value": 82.86734999999997,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Cancel (tail of queue) (mean latency)",
            "value": 208.0597,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Spread (BBO query) (mean latency)",
            "value": 14.028370000000002,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Depth (top 5) (mean latency)",
            "value": 74.08542000000001,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Order lookup (hit) (mean latency)",
            "value": 32.832140000000024,
            "unit": "ns"
          },
          {
            "name": "LOB v0 / Latency / Realistic mix (per-op) (mean latency)",
            "value": 119.59048999999996,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Add (passive) (mean latency)",
            "value": 81.79414000000006,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Add (sweep 5 levels, 50 fills) (mean latency)",
            "value": 1487.1724500000023,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Market (sweep 10 levels, 100 fills) (mean latency)",
            "value": 2787.587170000004,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Cancel (head of queue) (mean latency)",
            "value": 94.58464999999998,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Cancel (tail of queue) (mean latency)",
            "value": 54.16187000000001,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Spread (BBO query) (mean latency)",
            "value": 12.15502,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Depth (top 5) (mean latency)",
            "value": 304.4718899999998,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Order lookup (hit) (mean latency)",
            "value": 17.020690000000005,
            "unit": "ns"
          },
          {
            "name": "LOB v1 / Latency / Realistic mix (per-op) (mean latency)",
            "value": 111.25270000000006,
            "unit": "ns"
          }
        ]
      }
    ],
    "Rust benchmarks (Criterion / cargo)": [
      {
        "commit": {
          "author": {
            "email": "bruno.medeiros@cryptio.co",
            "name": "Bruno Medeiros"
          },
          "committer": {
            "email": "bruno.medeiros@cryptio.co",
            "name": "Bruno Medeiros"
          },
          "distinct": true,
          "id": "96f74a096e74fe0c64f25dbf93759b564bcbe6c9",
          "message": "remove mlockall",
          "timestamp": "2026-03-23T16:19:27Z",
          "tree_id": "1a6a10b2beac521357c6bcefe2022a48e8e902e3",
          "url": "https://github.com/bruno-medeiros/rust-low-latency-hft/commit/96f74a096e74fe0c64f25dbf93759b564bcbe6c9"
        },
        "date": 1774283932816,
        "tool": "cargo",
        "benches": [
          {
            "name": "add_limit_order/sweep_5_levels/10/50fills",
            "value": 4971,
            "range": "± 600",
            "unit": "ns/iter"
          },
          {
            "name": "add_limit_order/sweep_5_levels/100/50fills",
            "value": 9475,
            "range": "± 189",
            "unit": "ns/iter"
          },
          {
            "name": "add_limit_order/sweep_5_levels/1000/50fills",
            "value": 63195,
            "range": "± 883",
            "unit": "ns/iter"
          },
          {
            "name": "cancel_order//head/100",
            "value": 139,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "cancel_order//tail/100",
            "value": 191,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cancel_order//head/500",
            "value": 150,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "cancel_order//tail/500",
            "value": 328,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "cancel_order//head/1000",
            "value": 161,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "cancel_order//tail/1000",
            "value": 502,
            "range": "± 23",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "bruno.medeiros@cryptio.co",
            "name": "Bruno Medeiros"
          },
          "committer": {
            "email": "bruno.medeiros@cryptio.co",
            "name": "Bruno Medeiros"
          },
          "distinct": true,
          "id": "7eab970dcc25141c0592212ad22a971a0ed07946",
          "message": "remove caches",
          "timestamp": "2026-03-23T16:41:00Z",
          "tree_id": "6750e29e6a5e638033fabc2c9b9c2675238e22ee",
          "url": "https://github.com/bruno-medeiros/rust-low-latency-hft/commit/7eab970dcc25141c0592212ad22a971a0ed07946"
        },
        "date": 1774285209210,
        "tool": "cargo",
        "benches": [
          {
            "name": "add_limit_order/sweep_5_levels/10/50fills",
            "value": 4983,
            "range": "± 702",
            "unit": "ns/iter"
          },
          {
            "name": "add_limit_order/sweep_5_levels/100/50fills",
            "value": 9623,
            "range": "± 341",
            "unit": "ns/iter"
          },
          {
            "name": "add_limit_order/sweep_5_levels/1000/50fills",
            "value": 63006,
            "range": "± 2969",
            "unit": "ns/iter"
          },
          {
            "name": "cancel_order//head/100",
            "value": 143,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "cancel_order//tail/100",
            "value": 204,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "cancel_order//head/500",
            "value": 161,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "cancel_order//tail/500",
            "value": 338,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "cancel_order//head/1000",
            "value": 175,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "cancel_order//tail/1000",
            "value": 522,
            "range": "± 19",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}