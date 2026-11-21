#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# Sube el paquete fuente de la versión actual a OBS (contenedor Debian, como root).
# Variables: OBS_USER, OBS_PASSWORD, OBS_PROJECT (por defecto home:$OBS_USER), DRY_RUN=1 para no subir.
. "$(dirname "$0")/common.sh"
export DEBIAN_FRONTEND=noninteractive
if ! in_ci_image; then
  say "Herramientas"
  apt-get update -qq
  apt-get install -y -qq --no-install-recommends ca-certificates cargo xz-utils dpkg-dev osc >/dev/null
fi

v=$(version)
pkgdir=$(mktemp -d)
OUT="$pkgdir/new" "$(dirname "$0")/source-package.sh"

if [ "${DRY_RUN:-0}" = 1 ]; then
  say "DRY_RUN: no se sube nada a OBS"
  exit 0
fi

project=${OBS_PROJECT:-home:${OBS_USER:?falta OBS_USER}}
mkdir -p "$HOME/.config/osc"
umask 077
cat > "$HOME/.config/osc/oscrc" <<OSCRC
[general]
apiurl = https://api.opensuse.org

[https://api.opensuse.org]
user = ${OBS_USER:?falta OBS_USER}
pass = ${OBS_PASSWORD:?falta OBS_PASSWORD}
credentials_mgr_class = osc.credentials.PlaintextConfigFileCredentialsManager
OSCRC

say "Subiendo trigonum $v a $project"
cd "$pkgdir"
osc checkout --output-dir pkg "$project" trigonum
cd pkg
rm -f ./*.dsc ./*.tar.xz ./*.spec
cp ../new/* .
osc addremove
osc commit -m "trigonum $v"
