# SPDX-License-Identifier: GPL-3.0-or-later
project(
  'trigonum',
  version: '0.1.1', # mantener igual que en Cargo.toml
  license: 'GPL-3.0-or-later',
  meson_version: '>= 1.1',
)

# --- Identidad del proyecto: cambiar acá si el repo termina en otro lado. ---
app_id = 'ar.net.local.Trigonum'
# Repo original en Forgejo; GitHub es un espejo (push mirror) y es la cara pública:
# web, issues y el tarball que usa el AUR salen de ahí.
website = 'https://github.com/devedetti/trigonum'
repository = 'https://git.local.net.ar/nachopro/trigonum'

gettext_package = meson.project_name()
localedir = get_option('prefix') / get_option('localedir')

i18n = import('i18n')
gnome = import('gnome')

dependency('gtk4', version: '>= 4.12')
cargo = find_program('cargo')

cargo_version = run_command(
  find_program('grep'), '-m1', '^version', meson.project_source_root() / 'Cargo.toml',
  check: true,
).stdout().strip()
if not cargo_version.contains('"' + meson.project_version() + '"')
  error('La versión de meson.build (' + meson.project_version() + ') no coincide con Cargo.toml: ' + cargo_version)
endif

subdir('data')
subdir('po')
subdir('src')

gnome.post_install(
  gtk_update_icon_cache: true,
  update_desktop_database: true,
)
