#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# Uso: cargo-build.sh <cargo> <binario-generado> <destino> <argumentos de cargo...>
set -eu
cargo="$1"
built="$2"
output="$3"
shift 3
"$cargo" "$@"
cp "$built" "$output"
