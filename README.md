# Trigonum

*Español · [English](README.en.md)*

> *trigonum*: «triángulo» en latín. Un archivo STL no es más que una lista de triángulos.

Visor rápido y liviano de modelos 3D en formato STL para Linux, con una interfaz
de escritorio clásica: barra de menús, barra de herramientas y barra de estado.
Hecho con GTK 4 y OpenGL.

El repositorio original está en [git.local.net.ar/nachopro/trigonum](https://git.local.net.ar/nachopro/trigonum);
[GitHub](https://github.com/devedetti/trigonum) es un espejo. Los issues y los PR van en GitHub.

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

## Compilación

Requisitos: Rust ≥ 1.88 (cargo), Meson ≥ 1.1, GTK ≥ 4.12 y gettext.

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

## Paquete para Arch Linux

`packaging/arch/PKGBUILD` sigue las guías de empaquetado de Rust de Arch.

Para probarlo localmente, con el código de tu copia en vez del tag publicado:

```sh
build-aux/local-tarball.sh
cd packaging/arch
makepkg -si -p PKGBUILD.local
```

Para publicarlo en AUR:

1. Crear el tag de la versión y subirlo a Forgejo: `git tag v0.1.1 && git push --tags`.
2. Esperar a que el espejo lo copie a GitHub, porque el PKGBUILD baja el tarball de ahí.
3. En `packaging/arch`: `updpkgsums` y después `makepkg --printsrcinfo > .SRCINFO`.
4. Clonar `ssh://aur@aur.archlinux.org/trigonum.git`, copiar `PKGBUILD` y `.SRCINFO`, hacer commit y push.

## Traducciones

Los textos se extraen con gettext (carpeta `po/`). Para agregar un idioma:

1. Agregar el código del idioma en `po/LINGUAS`.
2. Ejecutar `meson compile -C build trigonum-pot` para actualizar la plantilla.
3. Ejecutar `msginit -i po/trigonum.pot -l <código> -o po/<código>.po`, traducir y enviar un pull request.

Después de cambiar textos en el código: `meson compile -C build trigonum-update-po`.

## Publicar una versión nueva

Subir la versión en `Cargo.toml` **y** en `meson.build` (el build falla si no coinciden),
agregar una entrada `<release>` en `data/trigonum.metainfo.xml.in.in`, actualizar
`pkgver` en el PKGBUILD y anotar los cambios en `CHANGELOG.md`.

## Licencia

GPL-3.0-or-later. Ver [LICENSE](LICENSE).
