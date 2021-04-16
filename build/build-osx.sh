#!/bin/bash

while getopts ":v:" opt; do
  case $opt in
    v) VERSION="$OPTARG"
    ;;
    \?) echo "Invalid option -$OPTARG" >&2
    ;;
  esac
done

MACOS_TARGET="x86_64-apple-darwin"

echo "Building target for platform ${MACOS_TARGET}"
echo

# Add osxcross toolchain to path
export PATH="$(pwd)/osxcross/target/bin:$PATH"

# Make libz-sys (git2-rs -> libgit2-sys -> libz-sys) build as a statically linked lib
# This prevents the host zlib from being linked
export LIBZ_SYS_STATIC=1

# Use Clang for C/C++ builds
export CC=o64-clang
export CXX=o64-clang++

cargo build --release --target "${MACOS_TARGET}"
mkdir -p release
tar -cJf release/github-actions-test.${VERSION}.x86_64-apple-darwin.tar.xz target/x86_64-apple-darwin/release/github-actions-test

echo
echo Done

