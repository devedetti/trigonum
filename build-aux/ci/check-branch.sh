#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# Reglas del flujo de ramas: feature → next → (rename) release/X.Y.Z → main.
# next es temporal: sale de main al empezar un ciclo y se renombra a release/X.Y.Z para el RC.
# El flujo completo está en CONTRIBUTING.md.
# Variables: EVENT (pull_request o workflow_dispatch), BASE_REF (rama destino del PR),
# HEAD_REF (rama de origen). Necesita el historial y los tags (checkout con fetch-depth: 0).
# Imprime "completo=true|false": si corresponde el CI completo con paquetes y artefactos.
. "$(dirname "$0")/common.sh"

event=${EVENT:-workflow_dispatch}
base=${BASE_REF:-}
head=${HEAD_REF:-}
v=$(version)

version_of() {
  git -C "$src" show "origin/$1:Cargo.toml" 2>/dev/null | sed -n 's/^version = "\(.*\)"/\1/p' | head -n1
}
same_version_as() {
  bv=$(version_of "$1")
  [ -n "$bv" ] || fail "no pude leer la versión de origin/$1"
  [ "$v" = "$bv" ] || fail "la versión de tu rama ($v) no coincide con la de $1 ($bv). Si la subiste, eso solo se hace en release/X.Y.Z con build-aux/bump-version.sh. Si tu rama quedó atrasada, traele los cambios: git merge origin/$1."
}

if [ "$event" != pull_request ]; then
  echo "Corrida manual: CI completo." >&2
  echo completo=true
  exit 0
fi

echo "PR: $head → $base (versión $v)" >&2
case $base in
  main)
    rv=$(printf '%s\n' "$head" | sed -n 's|^release/\([0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*\)$|\1|p')
    if [ -z "$rv" ]; then
      [ "$head" = next ] && fail "main solo acepta ramas release/X.Y.Z: renombrá next en Forgejo (Ramas → ✏️) a release/X.Y.Z y corré build-aux/bump-version.sh X.Y.Z."
      fail "main solo acepta ramas release/X.Y.Z (este PR viene de '$head'). Mandá tu rama a next."
    fi
    [ "$rv" = "$v" ] || fail "la rama es release/$rv pero Cargo.toml dice $v. Corré build-aux/bump-version.sh $rv en la rama."
    if git -C "$src" rev-parse -q --verify "refs/tags/v$v" >/dev/null; then
      fail "la versión $v ya está publicada (existe el tag v$v)"
    fi
    echo "RC de la versión $v: CI completo." >&2
    echo completo=true
    ;;
  next)
    same_version_as next
    echo completo=false
    ;;
  release/*)
    same_version_as "$base"
    echo completo=false
    ;;
  *)
    echo completo=false
    ;;
esac
