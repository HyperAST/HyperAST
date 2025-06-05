#!/bin/bash

set -e

# Step 1: Compile benchmarks in release mode
echo "Compiling in release mode..."
cargo build --release --bench bottom_up_bench

# Step 2: Run with valgrind massif for each algorithm/group
BENCH="./target/release/deps/bottom_up_bench-*"

# Resolve actual benchmark binary
BIN=$(ls $BENCH | grep -v '\.d$' | head -n 1)

declare -A GROUPS
GROUPS["HyperDiff"]="hyperdiff_group"
GROUPS["gumtree_greedy"]="greedy_group"
GROUPS["gumtree_simple"]="simple_group"

for algo in "${!GROUPS[@]}"; do
    echo "Running Valgrind Massif for $algo..."
    valgrind --tool=massif --massif-out-file="massif-${algo}.out" \
        "$BIN" --bench "${GROUPS[$algo]}"
done

echo "All done."
