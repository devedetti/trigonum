# SPDX-License-Identifier: GPL-3.0-or-later
%global debug_package %{nil}
%global app_id ar.net.local.Trigonum

Name:           trigonum
Version:        0.1.2
Release:        1%{?dist}
Summary:        Lightweight viewer for STL 3D models
License:        GPL-3.0-or-later
URL:            https://github.com/devedetti/trigonum
Source0:        %{name}_%{version}.orig.tar.xz

BuildRequires:  meson >= 1.1
BuildRequires:  cargo
BuildRequires:  rust >= 1.85
BuildRequires:  pkgconfig(gtk4) >= 4.12
BuildRequires:  desktop-file-utils
%if 0%{?suse_version}
BuildRequires:  gettext-tools
BuildRequires:  AppStream
Requires:       libepoxy0
%else
BuildRequires:  gettext
BuildRequires:  appstream
Requires:       libepoxy
%endif

%description
Trigonum is a fast, lightweight viewer for STL files, the format used by 3D
printers and most CAD programs. It opens binary and ASCII files with millions
of triangles, shows dimensions, volume and surface area, and reloads the model
automatically when the file changes on disk. It has a classic desktop
interface with a menu bar, toolbar and status bar.

%prep
%autosetup -n %{name}-%{version}
mkdir -p .cargo-home
cat > .cargo-home/config.toml <<CARGO
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "$PWD/vendor"
CARGO

%build
export CARGO_HOME="$PWD/.cargo-home" CARGO_NET_OFFLINE=true
%meson
%meson_build

%install
export CARGO_HOME="$PWD/.cargo-home" CARGO_NET_OFFLINE=true
%meson_install
%find_lang %{name}

%check
export CARGO_HOME="$PWD/.cargo-home" CARGO_NET_OFFLINE=true
%meson_test

%files -f %{name}.lang
%license LICENSE
%doc README.md README.en.md README.pt.md CHANGELOG.md
%{_bindir}/%{name}
%{_datadir}/applications/%{app_id}.desktop
%{_datadir}/metainfo/%{app_id}.metainfo.xml
%{_datadir}/icons/hicolor/scalable/apps/%{app_id}.svg
