use pinocchio_small_oracle_shared_bench as benchmark;

#[test]
fn test_update_value_cu() {
    benchmark::run_cu_benchmark(
        "NAIVE",
        env!("CARGO_MANIFEST_DIR"),
        "pinocchio_small_oracle_naive",
    );
}
