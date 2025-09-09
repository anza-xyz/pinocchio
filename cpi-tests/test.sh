# 1. build and test with "show-output":     ./test.sh s
# 2. build and test:                        ./test.sh

(
    clear

    for program in programs/*/; do
        if [ -d "$program" ]; then
            echo "Building $(basename "$program")..."
            cd "$program" && cargo build-sbf && cd - > /dev/null
        fi
    done
)

if [[ "$1" == "s" ]]; then
    (cd tests && clear && cargo test -- --show-output)
else
    (cd tests && clear && cargo test)
fi
