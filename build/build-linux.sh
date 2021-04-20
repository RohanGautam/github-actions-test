#!/bin/bash

while getopts ":t:v:" opt; do
  case $opt in
    t) TARGET="$OPTARG"
    ;;
    v) VERSION="$OPTARG"
    ;;
    \?) echo "Invalid option -$OPTARG" >&2
    ;;
  esac
done

sudo apt-get install pkg-config libssl-dev
cargo install cross



cross build --target $TARGET --features local-redir --release
mkdir -p release
tar -cJf release/github-actions-test.$VERSION.$TARGET.tar.xz target/$TARGET/github-actions-test
