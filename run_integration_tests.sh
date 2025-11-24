#!/bin/bash

set -e

echo "Building the project..."
cargo build --release

echo ""
echo "Running integration tests..."
echo "Note: This will start and stop the signing server multiple times."
echo ""

cd integration_tests
cargo test -- --nocapture --test-threads=1

echo ""
echo "All integration tests passed!"
