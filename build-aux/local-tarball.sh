#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# Arma packaging/arch/trigonum-<versión>.tar.gz desde el árbol local, con la misma
# estructura que el tarball que genera GitHub para un tag (carpeta trigonum-<versión>/).
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
version=$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/Cargo.toml" | head -n1)
name=trigonum-$version
out="$root/packaging/arch/$name.tar.gz"

tar -C "$root" \
  --exclude-vcs \
  --exclude=./target \
  --exclude=./build \
  --exclude=./_build \
  --exclude='./packaging/arch/src' \
  --exclude='./packaging/arch/pkg' \
  --exclude='./packaging/arch/*.tar.gz' \
  --exclude='./packaging/arch/*.pkg.tar.zst' \
  --exclude='*.code-workspace' \
  --exclude=./.vscode \
  --exclude=./.claude \
  --transform "s|^\.|$name|" \
  -czf "$out" .
echo "$out"
