#!/bin/bash

sudo apt-get update

#Wasm target
rustup target add wasm32-unknown-unknown
#Trunk for the dev server
cargo install --locked trunk