#!/bin/bash

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git checkout submission
source $HOME/.cargo/env
cargo build --release
