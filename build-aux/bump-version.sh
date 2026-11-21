#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# Cambia la versión en todos los archivos que la repiten y cierra la sección [Unreleased]
# de CHANGELOG.md. Uso: build-aux/bump-version.sh X.Y.Z
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

new=${1:-}
echo "$new" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$' || { echo "Uso: $0 X.Y.Z" >&2; exit 2; }
branch=$(git branch --show-current 2>/dev/null || true)
if [ "$branch" != "release/$new" ]; then
  echo "La versión solo se sube en la rama release/$new (estás en '${branch:-ninguna}')." >&2
  echo "Renombrá next en Forgejo (Ramas → ✏️ en next → release/$new) y después:" >&2
  echo "  git fetch -p origin && git switch release/$new" >&2
  exit 1
fi
old=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)
[ "$new" != "$old" ] || { echo "Ya está en $new" >&2; exit 1; }
today=$(date +%F)
repo=https://git.local.net.ar/nachopro/trigonum

notes=$(awk '/^## \[Unreleased\]/ {on=1; next} on && /^## \[/ {exit} on && NF {print}' CHANGELOG.md)
[ -n "$notes" ] || { echo "CHANGELOG.md no tiene notas en [Unreleased]" >&2; exit 1; }

sed -i "0,/^version = \"$old\"/s//version = \"$new\"/" Cargo.toml
cargo update --offline --quiet -p trigonum
sed -i "s/^  version: '$old',/  version: '$new',/" meson.build
sed -i "s/^pkgver=.*/pkgver=$new/; s/^pkgrel=.*/pkgrel=1/" packaging/arch/PKGBUILD
sed -i "s/^Version:\( *\).*/Version:\1$new/" packaging/rpm/trigonum.spec
sed -i "s/\"Project-Id-Version: trigonum $old\\\\n\"/\"Project-Id-Version: trigonum $new\\\\n\"/" po/*.po po/trigonum.pot

tmp=$(mktemp)
{
  printf 'trigonum (%s-1) unstable; urgency=medium\n\n' "$new"
  printf '  * Nueva versión. Ver CHANGELOG.md.\n\n'
  printf ' -- nachopro <tech@local.net.ar>  %s\n\n' "$(LC_ALL=C date -R)"
  cat debian/changelog
} > "$tmp"
mv "$tmp" debian/changelog

anchor=$(printf '%s---%s' "$(echo "$new" | tr -d .)" "$today")
awk -v old="$old" -v today="$today" -v anchor="$anchor" '
  /<release version="@VERSION@"/ && !done {
    print "    <release version=\"@VERSION@\" date=\"" today "\">"
    print "      <url type=\"details\">https://github.com/devedetti/trigonum/blob/main/CHANGELOG.md#" anchor "</url>"
    print "    </release>"
    sub(/@VERSION@/, old)
    done = 1
  }
  { print }
' data/trigonum.metainfo.xml.in.in > "$tmp"
mv "$tmp" data/trigonum.metainfo.xml.in.in

awk -v new="$new" -v old="$old" -v today="$today" -v repo="$repo" '
  /^## \[Unreleased\]$/ { print; print ""; print "## [" new "] - " today; next }
  /^\[Unreleased\]: / {
    print "[Unreleased]: " repo "/compare/v" new "...HEAD"
    print "[" new "]: " repo "/compare/v" old "...v" new
    next
  }
  { print }
' CHANGELOG.md > "$tmp"
mv "$tmp" CHANGELOG.md

"$root/build-aux/ci/check-version.sh"
echo
echo "Listo: $old → $new. Revisá el diff, commiteá, pusheá y abrí el PR de release/$new hacia main (el RC)."
