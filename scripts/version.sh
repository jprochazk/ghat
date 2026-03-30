#!/usr/bin/env bash

set -e

if [ -z "$1" ]; then
  echo "Usage: $0 <new-version>"
  exit 1
fi

sed -i -E "0,/^version = \".*\"/s//version = \"$1\"/" Cargo.toml
cargo check
