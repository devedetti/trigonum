#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# Compila e instala el .rpm en un contenedor Fedora u openSUSE (como root).
. "$(dirname "$0")/common.sh"
spec="$src/packaging/rpm/trigonum.spec"
v=$(version)
grep -q "^Version: *$v$" "$spec" || fail "trigonum.spec no está en la versión $v"

if command -v dnf >/dev/null; then
  install_tools() { dnf -y -q install rpm-build cargo xz 'dnf-command(builddep)'; }
  install_builddeps() { dnf -y -q builddep "$spec"; }
  install_rpm() { dnf -y -q install "$@"; }
else
  install_tools() {
    zypper -n -q refresh
    if rpm -q busybox-gawk >/dev/null 2>&1; then zypper -n -q install gawk '!busybox-gawk'; fi
    zypper -n -q install rpm-build cargo xz
  }
  install_builddeps() { rpmspec -q --buildrequires "$spec" | sed 's/ //g' | xargs zypper -n -q install; }
  install_rpm() { zypper -n -q install --allow-unsigned-rpm "$@"; }
fi
missing_builddeps() {
  rpmspec -q --buildrequires "$spec" | while read -r req; do
    rpm -q --whatprovides "$req" >/dev/null 2>&1 || echo "$req"
  done
}

if ! in_ci_image; then
  say "Herramientas"
  install_tools
fi
if [ -n "$(missing_builddeps)" ]; then
  say "Dependencias de compilación"
  install_builddeps
fi

top=$(mktemp -d)
mkdir -p "$top/SOURCES"
"$src/build-aux/dist-tarball.sh" "$top" >/dev/null
mv "$top/trigonum-$v.tar.xz" "$top/SOURCES/trigonum_$v.orig.tar.xz"

say "rpmbuild"
rpmbuild -ba --define "_topdir $top" "$spec"

say "Instalación"
install_rpm "$top"/RPMS/*/trigonum-"$v"-*.rpm
check_binary
check_translations "$(rpm -qlp "$top"/RPMS/*/trigonum-"$v"-*.rpm)"
out="$out/$(distro)"
mkdir -p "$out"
cp "$top"/RPMS/*/*.rpm "$out/"
