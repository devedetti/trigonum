# Trigonum

*Español · [English](README.en.md) · [Português](README.pt.md)*

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
- Pestañas: varios modelos abiertos a la vez, cada uno con su vista
- Malla de alambre superpuesta
- Recarga automática cuando el archivo cambia en el disco (ideal para trabajar con OpenSCAD o exportaciones de CAD)
- Arrastrar y soltar, y apertura desde el gestor de archivos
- Idiomas: español, portugués e inglés

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
| Cerrar pestaña | `Ctrl+W` o clic del medio en la pestaña |
| Pestaña siguiente / anterior | `Ctrl+Tab` / `Ctrl+Mayús+Tab`, o `Ctrl+AvPág` / `Ctrl+RePág` |
| Ir a la pestaña 1…8 / a la última | `Alt+1`…`Alt+8` / `Alt+9` |
| Salir | `Ctrl+Q` |

## Instalación

La forma más simple para Debian, Ubuntu, Fedora y openSUSE es la
[página de descarga](https://software.opensuse.org/download.html?project=home%3Anachopro&package=trigonum): elegís tu distro y te muestra cómo agregar el repositorio,
o te ofrece la instalación en un clic. Abajo están los mismos pasos para cada distro.

### Arch Linux y derivadas

Está en el AUR como [`trigonum`](https://aur.archlinux.org/packages/trigonum):

```sh
yay -S trigonum        # o: paru -S trigonum
```

Sin un ayudante de AUR:

```sh
git clone https://aur.archlinux.org/trigonum.git
cd trigonum
makepkg -si
```

### Debian 13 (trixie)

```sh
sudo install -d /etc/apt/keyrings
curl -fsSL https://download.opensuse.org/repositories/home:/nachopro/Debian_13/Release.key | sudo gpg --dearmor -o /etc/apt/keyrings/trigonum.gpg
echo 'deb [signed-by=/etc/apt/keyrings/trigonum.gpg] https://download.opensuse.org/repositories/home:/nachopro/Debian_13/ /' | sudo tee /etc/apt/sources.list.d/trigonum.list
sudo apt update
sudo apt install trigonum
```

### Ubuntu 26.04

```sh
sudo install -d /etc/apt/keyrings
curl -fsSL https://download.opensuse.org/repositories/home:/nachopro/xUbuntu_26.04/Release.key | sudo gpg --dearmor -o /etc/apt/keyrings/trigonum.gpg
echo 'deb [signed-by=/etc/apt/keyrings/trigonum.gpg] https://download.opensuse.org/repositories/home:/nachopro/xUbuntu_26.04/ /' | sudo tee /etc/apt/sources.list.d/trigonum.list
sudo apt update
sudo apt install trigonum
```

### Fedora 44 y 45

```sh
sudo dnf config-manager addrepo --from-repofile=https://download.opensuse.org/repositories/home:/nachopro/Fedora_$(rpm -E %fedora)/home:nachopro.repo
sudo dnf install trigonum
```

### openSUSE Tumbleweed

```sh
sudo zypper addrepo https://download.opensuse.org/repositories/home:/nachopro/openSUSE_Tumbleweed/home:nachopro.repo
sudo zypper refresh
sudo zypper install trigonum
```

`zypper refresh` pide confirmar la clave del repositorio: respondé `a` (aceptar siempre).

Con el repositorio agregado, las versiones nuevas llegan con las actualizaciones normales del sistema.
Los paquetes se compilan en el Open Build Service, en el proyecto `home:nachopro`.

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

El CI completo (`.forgejo/workflows/ci.yml`) compila, testea e instala el paquete real en
Debian 13, Ubuntu 26.04, Arch, Fedora y openSUSE Tumbleweed. Corre en el PR de cada RC y a
demanda (ver [CONTRIBUTING.md](CONTRIBUTING.md)). Para correr lo mismo localmente con Docker:

```sh
build-aux/ci/run-local.sh all        # o: debian, ubuntu, arch, fedora, opensuse
```

Los paquetes quedan en `ci-out/`. La versión mínima de Rust es 1.85, la de Debian 13:
clippy avisa si se usa algo más nuevo.

El CI usa imágenes propias (`trigonum-ci/debian`, `ubuntu`, `arch`, `fedora` y `opensuse`)
con todo preinstalado: herramientas, dependencias, los crates del `Cargo.lock` y, en la de
Arch, las dependencias de clippy ya compiladas. Se construyen en el servidor del runner con
`build-aux/ci/build-images.sh`, y `build-aux/ci/host/` trae un timer de systemd que las
reconstruye cada semana y limpia lo que dejan los jobs. Si cambiás las dependencias, el CI
baja lo nuevo hasta la próxima reconstrucción; para no esperar, corré `build-images.sh` a mano.

## Traducciones

Los textos se extraen con gettext (carpeta `po/`). Para agregar un idioma:

1. Agregar el código del idioma en `po/LINGUAS`.
2. Ejecutar `meson compile -C build trigonum-pot` para actualizar la plantilla.
3. Ejecutar `msginit -i po/trigonum.pot -l <código> -o po/<código>.po`, traducir y enviar un pull request.

Después de cambiar textos en el código: `meson compile -C build trigonum-update-po`.

## Contribuir y publicar versiones

Las features van por PR a una rama temporal `next`, que al sacar una versión se renombra a
`release/X.Y.Z`. Su PR hacia `main` es el RC: corre el CI completo en las 5 distros y, al
mergearlo, se publica. El detalle está en [CONTRIBUTING.md](CONTRIBUTING.md).

## Licencia

GPL-3.0-or-later. Ver [LICENSE](LICENSE).
