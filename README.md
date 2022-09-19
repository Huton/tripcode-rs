# Tripcode
(c) 2016-2022 Huton & Fredrick Brennan

[![Build Status](https://travis-ci.org/Huton/tripcode-rs.svg?branch=master)](https://travis-ci.org/Huton/tripcode-rs)
[![Current Version](http://meritbadge.herokuapp.com/tripcode)](https://crates.io/crates/tripcode)

[Documentation](https://docs.rs/tripcode/)

[README日本語版](README_ja.md)

A Rust library for generating tripcodes on imageboards and textboards.

## Compatibility

This crate is compatible with tripcodes of the following formats:

* 4chan's normal (non-secure) tripcode
* 2channel's tripcodes:
    * 10-character tripcode (10桁トリップ)
    * _Nama key_ tripcode (生キートリップ)
    * 12-character tripcode (12桁トリップ)
* _2ch.sc_'s tripcodes:
    * 15-character tripcode (15桁トリップ)
    * Katakana tripcode (カタカナトリップ)

## Usage

To use this crate, add the following to your project's `Cargo.toml`:

```toml
[dependencies]
tripcode = "0.2"
```

and this to your crate root:

```rust
extern crate tripcode;
```

## Examples

```rust
use tripcode::*;

let mut tripcode;

// 4chan's tripcode.
tripcode = Fourchan::generate(&"password");
assert_eq!("ozOtJW9BFA", &tripcode);

// 2channel (Monazilla)'s tripcode. This function automatically selects the proper hashing algorithm.
tripcode = Mona::generate(&"7 bytes");
assert_eq!("W/RvZlE2K.", &tripcode);
tripcode = Mona::generate(&"twelve bytes");
assert_eq!("t+lnR7LBqNQY", &tripcode);
tripcode = Mona::generate(&"#1145145554560721..");
assert_eq!("14cvFmVHg2", &tripcode);

// 2channel's 10-character tripcode (10桁トリップ).
tripcode = Mona10::generate(&"password longer than 12 bytes");
assert_eq!("ozOtJW9BFA", &tripcode);

// 2channel's nama key tripcode (生キートリップ).
// This generator is failable so we use `try_generate()` method, which yields an `Option<String>`.
tripcode = MonaRaw::try_generate(&"#0123456789ABCDEF./").unwrap();
assert_eq!(&"IP9Lda5FPc", &tripcode);

// 2channel's 12-character tripcode (12桁トリップ).
tripcode = Mona12::generate(&"<12 bytes");
assert_eq!("/9L00Vb1PBcb", &tripcode);
```

## `tripcode` command

This crate also provides a simple command line utility for generating tripcodes.

To install the command, run this in your shell:

```bash
cargo install tripcode
```

The command can take passwords either from arguments:

```bash
$ tripcode a b c
ZnBI2EKkq.
taAZ7oPCCM
wG1CV58ydQ
```

or from stdin (separated by newlines):

```bash
$ echo -e 'd\ne\nf' | tripcode -f
taZqHR8ods
xKvzozvsSk
bb6OCCHf8E
```

The command works with non-UTF-8 encodings as well:

```bash
$ echo トリップ | iconv -t sjis | tripcode -f
XSSH/ryx32
```

The command defaults to generate 4chan's tripcodes.
You can generate 2channel's tripcodes by using `--type=2ch` option.

# Internationalization

i18n is provided via `gettext`, _optionally_.

To build with it:

```bash
cd bin
cargo i18n
cargo build --release --features=i18n
```

Note: Requires [cargo-i18n](https://github.com/MFEK/cargo-i18n/) with [`9e86a65`](https://github.com/kellpossible/cargo-i18n/pull/93/commits/9e86a65e8bba8846c669953f634d617066695002) applied.

It increases the size of the binary not too much:

```bash
$ cargo build --release
…
$ ls -alh target/release/tripcode-cli
-rwxrwxr-x 2 fred fred 4.0M Sep 19 13:00 target/release/tripcode-cli
$ cargo build --release --features=i18n
…
$ ls -alh target/release/tripcode-cli
-rwxrwxr-x 2 fred fred 6.4M Sep 19 12:52 target/release/tripcode-cli
```

This can be brought down significantly with `--profile=release-lto`:

```bash
$ cargo build --profile=release-lto
…
$ ls -alh target/release-lto/tripcode-cli
-rwxrwxr-x 2 fred fred 441K Sep 19 13:03 target/release-lto/tripcode-cli
$ cargo build --profile=release-lto --features=i18n
…
$ ls -alh target/release-lto/tripcode-cli
-rwxrwxr-x 2 fred fred 2.3M Sep 19 13:02 target/release-lto/tripcode-cli
```
