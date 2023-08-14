#!/bin/bash

cargo build --release

target_dir="$HOME/.local/bin"

if [ ! -d "$target_dir" ]; then
	mkdir -p "$target_dir"
fi

cp ./target/release/tollo "$target_dir"

echo "Installation is complete to $target_dir. You can run the program with 'tollo'"
