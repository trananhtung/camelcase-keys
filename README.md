# camelcase-keys

[![All Contributors](https://img.shields.io/badge/all_contributors-1-orange.svg?style=flat-square)](#contributors-)

[![crates.io](https://img.shields.io/crates/v/camelcase-keys.svg)](https://crates.io/crates/camelcase-keys)
[![docs.rs](https://docs.rs/camelcase-keys/badge.svg)](https://docs.rs/camelcase-keys)
[![CI](https://github.com/trananhtung/camelcase-keys/actions/workflows/ci.yml/badge.svg)](https://github.com/trananhtung/camelcase-keys/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/camelcase-keys.svg)](#license)

**Convert the keys of a JSON object to `camelCase`.**

`{ "foo_bar": 1 }` → `{ "fooBar": 1 }`. A faithful Rust port of the widely-used
[`camelcase-keys`](https://www.npmjs.com/package/camelcase-keys) npm package, operating on
[`serde_json::Value`] and built on the [`camelcase`](https://crates.io/crates/camelcase)
crate.

- `deep`, `pascal_case`, `preserve_consecutive_uppercase`, `exclude`, and `stop_paths` options
- Numeric-looking keys (`"0"`, `"42"`, `"3.14"`, `"1e3"`, …) are preserved
- Differential-tested against the reference `camelcase-keys` implementation (120k cases)

## Install

```toml
[dependencies]
camelcase-keys = "0.1"
serde_json = "1"
```

## Usage

```rust
use serde_json::json;
use camelcase_keys::{camelcase_keys, camelcase_keys_with, Options};

assert_eq!(
    camelcase_keys(&json!({ "foo_bar": 1, "baz-qux": 2 })),
    json!({ "fooBar": 1, "bazQux": 2 })
);

// Recurse into nested objects and arrays:
assert_eq!(
    camelcase_keys_with(&json!({ "foo_bar": { "nested_key": 1 } }), &Options::new().deep(true)),
    json!({ "fooBar": { "nestedKey": 1 } })
);

// Exclude keys, stop at paths, or produce PascalCase:
let options = Options::new().deep(true).exclude(["api_key"]).stop_paths(["a_b"]);
let _ = camelcase_keys_with(&json!({ "api_key": "x", "a_b": { "c_d": 1 } }), &options);
```

## Notes

- The `exclude` and `stop_paths` options compare against the **original** keys. `stop_paths`
  uses `.`-joined original keys.
- Built-in caching aside, this port is deterministic: unlike the npm package — whose internal
  cache key omits `preserveConsecutiveUppercase`, so mixing that option for the same key in a
  long-lived process can return stale results — this crate always reflects the options you
  pass.

## Contributors ✨

This project follows the [all-contributors](https://github.com/all-contributors/all-contributors) specification. Contributions of any kind are welcome — code, docs, bug reports, ideas, reviews! See the [emoji key](https://allcontributors.org/docs/en/emoji-key) for how each contribution is recognized, and open a PR or issue to get involved.

Thanks goes to these wonderful people:

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/trananhtung"><img src="https://avatars.githubusercontent.com/u/30992229?v=4?s=100" width="100px;" alt="Tung Tran"/><br /><sub><b>Tung Tran</b></sub></a><br /><a href="https://github.com/trananhtung/./commits?author=trananhtung" title="Code">💻</a> <a href="#maintenance-trananhtung" title="Maintenance">🚧</a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
