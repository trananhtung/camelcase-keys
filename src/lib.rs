//! # camelcase-keys — convert JSON object keys to `camelCase`
//!
//! Recursively convert the keys of a [`serde_json::Value`] to `camelCase`. A faithful Rust
//! port of the widely-used
//! [`camelcase-keys`](https://www.npmjs.com/package/camelcase-keys) npm package, built on the
//! [`camelcase`](https://crates.io/crates/camelcase) crate.
//!
//! ```
//! use serde_json::json;
//! use camelcase_keys::{camelcase_keys, camelcase_keys_with, Options};
//!
//! assert_eq!(
//!     camelcase_keys(&json!({ "foo_bar": 1, "baz-qux": 2 })),
//!     json!({ "fooBar": 1, "bazQux": 2 })
//! );
//!
//! // Recurse into nested objects/arrays with `deep`:
//! assert_eq!(
//!     camelcase_keys_with(&json!({ "foo_bar": { "nested_key": 1 } }), &Options::new().deep(true)),
//!     json!({ "fooBar": { "nestedKey": 1 } })
//! );
//! ```
//!
//! Numeric-looking keys (`"0"`, `"42"`, `"3.14"`, …) are preserved, and the `pascal_case`,
//! `preserve_consecutive_uppercase`, `exclude`, and `stop_paths` options match the reference.

#![forbid(unsafe_code)]
#![doc(html_root_url = "https://docs.rs/camelcase-keys/0.1.0")]

use camelcase::{camel_case_with, Options as CamelOptions};
use serde_json::{Map, Value};

// Compile-test the README's examples as part of `cargo test`.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

/// Options controlling [`camelcase_keys_with`].
#[derive(Debug, Clone, Default)]
pub struct Options {
    deep: bool,
    pascal_case: bool,
    preserve_consecutive_uppercase: bool,
    exclude: Vec<String>,
    stop_paths: Vec<String>,
}

impl Options {
    /// Default options (shallow, `camelCase`).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Recurse into nested objects and arrays.
    #[must_use]
    pub fn deep(mut self, value: bool) -> Self {
        self.deep = value;
        self
    }

    /// Produce `PascalCase` keys instead of `camelCase`.
    #[must_use]
    pub fn pascal_case(mut self, value: bool) -> Self {
        self.pascal_case = value;
        self
    }

    /// Preserve consecutive uppercase letters (`HTTP_status` → `HTTPStatus`).
    #[must_use]
    pub fn preserve_consecutive_uppercase(mut self, value: bool) -> Self {
        self.preserve_consecutive_uppercase = value;
        self
    }

    /// Keys to leave untouched (compared against the original key).
    #[must_use]
    pub fn exclude<I, S>(mut self, keys: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.exclude = keys.into_iter().map(Into::into).collect();
        self
    }

    /// Dot-paths (built from the original keys) at which to stop recursing.
    #[must_use]
    pub fn stop_paths<I, S>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.stop_paths = paths.into_iter().map(Into::into).collect();
        self
    }

    fn camel_options(&self) -> CamelOptions {
        CamelOptions::new()
            .pascal_case(self.pascal_case)
            .preserve_consecutive_uppercase(self.preserve_consecutive_uppercase)
    }
}

/// Convert the keys of `value` to `camelCase` using the default options (shallow).
///
/// ```
/// # use serde_json::json;
/// # use camelcase_keys::camelcase_keys;
/// assert_eq!(camelcase_keys(&json!({ "a_b": 1 })), json!({ "aB": 1 }));
/// ```
#[must_use]
pub fn camelcase_keys(value: &Value) -> Value {
    camelcase_keys_with(value, &Options::new())
}

/// Convert the keys of `value` to `camelCase` with the given [`Options`].
#[must_use]
pub fn camelcase_keys_with(value: &Value, options: &Options) -> Value {
    transform(value, options, None)
}

/// A value `camelcase-keys` recurses into (an object or array; JSON has no other containers).
fn is_recursable(value: &Value) -> bool {
    value.is_object() || value.is_array()
}

fn transform(value: &Value, options: &Options, parent_path: Option<&str>) -> Value {
    match value {
        Value::Array(array) => Value::Array(
            array
                .iter()
                .map(|item| {
                    if is_recursable(item) {
                        transform(item, options, parent_path)
                    } else {
                        item.clone()
                    }
                })
                .collect(),
        ),
        Value::Object(map) => {
            let mut result = Map::new();
            for (key, val) in map {
                let mut new_value = val.clone();

                if options.deep && is_recursable(val) {
                    let path = match parent_path {
                        Some(parent) => format!("{parent}.{key}"),
                        None => key.clone(),
                    };
                    if !options.stop_paths.iter().any(|stop| stop == &path) {
                        new_value = match val {
                            Value::Array(items) => Value::Array(
                                items
                                    .iter()
                                    .map(|item| {
                                        if is_recursable(item) {
                                            transform(item, options, Some(&path))
                                        } else {
                                            item.clone()
                                        }
                                    })
                                    .collect(),
                            ),
                            _ => transform(val, options, Some(&path)),
                        };
                    }
                }

                let new_key = if is_numeric_key(key) || options.exclude.iter().any(|e| e == key) {
                    key.clone()
                } else {
                    camel_case_with(key, options.camel_options())
                };
                result.insert(new_key, new_value);
            }
            Value::Object(result)
        }
        other => other.clone(),
    }
}

/// Characters JavaScript's `String.prototype.trim` (and `Number`) treat as whitespace.
fn is_js_whitespace(c: char) -> bool {
    matches!(
        c,
        '\u{0009}'
            | '\u{000A}'
            | '\u{000B}'
            | '\u{000C}'
            | '\u{000D}'
            | '\u{0020}'
            | '\u{00A0}'
            | '\u{1680}'
            | '\u{2000}'
            ..='\u{200A}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202F}'
                | '\u{205F}'
                | '\u{3000}'
                | '\u{FEFF}'
    )
}

/// `key.trim() !== '' && !Number.isNaN(Number(key))` — keys that coerce to a number.
fn is_numeric_key(key: &str) -> bool {
    let trimmed = key.trim_matches(is_js_whitespace);
    !trimmed.is_empty() && number_is_not_nan(trimmed)
}

/// Whether `String(value) === text` coerces to a non-`NaN` `Number` (the string is already
/// trimmed and non-empty).
fn number_is_not_nan(text: &str) -> bool {
    let (had_sign, rest) = match text.as_bytes().first() {
        Some(b'+' | b'-') => (true, &text[1..]),
        _ => (false, text),
    };
    if rest.is_empty() {
        return false;
    }
    if rest == "Infinity" {
        return true;
    }
    // Hex / binary / octal literals do not allow a sign.
    if !had_sign {
        if let Some(digits) = strip_radix_prefix(rest, b'x') {
            return !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_hexdigit());
        }
        if let Some(digits) = strip_radix_prefix(rest, b'b') {
            return !digits.is_empty() && digits.bytes().all(|b| b == b'0' || b == b'1');
        }
        if let Some(digits) = strip_radix_prefix(rest, b'o') {
            return !digits.is_empty() && digits.bytes().all(|b| (b'0'..=b'7').contains(&b));
        }
    }
    is_decimal_literal(rest)
}

/// Strip a `0x` / `0b` / `0o` prefix (case-insensitive `letter`), returning the digits.
fn strip_radix_prefix(s: &str, letter: u8) -> Option<&str> {
    let bytes = s.as_bytes();
    if bytes.len() >= 2 && bytes[0] == b'0' && (bytes[1] == letter || bytes[1] == letter - 32) {
        Some(&s[2..])
    } else {
        None
    }
}

/// A JavaScript unsigned decimal literal: `123`, `123.`, `1.5`, `.5`, `1e3`, `1.5e-3`.
fn is_decimal_literal(s: &str) -> bool {
    let (mantissa, exponent) = match s.find(['e', 'E']) {
        Some(index) => (&s[..index], Some(&s[index + 1..])),
        None => (s, None),
    };

    if let Some(exponent) = exponent {
        let digits = match exponent.as_bytes().first() {
            Some(b'+' | b'-') => &exponent[1..],
            _ => exponent,
        };
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return false;
        }
    }

    let mut dots = 0;
    let mut digits = 0;
    for byte in mantissa.bytes() {
        match byte {
            b'.' => dots += 1,
            b'0'..=b'9' => digits += 1,
            _ => return false,
        }
    }
    dots <= 1 && digits >= 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn shallow() {
        assert_eq!(
            camelcase_keys(&json!({ "foo_bar": 1, "baz-qux": 2 })),
            json!({ "fooBar": 1, "bazQux": 2 })
        );
        // Default is shallow: nested keys untouched.
        assert_eq!(
            camelcase_keys(&json!({ "foo_bar": { "nested_key": 1 } })),
            json!({ "fooBar": { "nested_key": 1 } })
        );
    }

    #[test]
    fn deep_and_arrays() {
        assert_eq!(
            camelcase_keys_with(
                &json!({ "foo_bar": { "nested_key": 1 } }),
                &Options::new().deep(true)
            ),
            json!({ "fooBar": { "nestedKey": 1 } })
        );
        assert_eq!(
            camelcase_keys(&json!([{ "a_b": 1 }, { "c_d": 2 }])),
            json!([{ "aB": 1 }, { "cD": 2 }])
        );
        assert_eq!(
            camelcase_keys_with(
                &json!({ "a_b": [[{ "c_d": 1 }]] }),
                &Options::new().deep(true)
            ),
            json!({ "aB": [[{ "cD": 1 }]] })
        );
    }

    #[test]
    fn numeric_keys_preserved() {
        assert_eq!(
            camelcase_keys(&json!({ "0": 1, "42": 2, "3.14": 3, "foo_bar": 4 })),
            json!({ "0": 1, "42": 2, "3.14": 3, "fooBar": 4 })
        );
        assert!(is_numeric_key("1e3"));
        assert!(is_numeric_key("Infinity"));
        assert!(is_numeric_key("0x1f"));
        assert!(is_numeric_key(".5"));
        assert!(!is_numeric_key("12abc"));
        assert!(!is_numeric_key("1_0"));
        assert!(!is_numeric_key(" "));
    }

    #[test]
    fn options() {
        assert_eq!(
            camelcase_keys_with(
                &json!({ "first_name": 1, "last_name": 2 }),
                &Options::new().exclude(["first_name"])
            ),
            json!({ "first_name": 1, "lastName": 2 })
        );
        assert_eq!(
            camelcase_keys_with(
                &json!({ "a_b": { "c_d": 1 } }),
                &Options::new().deep(true).stop_paths(["a_b"])
            ),
            json!({ "aB": { "c_d": 1 } })
        );
        assert_eq!(
            camelcase_keys_with(&json!({ "foo_bar": 1 }), &Options::new().pascal_case(true)),
            json!({ "FooBar": 1 })
        );
        assert_eq!(
            camelcase_keys_with(
                &json!({ "HTTP_status": 1 }),
                &Options::new().preserve_consecutive_uppercase(true)
            ),
            json!({ "HTTPStatus": 1 })
        );
    }

    #[test]
    fn scalars_passthrough() {
        assert_eq!(camelcase_keys(&json!(42)), json!(42));
        assert_eq!(camelcase_keys(&json!("str")), json!("str"));
        assert_eq!(camelcase_keys(&json!(null)), json!(null));
    }
}
