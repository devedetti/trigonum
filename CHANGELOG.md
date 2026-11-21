# Registro de cambios

Todos los cambios importantes de Trigonum se anotan en este archivo.

El formato sigue [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) y el
proyecto usa [Versionado Semántico](https://semver.org/lang/es/).

## [Unreleased]

## [0.1.2] - 2026-11-21

### Agregado

- Paquetes para Debian 13, Ubuntu 26.04, Fedora y openSUSE Tumbleweed, compilados en el
  Open Build Service (`home:nachopro`). El tarball de cada versión incluye los crates en
  `vendor/` para compilar sin red.
- CI en Forgejo: cada PR compila e instala el paquete real en Debian 13, Ubuntu 26.04,
  Arch, Fedora y openSUSE. `build-aux/ci/run-local.sh` corre lo mismo localmente con Docker.
- Release automático al mergear a `main` con una versión nueva: crea el tag, sube el paquete
  fuente a OBS y actualiza el AUR.
- `build-aux/bump-version.sh` cambia la versión en todos los archivos que la repiten.
- Imágenes de CI propias (`trigonum-ci/*`), con las herramientas y dependencias ya
  instaladas, para que cada job no las baje de nuevo. Se construyen con
  `build-aux/ci/build-images.sh`. Los crates quedan en la caché de Actions mientras no
  cambie el `Cargo.lock`, y clippy reutiliza lo compilado en la corrida anterior.
- El CI corre solo en los PR, o a mano desde la pestaña Actions. Antes corría dos veces
  por cada push a una rama con PR abierto.
- Timer semanal para el servidor del runner (`build-aux/ci/host/`): reconstruye las
  imágenes y limpia los contenedores, volúmenes y la caché de builds que dejan los jobs.
- `PKGBUILD.local` usa los crates incluidos en el tarball en lugar de bajarlos.
- En openSUSE, el CI reemplaza `busybox-gawk` por `gawk` antes de instalar `rpm-build`.
  Las imágenes nuevas de Tumbleweed traen `busybox-gawk`, que choca con `gawk`, y zypper
  cancelaba la instalación.

### Cambiado

- La versión mínima de Rust baja a 1.85, la que trae Debian 13. Para eso, gtk4-rs pasa de
  0.11 a 0.10 y libloading de 0.9 a 0.8. GTK en el sistema sigue pidiendo 4.12 o más nuevo.
- `is_multiple_of` y `as_chunks` se reemplazan por `% 3 == 0` y `chunks_exact`, que hacen
  lo mismo y existen en Rust 1.85.
- `build-aux/local-tarball.sh` pasa a ser `build-aux/dist-tarball.sh` y genera un `.tar.xz`
  con los crates incluidos.
- El PKGBUILD del repo queda como plantilla. El `.SRCINFO` y las sumas los genera el release.

## [0.1.1] - 2026-11-20

### Corregido

- El programa entraba en pánico al arrancar con el mensaje *"GTK has not been initialized"*.
