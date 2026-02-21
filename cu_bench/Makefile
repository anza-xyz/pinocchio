.PHONY: test-opt test-manual test-naive

SMALL_ORACLE_MANIFEST := small_oracle/Cargo.toml

test-opt:
	cargo build-sbf --manifest-path $(SMALL_ORACLE_MANIFEST) --no-default-features --features opt
	cargo test --manifest-path $(SMALL_ORACLE_MANIFEST) --no-default-features --features opt --test cu_benchmark

test-manual:
	cargo build-sbf --manifest-path $(SMALL_ORACLE_MANIFEST) --no-default-features --features manual
	cargo test --manifest-path $(SMALL_ORACLE_MANIFEST) --no-default-features --features manual --test cu_benchmark

test-naive:
	cargo build-sbf --manifest-path $(SMALL_ORACLE_MANIFEST) --no-default-features --features naive
	cargo test --manifest-path $(SMALL_ORACLE_MANIFEST) --no-default-features --features naive --test cu_benchmark
