.PHONY: help sbf sbf-opt sbf-naive sbf-manual test-bench test-bench-opt test-bench-naive test-bench-manual bench test

# Build all small-oracle crates with SBF artifacts.
# Run with `make -jN` for parallel compilation.
sbf: \
	sbf-opt \
	sbf-naive \
	sbf-manual

sbf-opt:
	cargo build-sbf --manifest-path small-oracle/small-oracle-opt/Cargo.toml

sbf-naive:
	cargo build-sbf --manifest-path small-oracle/small-oracle-naive/Cargo.toml

sbf-manual:
	cargo build-sbf --manifest-path small-oracle/small-oracle-manual/Cargo.toml

# Run all cu_benchmark integration tests.
# Bench tests depend on corresponding SBF artifacts.
test-bench: \
	test-bench-opt \
	test-bench-naive \
	test-bench-manual

test-bench-opt: sbf-opt
	cargo test -p pinocchio-small-oracle-opt --test cu_benchmark

test-bench-naive: sbf-naive
	cargo test -p pinocchio-small-oracle-naive --test cu_benchmark

test-bench-manual: sbf-manual
	cargo test -p pinocchio-small-oracle-manual --test cu_benchmark

# Explicit aliases
bench: test-bench

test: bench

help:
	@echo "Targets:"
	@echo "  sbf             Build opt + naive + manual small-oracle SBF artifacts (use -jN for parallelism)"
	@echo "  sbf-opt         Build opt small-oracle SBF artifact"
	@echo "  sbf-naive       Build naive small-oracle SBF artifact"
	@echo "  sbf-manual      Build manual small-oracle SBF artifact"
	@echo "  test-bench      Run cu_benchmark for opt + naive + manual (runs sbf* prerequisites first)"
	@echo "  test-bench-opt  Run opt cu_benchmark (builds sbf-opt first)"
	@echo "  test-bench-naive Run naive cu_benchmark (builds sbf-naive first)"
	@echo "  test-bench-manual Run manual cu_benchmark (builds sbf-manual first)"
	@echo "  bench           Alias for test-bench"
	@echo "  test            Alias for bench"
