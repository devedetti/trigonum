#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# Compila e instala el paquete de Arch con el código local (PKGBUILD.local), como root.
. "$(dirname "$0")/common.sh"
if ! in_ci_image; then
  say "Herramientas"
  pacman -Syu --noconfirm --needed -q base-devel git namcap rust sudo >/dev/null
fi
id builder >/dev/null 2>&1 || useradd -m builder
echo 'builder ALL=(ALL) NOPASSWD: ALL' > /etc/sudoers.d/builder

w=$(mktemp -d)
cp "$src/packaging/arch/PKGBUILD" "$src/packaging/arch/PKGBUILD.local" "$w/"
"$src/build-aux/dist-tarball.sh" "$w" >/dev/null
chown -R builder "$w"

say "makepkg"
sudo -u builder -H sh -c "cd '$w' && makepkg -s --noconfirm -p PKGBUILD.local"

say "Instalación"
pkg=$(ls "$w"/trigonum-[0-9]*.pkg.tar.zst)
pacman -U --noconfirm "$pkg" >/dev/null
check_binary
check_translations "$(pacman -Qlpq "$pkg")"

say "namcap (informativo)"
namcap "$w/PKGBUILD" "$pkg" || true
out="$out/$(distro)"
mkdir -p "$out"
cp "$pkg" "$out/"
