# SPDX-License-Identifier: GPL-3.0-or-later
# Imagen de CI para Arch con las herramientas y dependencias del PKGBUILD ya instaladas, los crates
# ya bajados y las dependencias de clippy ya compiladas en $CLIPPY_TARGET_DIR.
# La usan el lint, el build de Arch y el release al AUR. La construye build-aux/ci/build-images.sh.
ARG BASE=archlinux:latest
FROM ${BASE}
ENV TRIGONUM_CI_IMAGE=1 CARGO_HOME=/root/.cargo CLIPPY_TARGET_DIR=/var/cache/trigonum/clippy-target
SHELL ["/bin/bash", "-c"]
COPY PKGBUILD /tmp/ci/PKGBUILD
COPY cargo /tmp/ci/cargo
RUN pacman -Syu --noconfirm --needed \
      base-devel git namcap rust sudo nodejs openssh pacman-contrib \
 && source /tmp/ci/PKGBUILD \
 && pacman -S --noconfirm --needed "${depends[@]}" "${makedepends[@]}" "${checkdepends[@]}" \
 && useradd -m builder \
 && echo 'builder ALL=(ALL) NOPASSWD: ALL' > /etc/sudoers.d/builder \
 && pacman -Scc --noconfirm \
 && cd /tmp/ci/cargo \
 && cargo fetch --locked \
 && CARGO_TARGET_DIR=$CLIPPY_TARGET_DIR cargo clippy --locked --all-targets \
 && cd / && rm -rf /tmp/ci
