#!/usr/bin/env bash

set -euo pipefail

if [ $# -ne 1 ]; then
  echo "Usage: $0 <new-version|major|minor|patch>"
  exit 1
fi

INPUT="$1"
CURRENT_VERSION=$(sed -nE '0,/^version = "([^"]+)"/s//\1/p' Cargo.toml)

if [ -z "$CURRENT_VERSION" ]; then
  echo "Failed to read version from Cargo.toml"
  exit 1
fi

if [[ "$CURRENT_VERSION" =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)$ ]]; then
  CURRENT_MAJOR="${BASH_REMATCH[1]}"
  CURRENT_MINOR="${BASH_REMATCH[2]}"
  CURRENT_PATCH="${BASH_REMATCH[3]}"
else
  echo "Unsupported current version format: $CURRENT_VERSION"
  exit 1
fi

case "$INPUT" in
  major)
    NEW_VERSION="$((CURRENT_MAJOR + 1)).0.0"
    ;;
  minor)
    NEW_VERSION="${CURRENT_MAJOR}.$((CURRENT_MINOR + 1)).0"
    ;;
  patch)
    NEW_VERSION="${CURRENT_MAJOR}.${CURRENT_MINOR}.$((CURRENT_PATCH + 1))"
    ;;
  *)
    if [[ "$INPUT" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
      NEW_VERSION="$INPUT"
    else
      echo "Invalid version: $INPUT"
      echo "Expected an exact version like 1.2.3 or one of: major, minor, patch"
      exit 1
    fi
    ;;
esac

echo "Bumping version: $CURRENT_VERSION -> $NEW_VERSION"

cargo insta test
cargo clippy
cargo fmt --check

sed -i -E "0,/^version = \".*\"/s//version = \"$NEW_VERSION\"/" Cargo.toml
cargo check
