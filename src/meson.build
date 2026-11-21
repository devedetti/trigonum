# SPDX-License-Identifier: GPL-3.0-or-later
cargo_target_dir = meson.project_build_root() / 'cargo-target'

cargo_env = environment()
cargo_env.set('TRIGONUM_APP_ID', app_id)
cargo_env.set('TRIGONUM_WEBSITE', website)
cargo_env.set('TRIGONUM_LOCALEDIR', localedir)
cargo_env.set('CARGO_TARGET_DIR', cargo_target_dir)

if get_option('buildtype') == 'debug'
  rust_profile = 'debug'
  profile_args = []
else
  rust_profile = 'release'
  profile_args = ['--release']
endif

manifest = ['--manifest-path', meson.project_source_root() / 'Cargo.toml']

custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  console: true,
  env: cargo_env,
  output: meson.project_name(),
  install: true,
  install_dir: get_option('bindir'),
  command: [
    find_program(meson.project_source_root() / 'build-aux' / 'cargo-build.sh'),
    cargo,
    cargo_target_dir / rust_profile / meson.project_name(),
    '@OUTPUT@',
    'build', manifest, '--locked', profile_args,
  ],
)

test(
  'cargo-test',
  cargo,
  args: ['test', manifest, '--locked', profile_args],
  env: cargo_env,
  timeout: 1800,
)
