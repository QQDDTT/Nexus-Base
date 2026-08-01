#!/bin/bash
set -e

echo "==========================================="
echo "   Nexus-Base Benchmark Suite (Automated) "
echo "==========================================="

echo ">> 1. Environment Cleanup"
rm -f public.key private.key
rm -f /tmp/nexus_main.log /tmp/nexus_snapshot.nxpack

echo ">> 2. Generating Admin Keys (Keygen)"
cargo run -q -p cli -- keygen

echo ">> 3. Starting Nexus Server (Release Mode)"
cargo run --release -q -p server &
SERVER_PID=$!
sleep 2

echo ">> 4. Running the Performance Benchmark Suite (Release Mode)"
# 运行压测 binary
cargo run --release -q -p nexus-test --bin perf_bench

echo ">> 5. Cleaning up"
kill $SERVER_PID
rm -f public.key private.key

echo "==========================================="
echo "        Benchmark Completed!               "
echo "==========================================="
