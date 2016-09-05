// Copyright 2016 Huton. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/*!
# Tripcode

A library for generating tripcodes on textboards and imageboards.

## Compatibility

This crate is compatible with tripcodes of the following formats:

* 4chan's normal (non-secure) tripcode
* 2channel's tripcodes:
  * 10-character tripcode (10桁トリップ)
  * *Nama key* tripcode (生キートリップ)
  * 12-character tripcode (12桁トリップ)
* _2ch.sc_'s tripcodes:
  * 15-character tripcode (15桁トリップ)
  * Katakana tripcode (カタカナトリップ)

## Usage

Add `tripcode` to the dependencies in your project's `Cargo.toml`:

```toml
[dependencies]
tripcode = "0.1"
```

and this to your crate root:

```
extern crate tripcode;
```

## Overview

Basic examples:

```
use tripcode::*;

let mut tripcode;

// 4chan's tripcode.
tripcode = Fourchan::generate(&"password");
assert_eq!("ozOtJW9BFA", &tripcode);

// The above method handles HTML escaping.
tripcode = Fourchan::generate(&"&\"");
assert_eq!("ydkX0LqkHM", &tripcode);
tripcode = FourchanNonescaping::generate(&"&amp;&quot;");
assert_eq!("ydkX0LqkHM", &tripcode);

// 2channel (Monazilla)'s tripcode. This method automatically selects the proper hashing algorithm.
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

## Handling non-UTF-8 passwords

Besides `str`, the `generate()` method can take any object that implements `AsRef<[u8]>`
so that it can handle non-UTF-8 passwords.

Here's an example of generating a tripcode from a CP932 (superset of Shift-JIS) encoded password.
This example uses `encoding` crate:

```
extern crate encoding;
extern crate tripcode;

use encoding::{Encoding, EncoderTrap};
use encoding::all::WINDOWS_31J;
use tripcode::*;

fn main() {
    let mut tripcode = String::with_capacity(10);
    let sjis = WINDOWS_31J.encode("トリップ", EncoderTrap::Strict).unwrap();
    let tripcode = Fourchan::generate(&sjis);
    assert_eq!("XSSH/ryx32", &tripcode);
}
```

## Avoiding reallocations

The `append()` method takes a `&mut String` and appends the resulting tripcode to it.
The method does not cause additional heap allocations if the buffer has sufficient capacity to
store the tripcode.

```
use tripcode::*;

// Prepare a buffer
let mut tripcode = String::with_capacity(20);

Fourchan::append(&"tripcode", &mut tripcode);
assert_eq!("3GqYIJ3Obs", &tripcode);
assert_eq!(tripcode.capacity(), 20);

Fourchan::append(&"TRIPCODE", &mut tripcode);
assert_eq!("3GqYIJ3ObsPvHEudHNso", &tripcode);
assert_eq!(tripcode.capacity(), 20); // No allocations have occured!
```

## Writing to streams

The `write()` method takes a mutable reference to a `Write` and writes the resulting tripcode
to it.

```
use std::io::Write;
use tripcode::*;

let mut tripcode = [0u8; 10];

// `&'a mut [u8]` implements `Write`.
Fourchan::write(&"Writing to stream", &mut (&mut tripcode as &mut [u8])).unwrap();
assert_eq!("N5MkEeXGtk", String::from_utf8_lossy(&tripcode));
```
*/

#![warn(missing_docs)]

extern crate crypto;

mod des;
#[macro_use]
mod util;

use crypto::sha1::Sha1;
use crypto::digest::Digest;
use hash::*;
use util::*;
use std::{io, mem};
use std::io::Write;

pub mod hash;

/// Generator for tripcodes on 4chan.
pub struct Fourchan;

/// Same as `Fourchan` and `Mona10` except that it does not escape HTML special characters
/// in passwords.
pub struct FourchanNonescaping;

/// Generator for tripcodes on 2channel.
///
/// The format of resulting tripcodes is determined as follows:
///
/// * If the password is 12 or greater bytes long and:
///     * begins with `'#'` sign -> _Nama key_ tripcode (生キートリップ).
///     * begins with `'$'` sign -> `"???"` (undefined).
///     * else -> 12-character tripcode (12桁トリップ).
/// * else -> 10-character tripcode.
pub struct Mona;

/// Same as `Mona` except that it does not escape HTML special characters in passwords.
pub struct MonaNonescaping;

/// Generator for 2channel's 10-character tripcodes (10桁トリップ).
pub struct Mona10;

pub use FourchanNonescaping as Mona10Nonescaping;

/// Generator for 2channel's 12-character tripcodes.
pub struct Mona12;

/// Same as `Mona12` except that it does not escape HTML special characters in passwords.
pub struct Mona12Nonescaping;

/// Generator for 2channel's `nama key` tripcodes (生キートリップ).
///
/// This generator is failable so only implements `TripcodeGeneratorFailable`.
pub struct MonaRaw;

/// Generator for tripcodes on _2ch.sc_.
///
/// The format of resulting tripcodes is determined as follows:
///
/// * If the password is 12 or greater bytes long and:
///     * begins with `'#'` sign -> _Nama key_ tripcode (生キートリップ).
///     * begins with `'$'` sign and:
///         * the `'$'` sign is followed by a half-width katakana character -> Katakana tripcode (カタカナトリップ).
///         * else -> 15-character tripcode (15桁トリップ).
///     * else -> 12-character tripcode (12桁トリップ).
/// * else -> 10-character tripcode.
///
/// The following is the list of characters to be treated as half-width katakanas above:
///
/// ```text
/// ｡｢｣､･ｦｧｨｩｪｫｬｭｮｯｰｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝﾞﾟ
/// ```
pub struct Sc;

/// Same as `Sc` except that it treats passwords as Shift-JIS-encoded
/// when generating katakana tripcodes.
pub struct ScSjis;

/// Generator for _2ch.sc_'s 15-character tripcodes (15桁トリップ).
pub struct Sc15;

/// Generator for _2ch.sc_'s katakana tripcodes (カタカナトリップ).
pub struct ScKatakana;

/// Generator for DES-based tripcodes (4chan and 2channel's 10-character tripcode)
/// that accepts custom salt characters.
///
/// It is essentially the same as `crypt(3)`, but treats invalid salt characters in 4chan and
/// 2channel's fashion, i.e., invalid salt characters are reinterpreted as per the following table.
///
/// | 0x |  0  |  1  |  2  |  3  |  4  |  5  |  6  |  7  |  8  |  9  |  A  |  B  |  C  |  D  |  E  |  F  |
/// |----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|
/// | 00 | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` |
/// | 10 | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` |
/// | 20 | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `/` |
/// | 30 | `0` | `1` | `2` | `3` | `4` | `5` | `6` | `7` | `8` | `9` | `A` | `B` | `C` | `D` | `E` | `F` |
/// | 40 | `G` | `A` | `B` | `C` | `D` | `E` | `F` | `G` | `H` | `I` | `J` | `K` | `L` | `M` | `N` | `O` |
/// | 50 | `P` | `Q` | `R` | `S` | `T` | `U` | `V` | `W` | `X` | `Y` | `Z` | `a` | `b` | `c` | `d` | `e` |
/// | 60 | `f` | `a` | `b` | `c` | `d` | `e` | `f` | `g` | `h` | `i` | `j` | `k` | `l` | `m` | `n` | `o` |
/// | 70 | `p` | `q` | `r` | `s` | `t` | `u` | `v` | `w` | `x` | `y` | `z` | `.` | `.` | `.` | `.` | `.` |
/// | 80 | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` |
/// | 90 | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` |
/// | A0 | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` |
/// | B0 | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` |
/// | C0 | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` |
/// | D0 | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` |
/// | E0 | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` |
/// | F0 | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` | `.` |
///
/// ## Reference
///
/// * https://osdn.jp/projects/naniya/wiki/2chtrip (Japanese)
pub struct Des;

/// Trait for generators of tripcodes.
pub trait TripcodeGenerator {
    /// The type of hash value that represents resulting tripcodes.
    ///
    /// See the documentation for [`tripcode::hash`](hash/index.html) module for the information
    /// of the hash value.
    type Hash: TripcodeHash;

    /// Generates a hash value that represents the tripcode for `password`.
    fn hash<P: AsRef<[u8]>>(password: &P) -> Self::Hash;

    #[inline]
    /// Generates a tripcode from `password`.
    fn generate<P: AsRef<[u8]>>(password: &P) -> String {
        Self::hash(password).encode()
    }

    #[inline]
    /// Generates a tripcode from `password` and appends it to a `String`.
    fn append<P: AsRef<[u8]>>(password: &P, dst: &mut String) {
        Self::hash(password).append(dst);
    }

    #[inline]
    /// Generates a tripcode into a `Write`.
    fn write<P, W>(password: &P, dst: &mut W) -> io::Result<()> where P: AsRef<[u8]>, W: Write {
        Self::hash(password).write(dst)
    }

    #[inline]
    /// Generates a tripcode in Shift-JIS encoding.
    fn generate_sjis<P: AsRef<[u8]>>(password: &P) -> Vec<u8> {
        Self::hash(password).encode_to_sjis()
    }

    #[inline]
    /// Generates a Shift-JIS-encoded tripcode and appends it to a `Vec<u8>`.
    fn append_sjis<P: AsRef<[u8]>>(password: &P, dst: &mut Vec<u8>) {
        Self::hash(password).append_sjis(dst)
    }

    #[inline]
    /// Generates a Shift-JIS-encoded tripcode into a `Write`.
    fn write_sjis<P, W>(password: &P, dst: &mut W) -> io::Result<()>
        where P: AsRef<[u8]>, W: Write
    {
        Self::hash(password).write_sjis(dst)
    }
}

/// Trait for tripcode generators which may fail in generation.
pub trait TripcodeGeneratorFailable {
    /// The type of hash value that represents resulting tripcodes.
    ///
    /// See the documentation for [`tripcode::hash`](hash/index.html) module for the information
    /// of the hash value.
    type Hash: TripcodeHash;

    /// Attempts to generate a hash value from `password`.
    ///
    /// Returns `None` when passed an invalid password.
    fn try_hash<P: AsRef<[u8]>>(password: &P) -> Option<Self::Hash>;

    #[inline]
    /// Attempts to generate a tripcode from `password`.
    ///
    /// Returns `None` when passed an invalid password.
    fn try_generate<P: AsRef<[u8]>>(password: &P) -> Option<String> {
        Self::try_hash(password).map(|h| h.encode())
    }

    #[inline]
    /// Attempts to generate a tripcode into a `String`.
    ///
    /// Returns `None` when passed an invalid password.
    fn try_append<P: AsRef<[u8]>>(password: &P, dst: &mut String) -> Option<()> {
        Self::try_hash(password).map(|h| h.append(dst))
    }

    #[inline]
    /// Attempts to generate a tripcode into a `Write`.
    ///
    /// Returns `None` when passed an invalid password.
    fn try_write<P, W>(password: &P, dst: &mut W) -> Option<io::Result<()>>
        where P: AsRef<[u8]>, W: Write
    {
        Self::try_hash(password).map(|h| h.write(dst))
    }

    #[inline]
    /// Attempts to generate a tripcode in Shift-JIS encoding.
    ///
    /// Returns `None` when passed an invalid password.
    fn try_generate_sjis<P: AsRef<[u8]>>(password: &P) -> Option<Vec<u8>> {
        Self::try_hash(password).map(|h| h.encode_to_sjis())
    }

    #[inline]
    /// Attempts to generate a Shift-JIS-encoded tripcode into a `Vec<u8>`.
    ///
    /// Returns `None` when passed an invalid password.
    fn try_append_sjis<P: AsRef<[u8]>>(password: &P, dst: &mut Vec<u8>) -> Option<()>
    {
        Self::try_hash(password).map(|h| h.append_sjis(dst))
    }

    #[inline]
    /// Attempts to generate a Shift-JIS-encoded tripcode into a `Write`.
    ///
    /// Returns `None` when passed an invalid password.
    fn try_write_sjis<P, W>(password: &P, dst: &mut W) -> Option<io::Result<()>>
        where P: AsRef<[u8]>, W: Write
    {
        Self::try_hash(password).map(|h| h.write_sjis(dst))
    }
}

impl<T> TripcodeGeneratorFailable for T where T: TripcodeGenerator {
    type Hash = <Self as TripcodeGenerator>::Hash;

    #[inline]
    fn try_hash<P: AsRef<[u8]>>(password: &P) -> Option<Self::Hash> {
        Some(Self::hash(password))
    }
}

// Escapes HTML special characters in a password and yields a DES hash value of the password.
macro_rules! des_cipher_escaped {
    // `$escaper`: `fourchan_escape` or `mona_escape`.
    ($password:expr, $escaper:ident) => {{
        let mut key = 0u64;
        let (mut salt1, mut salt2) = (b'H', b'.');

        /* HTML escape and pack the password */
        let mut j = 0; // index in escaped password
        for &c in $password {
            $escaper!(c, |escaped| {
                key |= pack_u64_be(escaped) >> 8*j;
                // assuming that `escaped` is at least 3 bytes long.
                match j {
                    0 => { salt1 = escaped[1]; salt2 = escaped[2] },
                    1 => { salt1 = escaped[0]; salt2 = escaped[1] },
                    2 =>                       salt2 = escaped[0],
                    _ => (),
                }
                j += escaped.len();
            }, || {
                key |= (c as u64) << (8*(7-j));
                match j {
                    1 => salt1 = c,
                    2 => salt2 = c,
                    _ => (),
                }
                j += 1;
            });
            if j >= 8 {
                break;
            }
        }

        if j == 2 { salt2 = b'H'; }

        key = key << 1 & 0xFEFE_FEFE_FEFE_FEFE;
        let salt = decode_salt(salt1, salt2);

        des::zero_cipher_58(key, salt)
    }};
}

impl TripcodeGenerator for Fourchan {
    type Hash = FourchanHash;

    fn hash<P: AsRef<[u8]>>(password: &P) -> Self::Hash {
        FourchanHash(des_cipher_escaped!(password.as_ref(), fourchan_escape))
    }
}

impl TripcodeGenerator for FourchanNonescaping {
    type Hash = FourchanHash;

    fn hash<P: AsRef<[u8]>>(password: &P) -> Self::Hash {
        let as_ref = password.as_ref();

        let (salt1, salt2) = match as_ref.len() {
            0 | 1 => (b'H', b'.'),
            2 => (as_ref[1], b'H'),
            _ => (as_ref[1], as_ref[2]),
        };

        Des::hash(password, salt1, salt2)
    }
}

fn mona_internal<P, H, I>(password: &P, escape: bool) -> MonaHash
    where P: AsRef<[u8]>, H: TripcodeGenerator<Hash=Mona10Hash>, I: TripcodeGenerator<Hash=Mona12Hash>
{
    use hash::MonaHash::*;

    let as_ref = password.as_ref();

    let len = if escape {
        as_ref.into_iter()
            .map(|&c| mona_escape!(c, |escaped| escaped.len(), || 1 as usize))
            .sum()
    } else {
        as_ref.len()
    };

    if len >= 12 {
        let sign = as_ref[0];
        if sign == b'#' {
            match MonaRaw::try_hash(password) {
                Some(h) => Ten(h),
                None    => Error,
            }
        } else if sign == b'$' {
            Error
        } else {
            Twelve(I::hash(password))
        }
    } else {
        Ten(H::hash(password))
    }
}

impl TripcodeGenerator for Mona {
    type Hash = MonaHash;

    fn hash<P: AsRef<[u8]>>(password: &P) -> Self::Hash {
        mona_internal::<_, Mona10, Mona12>(password, true)
    }
}

impl TripcodeGenerator for MonaNonescaping {
    type Hash = MonaHash;

    fn hash<P: AsRef<[u8]>>(password: &P) -> Self::Hash {
        mona_internal::<_, Mona10Nonescaping, Mona12Nonescaping>(password, false)
    }
}

impl TripcodeGenerator for Mona10 {
    type Hash = Mona10Hash;

    fn hash<P: AsRef<[u8]>>(password: &P) -> Self::Hash {
        Mona10Hash(des_cipher_escaped!(password.as_ref(), mona_escape))
    }
}

impl TripcodeGeneratorFailable for MonaRaw {
    type Hash = Mona10Hash;

    fn try_hash<P: AsRef<[u8]>>(password: &P) -> Option<Mona10Hash> {
        let password = password.as_ref();

        macro_rules! try_dec {
            ($c1:expr, $c2:expr) => {
                match decode_salt_strict($c1, $c2) {
                    Some(s) => s,
                    None    => return None,
                }
            }
        }

        let salt = match password.len() {
            17 => 0,
            18 => try_dec!(password[17], b'.'),
            19 => try_dec!(password[17], password[18]),
            _ => return None,
        };

        macro_rules! try_hex {
            ($c:expr) => {
                match hex_to_i($c) {
                    x @ 0...0xF => x,
                    _           => return None,
                }
            }
        }

        let mut packed = [0u8; 8];
        for (i, b) in packed.iter_mut().enumerate() {
            let (d1, d0) = (password[2*i+1], password[2*i+2]);
            let byte = (try_hex!(d1) << 4) | try_hex!(d0);
            // Ignore all bytes after a null byte.
            if byte == 0 {
                if password[(2*i+1)..17].iter().all(|&c| hex_to_i(c) != 0x10) {
                    break;
                } else {
                    return None;
                }
             }
            *b = byte;
        }

        Some(Mona10Hash(des::zero_cipher_58(secret_to_key(&packed), salt)))
    }
}

/// Digests `password` with SHA-1 and passes the digest to `result`.
fn sha1_internal<T, F>(password: &[u8], escape: bool, result: F) -> T
    where F: Fn(&[u8; 20]) -> T
{
    let mut sha1 = Sha1::new();
    let mut digest: [u8; 20] = unsafe { mem::uninitialized() };

    if escape {
        let mut first = 0;
        for (i, &c) in password.iter().enumerate() {
            mona_escape!(c, |escaped| {
                sha1.input(&password[first..i]);
                sha1.input(escaped);
                first = i+1;
            }, || ());
        }
        sha1.input(&password[first..]);
    } else {
        sha1.input(password);
    }
    sha1.result(&mut digest);

    result(&digest)
}

impl TripcodeGenerator for Mona12 {
    type Hash = Mona12Hash;

    fn hash<P: AsRef<[u8]>>(password: &P) -> Mona12Hash {
        sha1_internal(password.as_ref(), true, |d| unsafe {
            Mona12Hash((*(d.as_ptr() as *const u64)).to_be(), d[8])
        })
    }
}

impl TripcodeGenerator for Mona12Nonescaping {
    type Hash = Mona12Hash;

    fn hash<P: AsRef<[u8]>>(password: &P) -> Mona12Hash {
        sha1_internal(password.as_ref(), false, |d| unsafe {
            Mona12Hash((*(d.as_ptr() as *const u64)).to_be(), d[8])
        })
    }
}

fn sc_internal<P, F>(password: &P, katakana: F) -> ScHash
    where P: AsRef<[u8]>, F: Fn(&[u8]) -> bool
{
    use hash::ScHash::*;

    let as_ref = password.as_ref();

    if as_ref.len() >= 12 {
        let sign = as_ref[0];
        if sign == b'#' {
            match MonaRaw::try_hash(password) {
                Some(h) => Ten(h),
                None    => Error,
            }
        } else if sign == b'$' {
            let h = Sc15::hash(password);
            if katakana(as_ref) {
                Katakana(ScKatakanaHash(h))
            } else {
                Fifteen(h)
            }
        } else {
            Twelve(Mona12Nonescaping::hash(password))
        }
    } else {
        Ten(FourchanNonescaping::hash(password))
    }
}

impl TripcodeGenerator for Sc {
    type Hash = ScHash;

    fn hash<P: AsRef<[u8]>>(password: &P) -> ScHash {
        sc_internal(password, sc_password_starts_with_katakana)
    }
}

impl TripcodeGenerator for ScSjis {
    type Hash = ScHash;

    fn hash<P: AsRef<[u8]>>(password: &P) -> ScHash {
        sc_internal(password, |slice| {
                let first = slice[1];
                0xA1 <= first && first <= 0xDF // [｡-ﾟ]
            }
        )
    }
}

impl TripcodeGenerator for Sc15 {
    type Hash = Sc15Hash;

    fn hash<P: AsRef<[u8]>>(password: &P) -> Sc15Hash {
        sha1_internal(&password.as_ref(), false, |d| unsafe {
            // 2ch.sc's tripcode uses 19-108th bits of SHA-1 digest.
            // Sc15Hash(u64, u32) ->
            // u64: 0b 11111111 11111111 11111111 11111111 11111111 11111111 11111111 11110000
            //         ↑ 19th bit                                                        ↑ 78th bit
            // u32: 0b 11111111 11111111 11111111 11111100
            //         ↑ 79th bit                      ↑ 108th bit
            Sc15Hash(
                // 19-78th bits
                u64::from_be(*(d.as_ptr().offset(2) as *const u64)) << 2 & 0xFFFF_FFFF_FFFF_FFF0,
                // 79-108th bits
                (u64::from_be(*(d.as_ptr().offset(6) as *const u64)) >> 2 & 0xFFFF_FFFC) as u32
            )
        })
    }
}

impl TripcodeGenerator for ScKatakana {
    type Hash = ScKatakanaHash;

    fn hash<P: AsRef<[u8]>>(password: &P) -> ScKatakanaHash {
        ScKatakanaHash(Sc15::hash(password))
    }
}

impl Des {
    /// Generates a hash value from `password` and a pair of custom salt characters.
    pub fn hash<P: AsRef<[u8]>>(password: &P, salt1: u8, salt2: u8) -> FourchanHash {
        let key = secret_to_key(password.as_ref());
        let salt = decode_salt(salt1, salt2);
        FourchanHash(des::zero_cipher_58(key, salt))
    }

    #[inline]
    /// Generates a tripcode from `password` and a pair of custom salt characters.
    pub fn generate<P: AsRef<[u8]>>(password: &P, salt1: u8, salt2: u8) -> String {
        Self::hash(password, salt1, salt2).encode()
    }

    #[inline]
    /// Generates a tripcode and appends it to a `String`.
    pub fn append<P: AsRef<[u8]>>(password: &P, salt1: u8, salt2: u8, dst: &mut String) {
        Self::hash(password, salt1, salt2).append(dst);
    }

    #[inline]
    /// Generates a tripcode into a `Write`.
    pub fn write<P, W>(password: &P, salt1: u8, salt2: u8, dst: &mut W) -> io::Result<()> where P: AsRef<[u8]>, W: Write {
        Self::hash(password, salt1, salt2).write(dst)
    }
}

#[cfg(test)]
mod tests {
    extern crate encoding;

    use self::encoding::all::WINDOWS_31J as SJIS;
    use self::encoding::{Encoding, EncoderTrap};
    use super::*;

    macro_rules! assert_tripcode_eq {
        ($expected:expr, $password:expr) => {
            assert_tripcode_eq!($expected, $password, Mona);
        };

        ($expected:expr, $password:expr, $hasher:ty) => {{
            let tripcode = <$hasher>::generate(&$password);
            assert_tripcode_eq!(cmp $expected, tripcode, $password);

            let mut tripcode = String::with_capacity(45);
            <$hasher>::append(&$password, &mut tripcode);
            assert_tripcode_eq!(cmp $expected, &tripcode, $password);

            let mut tripcode = [0u8; 15];
            <$hasher>::write(&$password, &mut (&mut tripcode as &mut [u8])).unwrap();
            assert_tripcode_eq!(cmp $expected, String::from_utf8_lossy(&tripcode[..$expected.len()]), $password);
        }};

        (cmp $expected:expr, $tripcode:expr, $password:expr) => {
            assert!($expected == $tripcode,
                "tripcode mismatched: expected: `{:?}`, tripcode: `{:?}`, password: `{:?}`",
                $expected, $tripcode, $password)
        };
    }

    #[test]
    fn mona_10_matches() {
        assert_tripcode_eq!("jPpg5.obl6", r"");
        assert_tripcode_eq!("nOA3ItxPxI", r"k");
        assert_tripcode_eq!("GDsuFp4oF6", r"[K");
        assert_tripcode_eq!("IG4wjn.Cxc", r"2 V");
        assert_tripcode_eq!("P97zJ5IHPI", r"|TB~");
        assert_tripcode_eq!(".HIpR.ZMqM", r"(~A5|");
        assert_tripcode_eq!("9zAZPOvSZI", r"[6??Pz");
        assert_tripcode_eq!("KgeLOKK0NQ", r"ErdxpJ$");
        assert_tripcode_eq!("BX6/llcs1o", r"dib3Q_4x");
        assert_tripcode_eq!("6trdEPfEr6", r"R%!IuxM.t");
    }

    #[test]
    fn mona_raw_matches() {
        assert_tripcode_eq!("IP9Lda5FPc", "#0123456789abcdef./");
        assert_tripcode_eq!("7Uzd/KllpE", "#FF00000000000000");
        assert_tripcode_eq!("7Uzd/KllpE", "#FF00FFFFFFFFFFFF");
    }

    #[test]
    fn mona_12_matches() {
        assert_tripcode_eq!("50D13FhHVb0y", "POVD@psDFdsopfij");
        assert_tripcode_eq!("/ybNw16ve2hX", "123ABCdef#=?");
    }

    #[test]
    fn mona_invalid() {
        assert_tripcode_eq!("???", "$23456789012");
        assert_tripcode_eq!("???", "#abcdefghijklmnop");
        assert_tripcode_eq!("???", "#fedcba9876543210!!");
        assert_tripcode_eq!("???", "#abcdef0123456789ghi");
        assert_tripcode_eq!("???", "#00abcdefghijklmn..");
    }

    #[test]
    fn sc_matches() {
        let (t, k) = ("ﾃｽﾄ!ｹﾏﾜｬｴ･ｧﾎﾖｲﾎ", "$｡1008343131");
        let k = SJIS.encode(k, EncoderTrap::Strict).unwrap();
        assert_eq!(t, &ScSjis::generate(&k));

        assert_tripcode_eq!("h3Si!7m4Qie8e.u", "$0123456789a", Sc);
    }

    #[test]
    fn html_escaping() {
        assert_tripcode_eq!("TIWS518hyaVm", "abc&quot;def");
        assert_tripcode_eq!("TIWS518hyaVm", "abc\"def");

        assert_tripcode_eq!("pxr6zSrasrOD", "ab&quot;cdef");
        assert_tripcode_eq!("pxr6zSrasrOD", "ab\"cdef");

        assert_tripcode_eq!("ZC2NVileD3Mz", "a&quot;bcdef");
        assert_tripcode_eq!("ZC2NVileD3Mz", "a\"bcdef");

        assert_tripcode_eq!("Wg38i4X473pB", "&quot;abcdef");
        assert_tripcode_eq!("Wg38i4X473pB", "\"abcdef");

        assert_tripcode_eq!("Gw/f5wZwNg",   "&lt;&gt;");
        assert_tripcode_eq!("Gw/f5wZwNg",   "<>");

        assert_tripcode_eq!("LZ4ugyvTWU",   "&quot;&lt;");
        assert_tripcode_eq!("LZ4ugyvTWU",   "\"<");

        assert_tripcode_eq!("MhCJJ7GVT.",   "&amp;");
        assert_tripcode_eq!("MhCJJ7GVT.",   "&", Fourchan);
        assert_tripcode_eq!("2r2Ga7GHRc",   "&");
    }

    #[test]
    fn des() {
        let tripcode = Des::generate(&"password", b'a', b's');
        assert_eq!("ozOtJW9BFA", &tripcode);
        let tripcode = Des::generate(&"", b'H', b'.');
        assert_eq!("jPpg5.obl6", &tripcode);
    }

    #[test]
    fn append() {
        let mut tripcode = String::new();

        Mona10::append(&"a", &mut tripcode);
        Mona10::append(&"b", &mut tripcode);
        Mona10::append(&"c", &mut tripcode);
        Mona12::append(&"0123456789ab", &mut tripcode);
        Mona::append(&"##0123456789", &mut tripcode);
        let k = SJIS.encode(&"$｡1008343131", EncoderTrap::Strict).unwrap();
        ScKatakana::append(&k, &mut tripcode);
        Sc15::append(&"$a9876543210", &mut tripcode);
        Des::append(&"de", b'e', b'H', &mut tripcode);

        assert_eq!(
            "ZnBI2EKkq.\
             taAZ7oPCCM\
             wG1CV58ydQ\
             Ly0gXVRR0yVs\
             ???\
             ﾃｽﾄ!ｹﾏﾜｬｴ･ｧﾎﾖｲﾎ\
             x.r.XzgFZywTJhG\
             yqYXjvHgbk\
            ",
            &tripcode
         );
    }
}
