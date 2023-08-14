#!/bin/bash

cargo build --release

target_dir="$HOME/.local/bin"

if [ ! -d "$target_dir" ]; then
	mkdir -p "$target_dir"
fi

cp ./target/release/tollo "$target_dir"

if [ $? -eq 0 ]; then
	echo "Installation is completed to $target_dir. You can run the program with 'tollo'"
else
	echo "Installation failed. Make sure you have rust installed."
fi
