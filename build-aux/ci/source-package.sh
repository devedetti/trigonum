#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# Genera el paquete fuente: trigonum_<v>.orig.tar.xz (con vendor/), .dsc, .debian.tar.xz y el .spec.
# Requiere dpkg-dev, cargo y xz. Deja todo en $OUT (por defecto ci-out/).
. "$(dirname "$0")/common.sh"
v=$(version)
dv=$(deb_version)
[ "${dv%-*}" = "$v" ] || fail "debian/changelog dice $dv pero Cargo.toml dice $v"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
say "Tarball con crates incluidos"
"$src/build-aux/dist-tarball.sh" "$tmp" >/dev/null
mv "$tmp/trigonum-$v.tar.xz" "$tmp/trigonum_$v.orig.tar.xz"
tar -C "$tmp" -xJf "$tmp/trigonum_$v.orig.tar.xz"

say "Paquete fuente de Debian"
(cd "$tmp" && dpkg-source -b "trigonum-$v")

mkdir -p "$out"
mv "$tmp/trigonum_$v.orig.tar.xz" "$tmp/trigonum_$dv.dsc" "$tmp/trigonum_$dv.debian.tar.xz" "$out/"
cp "$src/packaging/rpm/trigonum.spec" "$out/"
ls -l "$out"
