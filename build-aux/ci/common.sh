# SPDX-License-Identifier: GPL-3.0-or-later
# Funciones compartidas por los scripts de build-aux/ci. Se incluye con `. common.sh`.
set -eu
src=${SRC:-$(cd "$(dirname "$0")/../.." && pwd)}
out=${OUT:-$src/ci-out}

say() { printf '\n==> %s\n' "$*"; }
fail() { printf 'ERROR: %s\n' "$*" >&2; exit 1; }
version() { sed -n 's/^version = "\(.*\)"/\1/p' "$src/Cargo.toml" | head -n1; }
deb_version() { sed -n '1s/^trigonum (\([^)]*\)).*/\1/p' "$src/debian/changelog"; }
in_ci_image() { [ "${TRIGONUM_CI_IMAGE:-0}" = 1 ]; }
distro() { . /etc/os-release; echo "$ID-${VERSION_ID:-rolling}"; }

check_binary() {
  test -x /usr/bin/trigonum || fail "/usr/bin/trigonum no quedó instalado"
  if ldd /usr/bin/trigonum | grep 'not found'; then fail "faltan bibliotecas"; fi
  for f in /usr/share/applications/ar.net.local.Trigonum.desktop \
           /usr/share/metainfo/ar.net.local.Trigonum.metainfo.xml \
           /usr/share/icons/hicolor/scalable/apps/ar.net.local.Trigonum.svg; do
    test -f "$f" || fail "falta $f"
  done
  say "Paquete instalado y completo"
}

check_translations() {
  for lang in es pt_BR; do
    printf '%s\n' "$1" | grep -q "usr/share/locale/$lang/LC_MESSAGES/trigonum.mo" \
      || fail "el paquete no trae la traducción $lang"
  done
  say "Traducciones incluidas en el paquete"
}
