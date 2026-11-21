#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# Verifica que todos los archivos que repiten la versión coincidan con Cargo.toml
# y que CHANGELOG.md tenga su sección. Sin dependencias más allá de sh, grep, sed y awk.
. "$(dirname "$0")/common.sh"
v=$(version)
errors=0
check() {
  if eval "$2"; then echo "ok    $1"; else echo "MAL   $1"; errors=$((errors + 1)); fi
}
lock_v=$(awk '/^name = "trigonum"$/ {getline; gsub(/version = |"/, ""); print; exit}' "$src/Cargo.lock")
dv=$(deb_version)

echo "Versión de Cargo.toml: $v"
check "Cargo.lock ($lock_v)" '[ "$lock_v" = "$v" ]'
check "meson.build" "grep -q \"^  version: '$v',\" \"\$src/meson.build\""
check "packaging/arch/PKGBUILD" "grep -qx 'pkgver=$v' \"\$src/packaging/arch/PKGBUILD\""
check "packaging/rpm/trigonum.spec" "grep -q '^Version: *$v\$' \"\$src/packaging/rpm/trigonum.spec\""
check "debian/changelog ($dv)" '[ "${dv%-*}" = "$v" ]'
for po in "$src"/po/*.po "$src"/po/trigonum.pot; do
  check "po/$(basename "$po")" "grep -q 'Project-Id-Version: trigonum $v\\\\n' '$po'"
done
check "metainfo: un solo <release> con @VERSION@" \
  '[ "$(grep -c "<release version=\"@VERSION@\"" "$src/data/trigonum.metainfo.xml.in.in")" = 1 ]'
check "CHANGELOG.md tiene la sección [$v]" "grep -q '^## \\[$v\\] - ' \"\$src/CHANGELOG.md\""

[ "$errors" -eq 0 ] || fail "$errors archivo(s) no coinciden. Usá build-aux/bump-version.sh."
