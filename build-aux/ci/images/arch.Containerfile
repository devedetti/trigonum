# SPDX-License-Identifier: GPL-3.0-or-later
# Imagen de CI para Arch con las herramientas y dependencias del PKGBUILD ya instaladas.
# La usan el lint, el build de Arch y el release al AUR. La construye build-aux/ci/build-images.sh.
ARG BASE=archlinux:latest
FROM ${BASE}
ENV TRIGONUM_CI_IMAGE=1 CARGO_HOME=/root/.cargo
SHELL ["/bin/bash", "-c"]
COPY PKGBUILD /tmp/ci/PKGBUILD
RUN pacman -Syu --noconfirm --needed \
      base-devel git namcap rust sudo nodejs openssh pacman-contrib \
 && source /tmp/ci/PKGBUILD \
 && pacman -S --noconfirm --needed "${depends[@]}" "${makedepends[@]}" "${checkdepends[@]}" \
 && useradd -m builder \
 && echo 'builder ALL=(ALL) NOPASSWD: ALL' > /etc/sudoers.d/builder \
 && pacman -Scc --noconfirm \
 && rm -rf /tmp/ci
