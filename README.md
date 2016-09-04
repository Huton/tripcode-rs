# Tripcode
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
tripcode = "0.1"
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
$ echo 'd\ne\nf' | tripcode -f
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
