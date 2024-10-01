#!/bin/bash

TIME=gtime
which $TIME


RUSTFLAGS=-Ctarget-cpu=native $TIME -f "Peak memory: %M kb CPU usage: %P" cargo run --release --package plonky2_bench_poseidon2 --bin plonky2_bench_poseidon2
