#!/bin/bash

cargo build --example test-realtime
clear
RUST_LOG=info API_KEY=ieDYfZVXcfmLpVKzLVQ64C9BbdJznb6O ./target/debug/examples/test-realtime