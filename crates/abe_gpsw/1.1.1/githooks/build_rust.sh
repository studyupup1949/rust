#!/bin/sh
set -ex

cd /root/project
cargo build --release --features ffi
