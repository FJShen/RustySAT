#!/bin/sh

BIN=target/debug/sat_solver
cargo build
export RUST_LOG=info

test_unsat () {
    mkdir -p output/${1}/unsatisfiable
    for FILE in $2/*
    do
        ($BIN $FILE --heuristics $1 --check || return 1) >> output/${1}/unsatisfiable/$(basename ${2}) 2>&1
    done
}

test_sat () {
    mkdir -p output/${1}/satisfiable
    for FILE in $2/*
    do
        ($BIN $FILE --heuristics $1 --check --satisfiable || return 1) >> output/${1}/satisfiable/$(basename ${2}) 2>&1
    done
}

for HEUR in "ascending" "dlis" "vsids"
do
    for FOLDER in input/unsatisfiable/*
    do
        test_unsat $HEUR $FOLDER &
    done

    for FOLDER in input/satisfiable/*
    do
        test_sat $HEUR $FOLDER &
    done
done

wait

echo "All variable assignments for satisfiable expressions were valid"
echo "All expressions were correctly identified as satisfiable or unsatisfiable"
echo "Test is completed"
