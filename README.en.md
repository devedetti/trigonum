# Trigonum

*[Español](README.md) · English*

> *trigonum*: Latin for “triangle”. An STL file is nothing but a list of triangles.

A fast, lightweight viewer for STL 3D models on Linux, built with GTK 4 and OpenGL.

The main repository lives at [git.local.net.ar/nachopro/trigonum](https://git.local.net.ar/nachopro/trigonum);
[GitHub](https://github.com/devedetti/trigonum) is a mirror. Issues and pull requests go on GitHub.

## Why I made this

For a few weeks now I've been designing 3D models in OpenSCAD, which a neighbour then
prints for me. Every time I exported an STL and wanted to check it had come out right,
I had to open it in some website: slow, heavy, full of ads, with the model stuttering
every time I tried to rotate it. For something you do over and over during the day,
it was torture.

So I made my own viewer. Now I open the file and it's there instantly, it rotates
smoothly, and when I save in OpenSCAD it reloads by itself. It runs at the speed of
light and I'm very happy.

Behind all this there's a conviction: the web is for pages, not for programs. It's
great for online banking or reading a blog, but when you have real work to do,
JavaScript isn't an option: everything gets slow, the lag kicks in, and on top of that
you have to put up with the ads and all the junk that comes along for free. A desktop
program done right starts fast, uses what it needs and asks nothing in return.

Bring back desktop apps.


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

## Installing

- **Arch Linux:** [`trigonum`](https://aur.archlinux.org/packages/trigonum) on the AUR.
- **Debian 13, Ubuntu 26.04, Fedora and openSUSE Tumbleweed:** repositories on the
  [Open Build Service](https://software.opensuse.org/download.html?project=home%3Anachopro&package=trigonum), with instructions to add them.

## Building

Requirements: Rust ≥ 1.85 (cargo), Meson ≥ 1.1, GTK ≥ 4.12, gettext.

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

## Packages and CI

Packaging lives in `debian/`, `packaging/rpm/trigonum.spec` and `packaging/arch/PKGBUILD`.
Each release tarball ships the crates in `vendor/`, so Debian, Ubuntu, Fedora and openSUSE
can build offline.

Every PR on Forgejo builds, tests and installs the real package on Debian 13, Ubuntu 26.04,
Arch, Fedora and openSUSE Tumbleweed (`.forgejo/workflows/ci.yml`). To run the same locally
with Docker:

```sh
build-aux/ci/run-local.sh all        # or: debian, ubuntu, arch, fedora, opensuse
```

Packages end up in `ci-out/`. The minimum Rust version is 1.85, the one in Debian 13;
clippy warns if anything newer is used.

CI uses its own images (`trigonum-ci/debian`, `ubuntu`, `arch`, `fedora` and `opensuse`)
with everything preinstalled. They are built on the runner host with
`build-aux/ci/build-images.sh`, and `build-aux/ci/host/` ships a systemd timer that rebuilds
them weekly and cleans up what jobs leave behind.

## Translations

Strings are extracted with gettext (`po/`). To add a language:

1. Add the language code to `po/LINGUAS`.
2. `meson compile -C build trigonum-pot` to refresh the template.
3. `msginit -i po/trigonum.pot -l <code> -o po/<code>.po`, translate, and open a pull request.

After changing strings in the code: `meson compile -C build trigonum-update-po`.

## Releasing a new version

1. Record the changes under `[Unreleased]` in `CHANGELOG.md` (written in Spanish).
2. Run `build-aux/bump-version.sh X.Y.Z` in the same PR. It updates the version in every file
   that repeats it and closes the CHANGELOG section.
3. Merge the PR into `main`. The `release.yml` workflow creates the `vX.Y.Z` tag, uploads the
   source package to the Open Build Service and updates the AUR.

A merge without a version change publishes nothing.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
