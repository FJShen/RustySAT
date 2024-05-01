#!/bin/sh

BIN=target/debug/sat_solver
export RUST_LOG=error

cargo build

for FILE in input/satisfiable/*
do
    $BIN --input $FILE --heuristics dlis --satisfiable || return 1
done
