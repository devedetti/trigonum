// SPDX-License-Identifier: GPL-3.0-or-later
//! Ayudas de traducción: placeholders con nombre y números con separadores del idioma.

use std::fmt::Display;

pub use gettextrs::{gettext, ngettext, pgettext};

/// Reemplaza `{clave}` en una cadena traducida. Con nombres (y no `{}` posicionales)
/// el traductor puede reordenar las partes de la frase.
pub fn fill(template: String, args: &[(&str, &dyn Display)]) -> String {
    let mut out = template;
    for (key, value) in args {
        out = out.replace(&format!("{{{key}}}"), &value.to_string());
    }
    out
}

fn thousands_sep() -> String {
    // TRANSLATORS: thousands separator for large numbers, e.g. "1,234,567".
    pgettext("thousands separator", ",")
}

fn decimal_sep() -> String {
    // TRANSLATORS: decimal separator, e.g. "12.34".
    pgettext("decimal separator", ".")
}

fn group(digits: &str, sep: &str) -> String {
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push_str(sep);
        }
        out.push(c);
    }
    out
}

pub fn int(n: usize) -> String {
    group(&n.to_string(), &thousands_sep())
}

pub fn dec(x: f64, places: usize) -> String {
    let s = format!("{:.*}", places, x.abs());
    let (whole, frac) = s.split_once('.').unwrap_or((&s, ""));
    let sign = if x < 0.0 && s.chars().any(|c| c.is_ascii_digit() && c != '0') { "-" } else { "" };
    let mut out = format!("{sign}{}", group(whole, &thousands_sep()));
    if !frac.is_empty() {
        out.push_str(&decimal_sep());
        out.push_str(frac);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_with_default_separators() {
        assert_eq!(int(1234567), "1,234,567");
        assert_eq!(int(12), "12");
        assert_eq!(dec(1234.5, 2), "1,234.50");
        assert_eq!(dec(-0.001, 2), "0.00");
        assert_eq!(dec(-3.25, 1), "-3.2");
    }

    #[test]
    fn fills_named_placeholders() {
        let s = fill("{b} then {a}".into(), &[("a", &1), ("b", &"x")]);
        assert_eq!(s, "x then 1");
    }
}
