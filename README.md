# Trigonum

*Español · [English](README.en.md)*

> *trigonum*: «triángulo» en latín. Un archivo STL no es más que una lista de triángulos.

Visor rápido y liviano de modelos 3D en formato STL para Linux, hecho con GTK 4 y OpenGL.

El repositorio original está en [git.local.net.ar/nachopro/trigonum](https://git.local.net.ar/nachopro/trigonum);
[GitHub](https://github.com/devedetti/trigonum) es un mirror. Los issues y los PR van en GitHub.

## Por qué lo hice

Hace unas semanas que vengo diseñando modelos 3D en OpenSCAD que después un vecino
me imprime. Cada vez que exportaba un STL y quería ver si había quedado bien, tenía
que abrirlo en alguna página web: lenta, pesada, llena de publicidad, con el modelo
trabándose cada vez que lo quería girar. Para algo que hacés una y otra vez en el día,
es un martirio.

Así que me hice mi propio visor. Ahora abro el archivo y aparece al toque, lo giro
y anda como un charango, y cuando guardo en OpenSCAD se actualiza solo. Corre a la velocidad
de la luz y soy muy feliz.

Detrás de todo esto hay una convicción: la web es para páginas, no para programas.
Está bárbara para el homebanking o para leer un blog, pero cuando tenés que laburar
en serio, JavaScript no es opción: todo se pone lento, aparecen los lags y encima te
tenés que bancar los anuncios y la basura que viene de yapa. Un programa de escritorio
hecho como la gente arranca rápido, usa lo que tiene que usar y no te pide nada a cambio.

Que vuelvan las apps de escritorio.

## Características

- Archivos STL binarios y ASCII, incluso modelos con millones de triángulos
- Dimensiones, cantidad de triángulos, volumen y superficie
- Malla de alambre superpuesta
- Recarga automática cuando el archivo cambia en el disco (ideal para trabajar con OpenSCAD o exportaciones de CAD)
- Arrastrar y soltar, y apertura desde el gestor de archivos
- Idiomas: español, portugués de Brasil e inglés

## Controles

| Acción | Mouse / tecla |
|---|---|
| Orbitar | Botón izquierdo + arrastrar |
| Desplazar | Botón central o derecho + arrastrar, o Mayús + botón izquierdo |
| Zoom | Rueda del mouse |
| Encuadrar el modelo | Doble clic o `F` |
| Malla de alambre | `W` |
| Panel de información | `I` o `F9` |
| Abrir / Recargar | `Ctrl+O` / `Ctrl+R` o `F5` |

## Instalación

- **Arch Linux:** [`trigonum`](https://aur.archlinux.org/packages/trigonum) en el AUR.
- **Debian 13, Ubuntu 26.04, Fedora y openSUSE Tumbleweed:** repositorios en el
  [Open Build Service](https://software.opensuse.org/download.html?project=home%3Anachopro&package=trigonum), con las instrucciones para agregarlos.

## Compilación

Requisitos: Rust ≥ 1.85 (cargo), Meson ≥ 1.1, GTK ≥ 4.12 y gettext.

```sh
meson setup build --buildtype=release
meson compile -C build
meson test -C build
sudo meson install -C build
```

Para desarrollo también alcanza con `cargo run -- modelo.stl` (la interfaz queda en
inglés salvo que las traducciones estén instaladas). Para probar las traducciones
sin instalar:

```sh
meson compile -C build
TRIGONUM_LOCALEDIR=build/po LANGUAGE=es build/trigonum modelo.stl
```

## Paquetes y CI

Los archivos de empaquetado están en `debian/`, `packaging/rpm/trigonum.spec` y
`packaging/arch/PKGBUILD`. El tarball de cada versión incluye los crates en `vendor/`,
para que Debian, Ubuntu, Fedora y openSUSE puedan compilar sin red.

Cada PR en Forgejo compila, testea e instala el paquete real en Debian 13, Ubuntu 26.04,
Arch, Fedora y openSUSE Tumbleweed (`.forgejo/workflows/ci.yml`). Para correr lo mismo
localmente con Docker:

```sh
build-aux/ci/run-local.sh all        # o: debian, ubuntu, arch, fedora, opensuse
```

Los paquetes quedan en `ci-out/`. La versión mínima de Rust es 1.85, la de Debian 13:
clippy avisa si se usa algo más nuevo.

El CI usa imágenes propias (`trigonum-ci/debian`, `ubuntu`, `arch`, `fedora` y `opensuse`)
con todo preinstalado. Se construyen en el servidor del runner con
`build-aux/ci/build-images.sh`, y `build-aux/ci/host/` trae un timer de systemd que las
reconstruye cada semana y limpia lo que dejan los jobs.

## Traducciones

Los textos se extraen con gettext (carpeta `po/`). Para agregar un idioma:

1. Agregar el código del idioma en `po/LINGUAS`.
2. Ejecutar `meson compile -C build trigonum-pot` para actualizar la plantilla.
3. Ejecutar `msginit -i po/trigonum.pot -l <código> -o po/<código>.po`, traducir y enviar un pull request.

Después de cambiar textos en el código: `meson compile -C build trigonum-update-po`.

## Publicar una versión nueva

1. Anotar los cambios en la sección `[Unreleased]` de `CHANGELOG.md`.
2. Ejecutar `build-aux/bump-version.sh X.Y.Z` en el mismo PR. Cambia la versión en todos
   los archivos que la repiten y cierra la sección del CHANGELOG.
3. Mergear el PR a `main`. El workflow `release.yml` crea el tag `vX.Y.Z`, sube el paquete
   fuente al Open Build Service y actualiza el AUR.

Un merge sin cambio de versión no publica nada.

## Licencia

GPL-3.0-or-later. Ver [LICENSE](LICENSE).
