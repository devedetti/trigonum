# Trigonum

*[Español](README.md) · English · [Português](README.pt.md)*

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
- Tabs: several models open at once, each with its own view
- Wireframe overlay
- Automatic reload when the file changes on disk (handy with OpenSCAD and CAD exports)
- Drag and drop, open from the file manager
- Translations: English, Spanish, Portuguese

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
| Close tab | `Ctrl+W` or middle click on the tab |
| Next / previous tab | `Ctrl+Tab` / `Ctrl+Shift+Tab`, or `Ctrl+PgDn` / `Ctrl+PgUp` |
| Go to tab 1…8 / last tab | `Alt+1`…`Alt+8` / `Alt+9` |
| Quit | `Ctrl+Q` |

## Installing

The easiest way on Debian, Ubuntu, Fedora and openSUSE is the
[download page](https://software.opensuse.org/download.html?project=home%3Anachopro&package=trigonum): pick your distro and it shows how to add the repository,
or offers a one-click install. The same steps for each distro are below.

### Arch Linux and derivatives

Available on the AUR as [`trigonum`](https://aur.archlinux.org/packages/trigonum):

```sh
yay -S trigonum        # or: paru -S trigonum
```

Without an AUR helper:

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

### Fedora 44 and 45

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

`zypper refresh` asks you to confirm the repository key: answer `a` (always trust).

Once the repository is added, new versions arrive with regular system updates.
Packages are built on the Open Build Service, in the `home:nachopro` project.

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

The full CI (`.forgejo/workflows/ci.yml`) builds, tests and installs the real package on
Debian 13, Ubuntu 26.04, Arch, Fedora and openSUSE Tumbleweed. It runs on every RC pull
request and on demand (see [CONTRIBUTING.en.md](CONTRIBUTING.en.md)). To run the same locally with Docker:

```sh
build-aux/ci/run-local.sh all        # or: debian, ubuntu, arch, fedora, opensuse
```

Packages end up in `ci-out/`. The minimum Rust version is 1.85, the one in Debian 13;
clippy warns if anything newer is used.

CI uses its own images (`trigonum-ci/debian`, `ubuntu`, `arch`, `fedora` and `opensuse`)
with everything preinstalled: tools, dependencies, the crates from `Cargo.lock` and, in the
Arch one, clippy's dependencies already compiled. They are built on the runner host with
`build-aux/ci/build-images.sh`, and `build-aux/ci/host/` ships a systemd timer that rebuilds
them weekly and cleans up what jobs leave behind. If you change dependencies, CI downloads
the new ones until the next rebuild; run `build-images.sh` by hand to skip the wait.

## Translations

Strings are extracted with gettext (`po/`). To add a language:

1. Add the language code to `po/LINGUAS`.
2. `meson compile -C build trigonum-pot` to refresh the template.
3. `msginit -i po/trigonum.pot -l <code> -o po/<code>.po`, translate, and open a pull request.

After changing strings in the code: `meson compile -C build trigonum-update-po`.

## Contributing and releasing

Features go through pull requests into a temporary `next` branch, which is renamed to
`release/X.Y.Z` when a version ships. Its PR into `main` is the RC: it runs the full CI on the
5 distros and, once merged, the version is published. Details are in
[CONTRIBUTING.en.md](CONTRIBUTING.en.md).

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
