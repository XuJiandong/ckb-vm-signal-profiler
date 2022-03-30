#!/bin/bash

set -e
BIN=${BIN:-~/projects/rvv-prototype/bn128-example/target/riscv64imac-unknown-none-elf/release/alt-bn128-example-rvv-asm-bench}
target/release/ckb-signal-profiler --bin ${BIN}
pprof --top ${BIN} simple.profile
