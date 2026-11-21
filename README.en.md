# Trigonum

*[Español](README.md) · English*

> *trigonum*: Latin for “triangle”. An STL file is nothing but a list of triangles.

A fast, lightweight viewer for STL 3D models on Linux, with a classic desktop
interface (menu bar, toolbar, status bar). Built with GTK 4 and OpenGL.

The main repository lives at [git.local.net.ar/nachopro/trigonum](https://git.local.net.ar/nachopro/trigonum);
[GitHub](https://github.com/devedetti/trigonum) is a mirror. Issues and pull requests go on GitHub.


## Features

- Binary and ASCII STL, including models with millions of triangles
- Dimensions, triangle count, volume and surface area
- Wireframe overlay
- Automatic reload when the file changes on disk (handy with OpenSCAD and CAD exports)
- Drag and drop, open from the file manager
- Translations: English, Spanish, Brazilian Portuguese

## Controls

| Action | Mouse / key |
|---|---|
| Orbit | Left button + drag |
| Pan | Middle or right button + drag, or Shift + left button |
| Zoom | Scroll wheel |
| Fit model | Double click or `F` |
| Wireframe | `W` |
| Information panel | `I` or `F9` |
| Open / Reload | `Ctrl+O` / `Ctrl+R` or `F5` |

## Building

Requirements: Rust ≥ 1.88 (cargo), Meson ≥ 1.1, GTK ≥ 4.12, gettext.

```sh
meson setup build --buildtype=release
meson compile -C build
meson test -C build
sudo meson install -C build
```

For development, plain `cargo run -- model.stl` also works (English UI unless the
translations are installed). To test translations without installing:

```sh
meson compile -C build
TRIGONUM_LOCALEDIR=build/po LANGUAGE=es build/trigonum model.stl
```

## Arch Linux package

`packaging/arch/PKGBUILD` follows the Arch Rust packaging guidelines.

Test locally, using your working copy instead of the published tag:

```sh
build-aux/local-tarball.sh
cd packaging/arch
makepkg -si -p PKGBUILD.local
```

Publishing to AUR:

1. Tag the release and push it to Forgejo: `git tag v0.1.1 && git push --tags`.
2. Wait for the mirror to copy it to GitHub, since the PKGBUILD downloads the tarball from there.
3. In `packaging/arch`: `updpkgsums`, then `makepkg --printsrcinfo > .SRCINFO`.
4. Clone `ssh://aur@aur.archlinux.org/trigonum.git`, copy `PKGBUILD` and `.SRCINFO`, commit and push.

## Translations

Strings are extracted with gettext (`po/`). To add a language:

1. Add the language code to `po/LINGUAS`.
2. `meson compile -C build trigonum-pot` to refresh the template.
3. `msginit -i po/trigonum.pot -l <code> -o po/<code>.po`, translate, and open a pull request.

After changing strings in the code: `meson compile -C build trigonum-update-po`.

## Releasing a new version

Bump the version in `Cargo.toml` **and** `meson.build` (the build fails if they differ),
add a `<release>` entry in `data/trigonum.metainfo.xml.in.in`, bump `pkgver` in the PKGBUILD,
and record the changes in `CHANGELOG.md` (written in Spanish).

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
