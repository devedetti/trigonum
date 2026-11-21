# SPDX-License-Identifier: GPL-3.0-or-later
# Imagen de CI para Fedora y openSUSE con las herramientas, las dependencias y los crates ya instalados.
# La construye build-aux/ci/build-images.sh.
ARG BASE=fedora:latest
FROM ${BASE}
ENV TRIGONUM_CI_IMAGE=1 CARGO_HOME=/root/.cargo
COPY trigonum.spec /tmp/ci/trigonum.spec
COPY cargo /tmp/ci/cargo
RUN if command -v dnf >/dev/null; then \
      dnf -y install rpm-build cargo xz nodejs git-core 'dnf-command(builddep)' \
      && dnf -y builddep /tmp/ci/trigonum.spec \
      && dnf clean all; \
    else \
      { ! rpm -q busybox-gawk >/dev/null 2>&1 || zypper -n install gawk '!busybox-gawk'; } \
      && zypper -n install rpm-build cargo xz nodejs-default git-core tar gzip \
      && rpmspec -q --buildrequires /tmp/ci/trigonum.spec | sed 's/ //g' | xargs zypper -n install \
      && zypper clean -a; \
    fi \
 && cargo fetch --locked --manifest-path /tmp/ci/cargo/Cargo.toml \
 && rm -rf /tmp/ci
