#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# Limpieza conservadora de Docker en el servidor del runner. Solo toca lo que dejan los jobs:
# contenedores y volúmenes de Actions sin usar, imágenes huérfanas (las versiones viejas
# de trigonum-ci que quedan al reconstruir) y la caché de builds de más de una semana.
# No borra imágenes con nombre: las imágenes base y trigonum-ci/* son justamente la caché.
set -eu
docker ps -a --filter status=exited --format '{{.ID}} {{.Names}}' \
  | awk '$2 ~ /^(FORGEJO|GITEA)-ACTIONS/ {print $1}' | xargs -r docker rm >/dev/null
docker volume ls -q --filter dangling=true | grep -E '^(act-|FORGEJO-ACTIONS|GITEA-ACTIONS)' \
  | xargs -r docker volume rm >/dev/null || true
docker image prune -f >/dev/null
docker builder prune -f --filter until=168h >/dev/null
docker system df
