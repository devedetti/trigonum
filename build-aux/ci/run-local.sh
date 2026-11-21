#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# Corre localmente con Docker lo mismo que el CI de Forgejo.
# Uso: build-aux/ci/run-local.sh <debian|ubuntu|arch|fedora|opensuse|all>
# Los paquetes quedan en ci-out/<distro>/.
set -eu
root=$(cd "$(dirname "$0")/../.." && pwd)

image_of() {
  case $1 in
    debian) echo debian:trixie ;;
    ubuntu) echo ubuntu:26.04 ;;
    arch) echo archlinux:latest ;;
    fedora) echo fedora:latest ;;
    opensuse) echo opensuse/tumbleweed:latest ;;
    *) echo "distro desconocida: $1" >&2; exit 2 ;;
  esac
}
script_of() {
  case $1 in
    debian | ubuntu) echo build-deb.sh ;;
    arch) echo build-arch.sh ;;
    fedora | opensuse) echo build-rpm.sh ;;
  esac
}

run() {
  image=$(image_of "$1")
  mkdir -p "$root/ci-out"
  echo "### $1 ($image)"
  status=0
  docker run --rm -v "$root":/src:ro -v "$root/ci-out":/out -e SRC=/src -e OUT=/out \
    "$image" sh "/src/build-aux/ci/$(script_of "$1")" || status=$?
  docker run --rm -v "$root/ci-out":/out "$image" chown -R "$(id -u):$(id -g)" /out
  return $status
}

[ $# -eq 1 ] || { sed -n '3,5p' "$0"; exit 2; }
if [ "$1" = all ]; then
  failed=""
  for d in debian ubuntu arch fedora opensuse; do
    run "$d" || failed="$failed $d"
  done
  [ -z "$failed" ] || { echo "Fallaron:$failed" >&2; exit 1; }
  echo "Todas las distros compilaron."
else
  run "$1"
fi
