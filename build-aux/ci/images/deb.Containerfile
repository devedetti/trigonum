# SPDX-License-Identifier: GPL-3.0-or-later
# Imagen de CI para Debian y Ubuntu con las herramientas, las dependencias y los crates ya instalados.
# La construye build-aux/ci/build-images.sh.
ARG BASE=debian:trixie
FROM ${BASE}
ARG EXTRA=""
ENV DEBIAN_FRONTEND=noninteractive TRIGONUM_CI_IMAGE=1 CARGO_HOME=/root/.cargo
COPY control /tmp/ci/control
COPY cargo /tmp/ci/cargo
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      ca-certificates cargo xz-utils dpkg-dev devscripts equivs lintian nodejs git $EXTRA \
 && cd /tmp/ci \
 && mk-build-deps -i -r -t "apt-get -y --no-install-recommends" control \
 && cargo fetch --locked --manifest-path /tmp/ci/cargo/Cargo.toml \
 && cd / && rm -rf /tmp/ci /var/lib/apt/lists/*
