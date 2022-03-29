#!/bin/bash

set -e
BIN=${BIN:-~/projects/rvv-testcases/cases/target/riscv64imac-unknown-none-elf/release/rvv-testcases}
target/release/ckb-vm-signal-profiler ${BIN}
pprof --top ${BIN} simple.profile
