#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# Arma trigonum-<versión>.tar.xz desde el árbol local, con los crates incluidos en vendor/
# para poder compilar sin red (Debian, OBS, RPM). Uso: dist-tarball.sh [directorio-de-salida]
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
outdir=$(mkdir -p "${1:-$root/packaging/arch}" && cd "${1:-$root/packaging/arch}" && pwd)
version=$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/Cargo.toml" | head -n1)
name=trigonum-$version
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT

mkdir "$stage/$name"
tar -C "$root" \
  --exclude-vcs \
  --exclude=./target \
  --exclude=./build \
  --exclude=./builddir \
  --exclude=./_build \
  --exclude=./vendor \
  --exclude=./ci-out \
  --exclude=./.cargo \
  --exclude=./debian/cargo-home \
  --exclude='./packaging/arch/src' \
  --exclude='./packaging/arch/pkg' \
  --exclude='./packaging/arch/*.tar.*' \
  --exclude='./packaging/arch/*.pkg.tar.*' \
  --exclude='*.code-workspace' \
  --exclude=./.vscode \
  --exclude=./.claude \
  -cf - . | tar -C "$stage/$name" -xf -

vendor() { cargo vendor --quiet --locked "$@" --manifest-path "$stage/$name/Cargo.toml" "$stage/$name/vendor" >/dev/null; }
vendor --offline 2>/dev/null || { rm -rf "$stage/$name/vendor"; vendor; }

tar -C "$stage" --sort=name --owner=0 --group=0 --numeric-owner \
  --mtime="@${SOURCE_DATE_EPOCH:-$(date +%s)}" \
  -cJf "$outdir/$name.tar.xz" "$name"
echo "$outdir/$name.tar.xz"
