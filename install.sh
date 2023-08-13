#!/bin/bash

cargo build --release

if [ ! -d "$HOME/.local/bin" ]; then
	mkdir -p "$HOME/.local/bin"
fi

cp ./target/release/tollo ~/.local/bin/

echo "Installation is complete. You can run the program with 'tollo'"
