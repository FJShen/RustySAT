#!/bin/sh

BIN=target/debug/sat_solver

cargo build

for HEUR in "ascending" "dlis" "vsids"
    do
    for FILE in input/unsatisfiable/*
    do
        $BIN $FILE --heuristics $HEUR --check || return 1
    done

    for FILE in input/satisfiable/*
    do
        $BIN $FILE --heuristics $HEUR --check --satisfiable || return 1
    done
done

echo "All variable assignments for satisfiable expressions were valid"
echo "All expressions were correctly identified as satisfiable or unsatisfiable"
echo "Test is completed"
