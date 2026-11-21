#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# Compila e instala el .deb en un contenedor Debian/Ubuntu (como root), igual que OBS.
. "$(dirname "$0")/common.sh"
export DEBIAN_FRONTEND=noninteractive
say "Herramientas"
apt-get update -qq
apt-get install -y -qq --no-install-recommends \
  ca-certificates cargo xz-utils dpkg-dev devscripts equivs lintian >/dev/null

out="$out/$(distro)"
OUT="$out" "$(dirname "$0")/source-package.sh"
dv=$(deb_version)

build=$(mktemp -d)
cd "$build"
say "Dependencias de compilación"
mk-build-deps -i -r -t "apt-get -y -qq --no-install-recommends" "$src/debian/control" >/dev/null
dpkg-source -x "$out/trigonum_$dv.dsc" trigonum

say "dpkg-buildpackage"
(cd trigonum && dpkg-buildpackage -us -uc -b)

say "Instalación"
apt-get install -y -qq "./trigonum_${dv}_$(dpkg --print-architecture).deb" >/dev/null
check_binary
check_translations "$(dpkg-deb -c "./trigonum_${dv}_$(dpkg --print-architecture).deb")"

say "lintian (informativo)"
lintian trigonum_*.changes || true
cp trigonum_*.deb "$out/"
