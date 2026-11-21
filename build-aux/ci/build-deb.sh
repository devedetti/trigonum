#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# Compila e instala el .deb en un contenedor Debian/Ubuntu (como root), igual que OBS.
. "$(dirname "$0")/common.sh"
export DEBIAN_FRONTEND=noninteractive
apt_updated=0
apt_update() { [ "$apt_updated" = 1 ] || { apt-get update -qq; apt_updated=1; }; }
if ! in_ci_image; then
  say "Herramientas"
  apt_update
  apt-get install -y -qq --no-install-recommends \
    ca-certificates cargo xz-utils dpkg-dev devscripts equivs lintian >/dev/null
fi

out="$out/$(distro)"
OUT="$out" "$(dirname "$0")/source-package.sh"
dv=$(deb_version)

build=$(mktemp -d)
cd "$build"
if ! dpkg-checkbuilddeps "$src/debian/control" 2>/dev/null; then
  say "Dependencias de compilación"
  apt_update
  mk-build-deps -i -r -t "apt-get -y -qq --no-install-recommends" "$src/debian/control" >/dev/null
fi
dpkg-source -x "$out/trigonum_$dv.dsc" trigonum

say "dpkg-buildpackage"
(cd trigonum && dpkg-buildpackage -us -uc -b)

say "Instalación"
apt_update
apt-get install -y -qq "./trigonum_${dv}_$(dpkg --print-architecture).deb" >/dev/null
check_binary
check_translations "$(dpkg-deb -c "./trigonum_${dv}_$(dpkg --print-architecture).deb")"

say "lintian (informativo)"
lintian trigonum_*.changes || true
cp trigonum_*.deb "$out/"
