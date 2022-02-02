#!/usr/bin/env bash
set -euo pipefail

TARGET="wasm32-unknown-unknown"
SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

pushd $SCRIPT_DIR

# NOTE: On macOS a specific version of llvm-ar and clang need to be set here.
# Otherwise the wasm compilation of rust-secp256k1 will fail.
if [ "$(uname)" == "Darwin" ]; then
  # On macs we need to use the brew versions
  AR="/usr/local/opt/llvm/bin/llvm-ar" CC="/usr/local/opt/llvm/bin/clang" cargo build --target $TARGET --release
else
  cargo build --target $TARGET --release
fi

cargo install ic-cdk-optimizer --version 0.3.1 --root "$SCRIPT_DIR"/../target
STATUS=$?

if [ "$STATUS" -eq "0" ]; then
      "$SCRIPT_DIR"/../target/bin/ic-cdk-optimizer \
      "$SCRIPT_DIR"/target/$TARGET/release/example.wasm \
      -o "$SCRIPT_DIR"/target/$TARGET/release/example.wasm
  true
else
  echo Could not install ic-cdk-optimizer.
  false
fi

popd
