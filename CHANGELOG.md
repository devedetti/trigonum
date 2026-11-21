# Registro de cambios

Todos los cambios importantes de Trigonum se anotan en este archivo.

El formato sigue [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) y el
proyecto usa [Versionado Semántico](https://semver.org/lang/es/).

## [Unreleased]

### Agregado

- Pestañas: cada archivo se abre en su propia pestaña en vez de reemplazar al anterior, con
  su cámara, su modo malla y su información. El panel de información, la barra de estado y
  el título de la ventana siguen a la pestaña visible. Reabrir un archivo que ya está abierto
  vuelve a su pestaña y lo relee.
- Abrir varios archivos a la vez: selección múltiple en el diálogo, arrastrar varios juntos o
  pasarlos por línea de comandos (antes solo se tomaba el primero).
- Las pestañas que no están a la vista se parsean recién la primera vez que se muestran, y si
  su archivo cambia en disco mientras están ocultas se releen al volver a ellas. Así abrir
  muchos modelos juntos no dispara la RAM ni la CPU.
- Ver → Pestaña siguiente / anterior (`Ctrl+Tab` / `Ctrl+Mayús+Tab`); también funcionan
  `Ctrl+AvPág` / `Ctrl+RePág`, y `Alt+1`…`Alt+8` saltan a esa pestaña (`Alt+9`, a la
  última). Las pestañas se pueden reordenar arrastrándolas.
- La ventana recuerda su tamaño, si estaba maximizada y si el panel de información estaba
  abierto (`~/.config/trigonum/window.ini`). La posición no se puede guardar: en GTK 4 la
  decide el gestor de ventanas.
- Instrucciones de instalación para cada distro en los READMEs.
- Flujo de ramas feature → `next` → `release/X.Y.Z` → `main`. `next` es temporal: sale de
  `main` en cada ciclo y se renombra a `release/X.Y.Z` para el RC, así no queda ninguna rama
  de integración desincronizada. `main` solo acepta ramas `release/X.Y.Z`, y son las únicas
  que pueden subir la versión (`build-aux/ci/check-branch.sh`). Al publicar, el release avisa
  si quedó una `next` que no incluye la versión nueva.
- `CONTRIBUTING.md`: el flujo de ramas, las reglas del CI y cómo publicar una versión.
- README en portugués y `CONTRIBUTING.md` en inglés y portugués.
- El CI completo, con los paquetes de las 5 distros como artefactos para probar, corre en el
  PR de cada RC (`release/X.Y.Z` → `main`) y a demanda desde Actions en cualquier rama. En el
  resto de los PR solo corren las reglas de versión y clippy.

### Cambiado

- La traducción al portugués pasa de `pt_BR` a `pt`, así la toman también Portugal y el resto
  de los países de habla portuguesa (gettext cae de `pt_PT` a `pt`, pero nunca a `pt_BR`).
- Archivo → Cerrar pasa a ser Cerrar pestaña (`Ctrl+W`, o clic del medio en la pestaña).
  Antes cerraba la ventana, que con una sola ventana era lo mismo que Salir.
- Ayuda → Acerca de Trigonum explica para qué está hecho el programa y en qué distros está, y
  su enlace principal es el repositorio en git.local.net.ar. En la pestaña Créditos están los enlaces
  a los binarios para cada distro (openSUSE repo) y a GitHub para issues y PRs, y el autor figura
  como "nachopro, desde Argentina".
- `build-aux/bump-version.sh` solo funciona en la rama `release/<versión nueva>`.

### Corregido

- Regenerar `po/trigonum.pot` con meson ya no borra la versión del encabezado
  (`Project-Id-Version`), que el CI exige: ahora se le pasa `--package-version` a xgettext.
- El PKGBUILD ya no genera un paquete `trigonum-debug` vacío (`options=(!debug)`): el build
  en release no incluye símbolos de depuración.
- El CI ya no usa la caché de Actions: Forgejo la separa por PR, así que cada PR nuevo
  arrancaba vacío. Ahora las imágenes `trigonum-ci/*` traen los crates del `Cargo.lock`, y la
  de Arch también las dependencias de clippy ya compiladas. `dist-tarball.sh` usa los crates
  que ya están y solo baja de internet lo que falta.
- Debian 13 en OBS: se agrega `packaging/obs/prjconf` con `Prefer: libjpeg62-turbo-dev`.
  `libjpeg-dev` lo proveen dos paquetes y OBS no elegía solo, así que marcaba el build
  como irresolvable.

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
