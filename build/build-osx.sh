#!/bin/bash
CUR_DIR=$( cd $( dirname $0 ) && pwd )
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

echo "done 🤖"

## begin compilation
# cd ${CUR_DIR} && 
#     PATH="$CUR_DIR/osxcross/target/bin:$PATH" \ 
#     CC=o64-clang \
#     CXX=o64-clang++ \
#     cargo build --release --target x86_64-apple-darwin

PATH="$CUR_DIR/osxcross/target/bin:$PATH" \
    CC=o64-clang \
    CXX=o64-clang++ \
    LIBZ_SYS_STATIC=1 \
    cargo build --target x86_64-apple-darwin;

# mkdir -p ${CUR_DIR}/release
# ls ${CUR_DIR}
# tar -cJf ${CUR_DIR}/release/github-actions-test.xyz.x86_64-apple-darwin.tar.xz ${CUR_DIR}target/x86_64-apple-darwin/release/github-actions-test