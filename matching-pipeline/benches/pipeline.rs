//! End-to-end pipeline throughput on the LOBSTER GOOG sample (same file and config as
//! `tests/lobster_goog.rs`).

use bench_tool::{BenchReportSection, BenchRunner, CliArgs, core_pinning_disabled_by_env};
use limit_order_book::LimitOrderBookV1;
use matching_pipeline::{Pipeline, PipelineConfig, test_data};
use std::ops::Not;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse_args();
    let cmds = test_data::goog_sample_commands().to_vec();
    assert_eq!(cmds.len(), 37_797);

    let pipeline_config = PipelineConfig {
        queue_slots: 4096,
        price_range: (5_500_000, 5_900_000),
        order_capacity: 30_000,
        core_pinning_enabled: core_pinning_disabled_by_env().not(),
        producer_pin_core: args.pin_core,
        consumer_pin_core: args.pin_core_b,
    };

    const THROUGHPUT_ITERS: u64 = 40;

    let mut runner = BenchRunner::new("Matching pipeline")
        .warmup_iters(2)
        .sample_iters(THROUGHPUT_ITERS)
        .filter(args.filter.clone());

    let mut report = runner.initial_report();

    runner.run_throughput(
        "Pipeline (Lobster data)",
        || Pipeline::new::<LimitOrderBookV1>(pipeline_config),
        |pipeline| pipeline.run_and_terminate(&[]).events,
        move |pipeline, op_count| {
            let n = cmds.len() as u64;
            *op_count += n;
            #[allow(clippy::unit_arg)]
            std::hint::black_box(pipeline.ingest_commands(&cmds));
        },
        THROUGHPUT_ITERS,
    );

    let mut section = BenchReportSection::new("");

    section.add_param("sample", test_data::GOOG_SAMPLE_MESSAGE_REL_PATH);
    section.add_param("queue_slots", pipeline_config.queue_slots.to_string());

    // Note: threads were pinned by pipeline, this is just to get useful msg for metadata.
    let pin_core_note = runner.pin_to_isolated_core(pipeline_config.producer_pin_core);
    let pin_core_b_note = runner.pin_to_isolated_core(pipeline_config.consumer_pin_core);
    section.add_param("producer_pin_core", pin_core_note);
    section.add_param("consumer_pin_core", pin_core_b_note);

    runner.push_section(section, &mut report);
    args.execute(&report)
}
