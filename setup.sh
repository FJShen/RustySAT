#!/bin/bash

# download and install cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# so that cargo is in PATH
source $HOME/.cargo/env

# compile
cargo build --release
