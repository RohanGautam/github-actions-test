#!/bin/bash

# Prerequisites
sudo apt-get install \
    clang \
    gcc \
    g++ \
    zlib1g-dev \
    libmpc-dev \
    libmpfr-dev \
    libgmp-dev

# Add macOS Rust target
rustup target add x86_64-apple-darwin
## set up OSX compilation toolchain
git clone https://github.com/tpoechtrager/osxcross
cd osxcross
wget -nc https://s3.dockerproject.org/darwin/v2/MacOSX10.10.sdk.tar.xz
mv MacOSX10.10.sdk.tar.xz tarballs/
UNATTENDED=yes OSX_VERSION_MIN=10.7 ./build.sh
PATH="$(pwd)/osxcross/target/bin:$PATH" 

echo "the path is"
$PATH

## begin compilation
cargo build --release --target x86_64-apple-darwin
mkdir -p release
tar -cJf release/github-actions-test.xyz.x86_64-apple-darwin.tar.xz target/x86_64-apple-darwin/release/github-actions-test