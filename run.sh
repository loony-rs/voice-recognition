#!/bin/bash

cargo build --example websocket
clear
RUST_LOG=info \
API_KEY=evK20Lpk7TTRtpNAv0Cbh4pCBzvr32Y6 \
PORT=2011 \
./target/debug/examples/websocket