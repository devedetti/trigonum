#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# Actualiza el paquete del AUR con la versión actual (contenedor archlinux, como root).
# Espera a que el tag esté en GitHub, porque el PKGBUILD baja el tarball de ahí.
# Variables: AUR_SSH_KEY (clave privada), DRY_RUN=1 para no pushear, AUR_WAIT_SECONDS (1800).
. "$(dirname "$0")/common.sh"
aur_host_key='aur.archlinux.org ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIEuBKrPzbawxA/k2g6NcyV5jmqwJ2s+zpgZGZ7tpLIcN'
if ! in_ci_image; then
  say "Herramientas"
  pacman -Syu --noconfirm --needed -q base-devel git openssh pacman-contrib sudo >/dev/null
fi
id builder >/dev/null 2>&1 || useradd -m builder

v=$(version)
tag=v$v
waited=0
until git ls-remote --exit-code --tags https://github.com/devedetti/trigonum.git "refs/tags/$tag" >/dev/null; do
  [ "$waited" -lt "${AUR_WAIT_SECONDS:-1800}" ] || fail "el tag $tag no llegó a GitHub; revisá el push mirror de Forgejo"
  echo "Esperando a que $tag aparezca en GitHub..."
  sleep 30
  waited=$((waited + 30))
done

w=$(mktemp -d)
cp "$src/packaging/arch/PKGBUILD" "$w/"
chown -R builder "$w"
say "Sumas y .SRCINFO para $tag"
sudo -u builder -H sh -c "cd '$w' && updpkgsums && makepkg --printsrcinfo > .SRCINFO && makepkg --verifysource --noconfirm"
grep -E '^(pkgver|pkgrel|sha256sums)=' "$w/PKGBUILD"

if [ "${DRY_RUN:-0}" = 1 ]; then
  say "DRY_RUN: no se pushea al AUR"
  cat "$w/.SRCINFO"
  exit 0
fi

mkdir -p "$HOME/.ssh"
umask 077
printf '%s\n' "${AUR_SSH_KEY:?falta AUR_SSH_KEY}" > "$HOME/.ssh/aur"
printf '%s\n' "$aur_host_key" > "$HOME/.ssh/known_hosts_aur"
export GIT_SSH_COMMAND="ssh -i $HOME/.ssh/aur -o IdentitiesOnly=yes -o UserKnownHostsFile=$HOME/.ssh/known_hosts_aur -o StrictHostKeyChecking=yes"

say "Publicando en el AUR"
git clone ssh://aur@aur.archlinux.org/trigonum.git "$w/aur"
cp "$w/PKGBUILD" "$w/.SRCINFO" "$w/aur/"
cd "$w/aur"
git add PKGBUILD .SRCINFO
if git diff --cached --quiet; then
  echo "El AUR ya tiene trigonum $v; no hay nada que subir."
  exit 0
fi
git -c user.name=nachopro -c user.email=tech@local.net.ar commit -q -m "trigonum $v"
git push origin HEAD:master
