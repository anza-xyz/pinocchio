TOOLCHAIN_BUILD := "1.84.1"
TOOLCHAIN_FORMAT := "nightly-2025-02-16"
TOOLCHAIN_LINT := "nightly-2025-02-16"
TOOLCHAIN_TEST := "1.84.1"

CRATE_FOLDERS := \
    "programs/associated-token-account " + \
    "programs/memo " + \
    "programs/system " + \
    "programs/token " + \
    "programs/token-2022 " + \
    "sdk"

# Ignore known RustSec advisories.
audit:
    @cargo audit \
        --ignore RUSTSEC-2022-0093 \
        --ignore RUSTSEC-2024-0421 \
        --ignore RUSTSEC-2024-0344 \
        --ignore RUSTSEC-2024-0376 \

# Build Solana SBF program with all targets & features.
build-sbf folder *args:
    @cargo-build-sbf \
        --manifest-path {{folder}}/Cargo.toml \
        {{args}} \
        -- \
        --all-targets \
        --all-features \

# Lint a crate using Clippy.
clippy folder *args:
    @cargo +{{TOOLCHAIN_LINT}} clippy \
        --manifest-path {{folder}}/Cargo.toml \
        -Zunstable-options \
        --all-targets \
        --all-features \
        --no-deps \
        -- \
        --deny=warnings \
        {{args}} \

# Lint a crate using Clippy, with --fix.
clippy-fix folder *args:
    @cargo +{{TOOLCHAIN_LINT}} clippy \
        --manifest-path {{folder}}/Cargo.toml \
        --fix \
        -Zunstable-options \
        --all-targets \
        --all-features \
        --no-deps \
        -- \
        --deny=warnings \
        {{args}} \

# Generate documentation for a specific folder with all features and no dependencies.
doc folder *args:
    @cargo +{{TOOLCHAIN_LINT}} doc \
        --manifest-path {{folder}}/Cargo.toml \
        --all-features \
        --no-deps \
        {{args}} \

# Format a crate using fmt.
format folder:
    @cargo +{{TOOLCHAIN_FORMAT}} fmt --manifest-path {{folder}}/Cargo.toml --all -- --check

# Format a crate using fmt, with --fix.
format-fix folder:
    @cargo +{{TOOLCHAIN_FORMAT}} fmt --manifest-path {{folder}}/Cargo.toml --all

# Check feature permutations using cargo-hack.
hack folder *args:
    @cargo +{{TOOLCHAIN_LINT}} hack check \
        --manifest-path {{folder}}/Cargo.toml \
        --all-targets \
        --feature-powerset \
        {{args}} \

# Run clippy, doc, and hack checks.
lint folder *args:
    @just clippy {{folder}} {{args}}
    @just doc {{folder}} {{args}}
    @just hack {{folder}} {{args}}

# Run miri tests to detect undefined behaviors.
miri *args:
    @cargo +{{TOOLCHAIN_LINT}} miri test {{args}}

# List members.
members:
    @echo {{trim(CRATE_FOLDERS)}}

# Check semver compatibility.
semver folder *args:
    @cargo +stable semver-checks --manifest-path {{folder}}/Cargo.toml {{args}}

# Run spellcheck on the crate.
spellcheck:
    @cargo spellcheck -j1 --code 1

# Run tests on specific folder.
test folder *args:
    @cargo +{{TOOLCHAIN_LINT}} test \
        --manifest-path {{folder}}/Cargo.toml \
        --all-features \
        {{args}}

# Run all tests.
test-all *args:
    #!/usr/bin/env bash
    for folder in {{CRATE_FOLDERS}}; do \
        echo "Testing $folder..."; \
        just test $folder {{args}}; \
    done

# Publish a crate.
publish folder level *args:
    #!/usr/bin/env bash
    set -e

    MANIFEST_PATH="{{folder}}/Cargo.toml"

    if [ -z "{{level}}" ]; then
        echo "Error: A version level — e.g. \"patch\" — must be provided."
        exit 1
    fi

    DRY_RUN=false
    if [[ "{{args}}" == *"--dry-run"* ]]; then
        DRY_RUN=true
    fi

    METADATA=$(cargo metadata --no-deps --format-version 1 --manifest-path "$MANIFEST_PATH")
    NAME=$(echo "$METADATA" | jq -r '.packages[0].name')
    PREVIOUS=$(echo "$METADATA" | jq -r '.packages[0].version')

    cd "{{folder}}"

    if [ "$DRY_RUN" = true ]; then
        echo "Running dry run for $NAME..."
        cargo release {{level}} --dry-run
        exit 0
    else
        cargo release {{level}} --tag-name "${NAME}@v{{ '{{' }}version{{ '}}' }}" --no-confirm --execute
    fi

    VERSION=$(cargo metadata --no-deps --format-version 1 --manifest-path "Cargo.toml" | jq -r '.packages[0].version')

    NEW_GIT_TAG="${NAME}@v${VERSION}"
    OLD_GIT_TAG="${NAME}@v${PREVIOUS}"

    if [ -n "${CI:-}" ]; then
        echo "new_git_tag=${NEW_GIT_TAG}" >> "$GITHUB_OUTPUT"
        echo "old_git_tag=${OLD_GIT_TAG}" >> "$GITHUB_OUTPUT"
    fi

    echo "Successfully released $NEW_GIT_TAG"
