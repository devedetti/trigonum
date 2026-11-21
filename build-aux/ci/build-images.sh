#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# Construye las imágenes de CI (trigonum-ci/<distro>) en el Docker local, con las herramientas
# y dependencias ya instaladas y los crates del Cargo.lock ya bajados, para que cada job no los
# baje de nuevo. La de Arch además trae compiladas las dependencias de clippy.
# Uso: build-aux/ci/build-images.sh [debian ubuntu arch fedora opensuse]   (sin argumentos: todas)
set -eu
root=$(cd "$(dirname "$0")/../.." && pwd)
ctx=$(mktemp -d)
trap 'rm -rf "$ctx"' EXIT
cp "$root"/build-aux/ci/images/*.Containerfile "$root/debian/control" \
   "$root/packaging/rpm/trigonum.spec" "$root/packaging/arch/PKGBUILD" "$ctx/"
mkdir "$ctx/cargo"
cp -r "$root/Cargo.toml" "$root/Cargo.lock" "$root/src" "$ctx/cargo/"

build() {
  name=$1 file=$2 base=$3
  shift 3
  echo "### trigonum-ci/$name (desde $base)"
  docker build --pull -q -t "trigonum-ci/$name:latest" -f "$ctx/$file" --build-arg "BASE=$base" "$@" "$ctx"
}

[ $# -gt 0 ] || set -- debian ubuntu arch fedora opensuse
for d in "$@"; do
  case $d in
    debian) build debian deb.Containerfile debian:trixie --build-arg EXTRA=osc ;;
    ubuntu) build ubuntu deb.Containerfile ubuntu:26.04 ;;
    arch) build arch arch.Containerfile archlinux:latest ;;
    fedora) build fedora rpm.Containerfile fedora:latest ;;
    opensuse) build opensuse rpm.Containerfile opensuse/tumbleweed:latest ;;
    *) echo "distro desconocida: $d" >&2; exit 2 ;;
  esac
done
docker image ls trigonum-ci
