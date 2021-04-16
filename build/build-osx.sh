#!/bin/bash
CUR_DIR = cd $( dirname $0 ) && pwd 

## set up OSX compilation toolchain
git clone https://github.com/tpoechtrager/osxcross
cd osxcross
wget -nc https://s3.dockerproject.org/darwin/v2/MacOSX10.10.sdk.tar.xz
mv MacOSX10.10.sdk.tar.xz tarballs/
UNATTENDED=yes OSX_VERSION_MIN=10.7 ./build.sh

## begin compilation
cd CUR_DIR
PATH="$(pwd)/osxcross/target/bin:$PATH" 
cargo build --target x86_64-apple-darwin
mkdir -p release
tar -cJf release/github-actions-test.${{ steps.get_version.outputs.version }}.x86_64-apple-darwin.tar.xz target/x86_64-apple-darwin/release/github-actions-test