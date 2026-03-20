//! End-to-end pipeline throughput on the LOBSTER GOOG sample (same file and config as
//! `tests/lobster_goog.rs`).

use bench_tool::{BenchReportSection, BenchRunner, CliArgs};
use limit_order_book::LimitOrderBookV1;
use matching_pipeline::{Pipeline, PipelineConfig, test_data};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse_args();
    let commands = test_data::goog_sample_commands();
    assert_eq!(commands.len(), 37_797);

    let pipeline_config = PipelineConfig {
        queue_slots: 4096,
        price_range: (5_500_000, 5_900_000),
        order_capacity: 30_000,
    };

    const THROUGHPUT_ITERS: u64 = 40;

    let mut runner = BenchRunner::new("Matching pipeline")
        .warmup_iters(2)
        .sample_iters(THROUGHPUT_ITERS)
        .filter(args.filter.clone());

    runner.apply_core_pinning();

    let mut report = runner.initial_report();

    runner.run_throughput(
        "Pipeline (Lobster data)",
        // FIXME
        || (),
        |_, sink, op_count| {
            let cmds = test_data::goog_sample_commands().to_vec();
            let n = cmds.len() as u64;
            let result = Pipeline::new(pipeline_config).run::<LimitOrderBookV1>(cmds);
            sink.accepted += result.events.accepted;
            sink.rejected += result.events.rejected;
            sink.fill += result.events.fill;
            sink.filled += result.events.filled;
            sink.cancelled += result.events.cancelled;
            *op_count += n;
            std::hint::black_box(result);
        },
        THROUGHPUT_ITERS,
    );

    let mut section = BenchReportSection::new("");

    section.add_param("sample", test_data::GOOG_SAMPLE_MESSAGE_REL_PATH);
    section.add_param("queue_slots", pipeline_config.queue_slots.to_string());

    runner.push_section(section, &mut report);
    args.execute(&report)
}
