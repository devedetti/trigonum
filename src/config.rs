// SPDX-License-Identifier: GPL-3.0-or-later
//! Valores de build. Meson los inyecta como variables de entorno al llamar a cargo;
//! con `cargo build` a secas se usan los valores por defecto.

use std::path::PathBuf;

const fn or(v: Option<&'static str>, default: &'static str) -> &'static str {
    match v {
        Some(v) => v,
        None => default,
    }
}

pub const APP_ID: &str = or(option_env!("TRIGONUM_APP_ID"), "ar.net.local.Trigonum");
pub const WEBSITE: &str = or(option_env!("TRIGONUM_WEBSITE"), "https://github.com/devedetti/trigonum");
pub const DOWNLOADS: &str =
    "https://software.opensuse.org/download.html?project=home%3Anachopro&package=trigonum";
pub const REPOSITORY: &str =
    or(option_env!("TRIGONUM_REPOSITORY"), "https://git.local.net.ar/nachopro/trigonum");
pub const GETTEXT_PACKAGE: &str = "trigonum";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const LOCALEDIR: &str = or(option_env!("TRIGONUM_LOCALEDIR"), "/usr/share/locale");

/// `TRIGONUM_LOCALEDIR` en tiempo de ejecución permite probar traducciones sin instalar
/// (p. ej. apuntando a `build/po`).
pub fn locale_dir() -> PathBuf {
    std::env::var_os("TRIGONUM_LOCALEDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(LOCALEDIR))
}
