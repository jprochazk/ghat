#!/usr/bin/env bash

set -e

VERSION=$(sed -nE '0,/^version = "([^"]+)"/s//\1/p' Cargo.toml)

if [ -z "$VERSION" ]; then
  echo "Failed to read version from Cargo.toml"
  exit 1
fi

echo "About to:"
echo "  - commit with message: v$VERSION"
echo "  - create tag: v$VERSION"
echo "  - push to remote"
echo "  - cargo publish"
echo
read -p "Continue? [y/N] " CONFIRM

case "$CONFIRM" in
  [yY][eE][sS]|[yY])
    ;;
  *)
    echo "Aborted."
    exit 0
    ;;
esac

git commit -am "v$VERSION"
git tag "v$VERSION"
git push
git push origin "v$VERSION"
cargo publish

echo "Done."
