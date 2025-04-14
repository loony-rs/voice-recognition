#!/bin/bash

cargo build --example websocket
clear
RUST_LOG=info API_KEY=ieDYfZVXcfmLpVKzLVQ64C9BbdJznb6O ./target/debug/examples/websocket