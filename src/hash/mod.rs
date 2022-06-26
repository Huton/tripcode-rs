// Copyright 2016 Huton. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/*!
Tools for handling lightweight hash values that can be encoded to/decoded from tripcodes.

# Examples
```
use tripcode::TripcodeGenerator;
use tripcode::hash::TripcodeHash;

// Generating a hash value from a password.
let hash = tripcode::Fourchan::hash(&"password");
assert_eq!(hash, tripcode::hash::FourchanHash(0xD3F6B95622CD44C0));

// Encoding the hash value into a tripcode.
let tripcode = hash.encode();
assert_eq!("ozOtJW9BFA", &tripcode);

// Decoding back the tripcode into a hash value.
let decoded = tripcode::hash::FourchanHash::decode(&tripcode).unwrap();
assert_eq!(decoded, hash);
```
*/

mod enc_dec;

use self::enc_dec::EncoderDecoder;
use std::{io, mem};
use std::io::Write;
use mem::MaybeUninit;

/// 58-bit hash value that represents a 10-character tripcode
/// i.e. 4chan's tripcode or 2channel's 10-character tripcode (10桁トリップ).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FourchanHash(pub u64);

pub use self::FourchanHash as Mona10Hash;

/// 72-bit hash value that represents 2channel's 12-character tripcode (12桁トリップ).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Mona12Hash(pub u64, pub u8);

/// 90-bit hash value that represents _2ch.sc_'s 15-character tripcode (15桁トリップ).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sc15Hash(pub u64, pub u32);

/// 90-bit hash value that represents _2ch.sc_'s katakana tripcode (カタカナトリップ).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScKatakanaHash(pub Sc15Hash);

/// Hash value that represents a 2channel tripcode.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MonaHash {
    /// Hash value for a 10-character tripcode (10桁トリップ)
    Ten(Mona10Hash),
    /// Hash value for a 12-character tripcode (12桁トリップ)
    Twelve(Mona12Hash),
    /// Tripcode for undefined password format.
    Error,
}

/// Hash value that represents a _2ch.sc_ tripcode.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScHash {
    /// Hash value for a 10-character tripcode (10桁トリップ)
    Ten(Mona10Hash),
    /// Hash value for a 12-character tripcode (12桁トリップ)
    Twelve(Mona12Hash),
    /// Hash value for a 15-character tripcode (15桁トリップ)
    Fifteen(Sc15Hash),
    /// Hash value for a katakana tripcode (カタカナトリップ)
    Katakana(ScKatakanaHash),
    /// Tripcode for undefined password format.
    Error,
}

macro_rules! try_dec {
    ($d:expr) => {
        match $d {
            0..=0x3F => $d,
            _            => return None,
        }
    }
}

/// Trait for hash values that represent tripcodes.
pub trait TripcodeHash : Sized {
    /// Decodes a tripcode into a hash value.
    ///
    /// Returns `None` when passed a invalid tripcode.
    fn decode(tripcode: &str) -> Option<Self>;

    /// Returns maximum length of resulting tripcode, in bytes.
    fn max_len() -> usize;

    /// Returns maximum length of resulting tripcode when encoded in Shift-JIS, in bytes.
    fn max_len_sjis() -> usize;

    /// Encodes `self` into a tripcode and appends it on a `String`.
    fn append(self, _: &mut String);

    /// Encodes `self` into a tripcode and writes it to a `Write`.
    fn write<W: Write>(self, _: &mut W) -> io::Result<()>;

    /// Decodes a Shift-JIS-encoded tripcode.
    fn decode_from_sjis(_: &[u8]) -> Option<Self>;

    /// Encodes `self` into a Shift-JIS-encoded tripcode and appends it on a `Vec<u8>`.
    fn append_sjis(self, _: &mut Vec<u8>);

    /// Encodes `self` into a Shift-JIS-encoded tripcode and writes it to a `Write`.
    fn write_sjis<W: Write>(self, _: &mut W) -> io::Result<()>;

    #[inline]
    /// Encodes `self` into a tripcode.
    fn encode(self) -> String {
        let mut ret = String::with_capacity(Self::max_len());
        self.append(&mut ret);
        ret
    }

    #[inline]
    /// Encodes `self` into a Shift-JIS-encoded tripcode.
    fn encode_to_sjis(self) -> Vec<u8> {
        let mut ret = Vec::with_capacity(Self::max_len_sjis());
        self.append_sjis(&mut ret);
        ret
    }
}

/// Trait for hash values that represent ASCII-encoded tripcodes.
///
/// Users should not use the methods of this trait directly. Instead, use the methods of the trait
/// `TripcodeHash`, which is automatically implemented for types of this trait.
pub trait AsciiTripcodeHash : Sized {
    /// Decodes an ASCII-encoded tripcode into a hash value.
    fn decode_from_ascii(_: &[u8]) -> Option<Self>;

    /// Returns maximum length of resulting tripcode.
    fn max_len() -> usize;

    /// Encodes `self` into an ASCII-encoded tripcode and writes it on a `String`.
    fn append_ascii(self, _: &mut Vec<u8>);

    /// Encodes `self` into an ASCII-encoded tripcode and writes it to a `Write`.
    fn write_ascii<W: Write>(self, _: &mut W) -> io::Result<()>;

    #[inline]
    /// Encodes `self` into an ASCII-encoded tripcode.
    fn encode_to_ascii(self) -> Vec<u8> {
        let mut ret = Vec::with_capacity(Self::max_len());
        self.append_ascii(&mut ret);
        ret
    }
}

impl<T> TripcodeHash for T where T: AsciiTripcodeHash {
    #[inline]
    fn decode(tripcode: &str) -> Option<Self> {
        Self::decode_from_ascii(tripcode.as_bytes())
    }

    #[inline]
    fn max_len() -> usize {
        <Self as AsciiTripcodeHash>::max_len()
    }

    #[inline]
    fn max_len_sjis() -> usize {
        <Self as AsciiTripcodeHash>::max_len()
    }

    #[inline]
    fn append(self, dst: &mut String) {
        unsafe {
            self.append_ascii(dst.as_mut_vec());
        }
    }

    #[inline]
    fn write<W: Write>(self, dst: &mut W) -> io::Result<()> {
        self.write_ascii(dst)
    }

    #[inline]
    fn decode_from_sjis(tripcode: &[u8]) -> Option<Self> {
        Self::decode_from_ascii(tripcode)
    }

    #[inline]
    fn append_sjis(self, dst: &mut Vec<u8>) {
        self.append_ascii(dst);
    }

    #[inline]
    fn write_sjis<W: Write>(self, dst: &mut W) -> io::Result<()> {
        self.write_ascii(dst)
    }

    #[inline]
    fn encode(self) -> String {
        unsafe {
            mem::transmute(self.encode_to_ascii())
        }
    }

    #[inline]
    fn encode_to_sjis(self) -> Vec<u8> {
        self.encode_to_ascii()
    }
}

impl AsciiTripcodeHash for MonaHash {
    fn decode_from_ascii(tripcode: &[u8]) -> Option<Self> {
        use self::MonaHash::*;

        match tripcode.len() {
            10 => Mona10Hash::decode_from_ascii(tripcode).map(Ten),
            12 => Mona12Hash::decode_from_ascii(tripcode).map(Twelve),
            3  => if tripcode == b"???" { Some(Error) } else { None },
            _  => None,
        }
    }

    #[inline]
    fn max_len() -> usize {
        12
    }

    fn append_ascii(self, dst: &mut Vec<u8>) {
        use self::MonaHash::*;

        match self {
            Twelve(h) => h.append_ascii(dst),
            Ten(h)    => h.append_ascii(dst),
            Error     => dst.extend_from_slice(b"???")
        }
    }

    fn write_ascii<W: Write>(self, dst: &mut W) -> io::Result<()> {
        use self::MonaHash::*;

        match self {
            Twelve(h) => h.write_ascii(dst),
            Ten(h) => h.write_ascii(dst),
            Error => dst.write_all(b"???"),
        }
    }

    fn encode_to_ascii(self) -> Vec<u8> {
        use hash::MonaHash::*;

        match self {
            Ten(h)    => h.encode_to_ascii(),
            Twelve(h) => h.encode_to_ascii(),
            Error     => b"???".to_vec(),
        }
    }
}

impl AsciiTripcodeHash for FourchanHash {
    fn decode_from_ascii(tripcode: &[u8]) -> Option<Self> {
        if tripcode.len() != 10 { return None; }

        let mut ret = 0u64;

        for &c in tripcode.iter().take(9) {
            ret |= try_dec!(enc_dec::Crypt::decode(c));
            ret <<= 6;
        }
        ret |= try_dec!(enc_dec::CryptLastChar::decode(tripcode[9]));
        ret <<= 4;

        Some(FourchanHash(ret))
    }

    #[inline]
    fn max_len() -> usize {
        10
    }

    fn append_ascii(mut self, dst: &mut Vec<u8>) {
        let len = dst.len();
        dst.reserve(10);

        unsafe { dst.set_len(len + 10); }

        for b in dst.iter_mut().skip(len) {
            *b = enc_dec::Crypt::encode((self.0 >> 58) as usize);
            self.0 <<= 6;
        }
    }

    fn write_ascii<W: Write>(mut self, dst: &mut W) -> io::Result<()> {
        let mut buf: [u8; 10] = unsafe { MaybeUninit::zeroed().assume_init() };

        for b in &mut buf {
            *b = enc_dec::Crypt::encode((self.0 >> 58) as usize);
            self.0 <<= 6;
        }

        dst.write_all(&buf)
    }
}

macro_rules! encode_mona_12_main {
    ($hash:expr, $dst:expr) => {{
        for i in 0..10 {
            $dst[i] = enc_dec::Base64::encode(($hash.0 >> 58) as usize);
            $hash.0 <<= 6;
        }
        $dst[10] = enc_dec::Base64::encode((($hash.0 >> 58) | ($hash.1 as u64 >> 6)) as usize);
        $dst[11] = enc_dec::Base64::encode(($hash.1 & 0b111111) as usize);
    }};
}

impl AsciiTripcodeHash for Mona12Hash {
    fn decode_from_ascii(tripcode: &[u8]) -> Option<Self> {
        if tripcode.len() != 12 { return None; }

        let mut ret = Mona12Hash(0u64, 0u8);

        for &c in tripcode.iter().take(10) {
            ret.0 <<= 6;
            ret.0 |= try_dec!(enc_dec::Base64::decode(c));
        }
        ret.0 <<= 4;

        let d11 = try_dec!(enc_dec::Base64::decode(tripcode[10]));
        ret.0 |= d11 >> 2;
        ret.1 = (d11 << 6 | try_dec!(enc_dec::Base64::decode(tripcode[11]))) as u8;

        Some(ret)
    }

    #[inline]
    fn max_len() -> usize {
        12
    }

    fn append_ascii(mut self, dst: &mut Vec<u8>) {
        let len = dst.len();
        dst.reserve(12);
        unsafe { dst.set_len(len+12); }
        encode_mona_12_main!(self, dst[len..]);
    }

    fn write_ascii<W: Write>(mut self, dst: &mut W) -> io::Result<()> {
        let mut buf: [u8; 12] = unsafe { MaybeUninit::zeroed().assume_init() };
        encode_mona_12_main!(self, buf);
        dst.write_all(&buf)
    }
}

impl TripcodeHash for ScHash {
    fn decode(tripcode: &str) -> Option<Self> {
        use self::ScHash::*;

        match tripcode.len() {
            10           => Mona10Hash::decode(tripcode).map(Ten),
            12           => Mona12Hash::decode(tripcode).map(Twelve),
            // The only 15-byte-long katakana tripcode "!!!!!!!!!!!!!!!" is regarded as a 15-character tripcode.
            15           => Sc15Hash::decode(tripcode).map(Fifteen),
            3            => if tripcode == "???" { Some(Error) } else { None },
            l if l >= 17 => ScKatakanaHash::decode(tripcode).map(Katakana),
            _            => None,
        }
    }

    #[inline]
    fn max_len() -> usize {
        45
    }

    #[inline]
    fn max_len_sjis() -> usize {
        15
    }

    fn append(self, dst: &mut String) {
        use self::ScHash::*;

        match self {
            Ten(h)      => h.append(dst),
            Twelve(h)   => h.append(dst),
            Fifteen(h)  => h.append(dst),
            Katakana(h) => h.append(dst),
            Error       => dst.push_str("???")
        }
    }

    fn write<W: Write>(self, dst: &mut W) -> io::Result<()> {
        use self::ScHash::*;

        match self {
            Ten(h)      => h.write(dst),
            Twelve(h)   => h.write(dst),
            Fifteen(h)  => h.write(dst),
            Katakana(h) => h.write(dst),
            Error       => dst.write_all(b"???"),
        }
    }

    fn decode_from_sjis(tripcode: &[u8]) -> Option<Self> {
        use self::ScHash::*;

        match tripcode.len() {
            10 => Mona10Hash::decode_from_sjis(tripcode).map(Ten),
            12 => Mona12Hash::decode_from_sjis(tripcode).map(Twelve),
            15 => Sc15Hash::decode_from_sjis(tripcode).map(Fifteen).or_else(||
                ScKatakanaHash::decode_from_sjis(tripcode).map(Katakana)
            ),
            3 => if tripcode == b"???" { Some(Error) } else { None },
            _ => None,
        }
    }

    fn append_sjis(self, dst: &mut Vec<u8>) {
        use self::ScHash::*;

        match self {
            Ten(h)      => h.append_sjis(dst),
            Twelve(h)   => h.append_sjis(dst),
            Fifteen(h)  => h.append_sjis(dst),
            Katakana(h) => h.append_sjis(dst),
            Error       => dst.extend_from_slice(b"???")
        }
    }

    fn write_sjis<W: Write>(self, dst: &mut W) -> io::Result<()> {
        use self::ScHash::*;

        match self {
            Ten(h)      => h.write_sjis(dst),
            Twelve(h)   => h.write_sjis(dst),
            Fifteen(h)  => h.write_sjis(dst),
            Katakana(h) => h.write_sjis(dst),
            Error       => dst.write_all(b"???"),
        }
    }

    fn encode(self) -> String {
        use hash::ScHash::*;

        match self {
            Ten(h)      => h.encode(),
            Twelve(h)   => h.encode(),
            Fifteen(h)  => h.encode(),
            Katakana(h) => h.encode(),
            Error       => "???".to_owned(),
        }
    }

    fn encode_to_sjis(self) -> Vec<u8> {
        use hash::ScHash::*;

        match self {
            Ten(h)      => h.encode_to_sjis(),
            Twelve(h)   => h.encode_to_sjis(),
            Fifteen(h)  => h.encode_to_sjis(),
            Katakana(h) => h.encode_to_sjis(),
            Error       => b"???".to_vec(),
        }
    }
}

macro_rules! encode_sc_sha1_main {
    ($encoder:ty, $hash:expr, $dst:expr) => {{
        for i in 0..10 {
            $dst[i] = <$encoder>::encode(($hash.0 >> 58) as usize);
            $hash.0 <<= 6;
        }
        for i in 10..15 {
            $dst[i] = <$encoder>::encode(($hash.1 >> 26) as usize);
            $hash.1 <<= 6;
        }
    }};
}

fn decode_sc_sha1_internal<D, T, G>(tripcode: &[u8], wrapper: G) -> Option<T>
    where D: enc_dec::EncoderDecoder, G: Fn(Sc15Hash) -> T
{
    if tripcode.len() != 15 { return None; }
    let mut ret = Sc15Hash(0, 0);

    let mut iter = tripcode.iter();

    for &c in iter.by_ref().take(10) {
        ret.0 <<= 6;
        ret.0 |= try_dec!(D::decode(c));
    }
    ret.0 <<= 4;

    for &c in iter {
        ret.1 <<= 6;
        ret.1 |= try_dec!(D::decode(c)) as u32;
    }
    ret.1 <<= 2;

    Some(wrapper(ret))
}

impl AsciiTripcodeHash for Sc15Hash {
    fn decode_from_ascii(tripcode: &[u8]) -> Option<Self> {
        decode_sc_sha1_internal::<enc_dec::Sc15, _, _>(tripcode, |h| h)
    }

    #[inline]
    fn max_len() -> usize {
        15
    }

    fn append_ascii(mut self, dst: &mut Vec<u8>) {
        let len = dst.len();
        dst.reserve(15);
        unsafe { dst.set_len(len+15); }
        encode_sc_sha1_main!(enc_dec::Sc15, self, dst[len..]);
    }

    fn write_ascii<W: Write>(mut self, dst: &mut W) -> io::Result<()> {
        let mut buf: [u8; 15] = unsafe { MaybeUninit::zeroed().assume_init() };
        encode_sc_sha1_main!(enc_dec::Sc15, self, buf);
        dst.write_all(&buf)
    }
}

impl TripcodeHash for ScKatakanaHash {
    fn decode(tripcode: &str) -> Option<Self> {
        const SC_KATAKANA_DECODING_EFBD: [u8; 0x100] = [
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x00, 0x01, 0x02, 0x03, 0x04,
            0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
        ];

        const SC_KATAKANA_DECODING_EFBE: [u8; 0x100] = [
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x24,
            0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32, 0x33, 0x3E,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
        ];

        macro_rules! try_dec_kana {
            ($iter:expr) => {
                // See also the trie in `::utils::sc_password_starts_with_katakana()`.
                match $iter.next() {
                    Some(&0xEF) => match $iter.next() {
                        Some(&0xBD) => match $iter.next() {
                            Some(&b) => try_dec!(SC_KATAKANA_DECODING_EFBD[b as usize]),
                            _ => return None,
                        },
                        Some(&0xBE) => match $iter.next() {
                            Some(&b) => try_dec!(SC_KATAKANA_DECODING_EFBE[b as usize]),
                            _  => return None,
                        },
                        _ => return None,
                    },
                    Some(&b'!') => 0x3F,
                    _ => return None,
                }
            }
        }

        let mut iter = tripcode.as_bytes().iter();
        let mut ret = Sc15Hash(0, 0);

        for _ in 0..10 {
            ret.0 <<= 6;
            ret.0 |= try_dec_kana!(iter) as u64;
        }
        ret.0 <<= 4;

        for _ in 10..15 {
            ret.1 <<= 6;
            ret.1 |= try_dec_kana!(iter) as u32;
        }
        ret.1 <<= 2;

        match iter.next() {
            None => Some(ScKatakanaHash(ret)),
            _    => None,
        }
    }

    #[inline]
    fn max_len() -> usize {
        45
    }

    #[inline]
    fn max_len_sjis() -> usize {
        15
    }

    fn append(mut self, dst: &mut String) {
        unsafe {
            let dst = dst.as_mut_vec();
            for _ in 0..10 {
                dst.extend_from_slice(enc_dec::ScKatakana::encode(((self.0).0 >> 58) as usize));
                (self.0).0 <<= 6;
            }
            for _ in 10..15 {
                dst.extend_from_slice(enc_dec::ScKatakana::encode(((self.0).1 >> 26) as usize));
                (self.0).1 <<= 6;
            }
        }
    }

    fn write<W: Write>(mut self, dst: &mut W) -> io::Result<()> {
        for _ in 0..10 {
            (dst.write_all(enc_dec::ScKatakana::encode(((self.0).0 >> 58) as usize)))?;
            (self.0).0 <<= 6;
        }
        for _ in 10..15 {
            (dst.write_all(enc_dec::ScKatakana::encode(((self.0).1 >> 26) as usize)))?;
            (self.0).1 <<= 6;
        }
        Ok(())
    }

    fn decode_from_sjis(tripcode: &[u8]) -> Option<Self> {
        decode_sc_sha1_internal::<enc_dec::ScSjisKatakana, _, _>(tripcode, ScKatakanaHash)
    }

    fn append_sjis(mut self, dst: &mut Vec<u8>) {
        let len = dst.len();
        dst.reserve(15);
        unsafe { dst.set_len(len+15) };
        encode_sc_sha1_main!(enc_dec::ScSjisKatakana, self.0, dst[len..]);
    }

    fn write_sjis<W: Write>(mut self, dst: &mut W) -> io::Result<()> {
        let mut buf: [u8; 15] = unsafe { MaybeUninit::zeroed().assume_init() };
        encode_sc_sha1_main!(enc_dec::ScSjisKatakana, self.0, buf);
        dst.write_all(&buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_dec_enc {
        ($h:ident, $t:expr) => (assert_eq!($t, &$h::decode($t).unwrap().encode()))
    }

    #[test]
    fn des_dec_enc() {
        test_dec_enc!(Mona10Hash, "abcd/0123.");
        test_dec_enc!(Mona10Hash, "/././././.");
    }

    #[test]
    fn sha1_dec_enc() {
        test_dec_enc!(Mona12Hash, "Tripcode+rs/");
        test_dec_enc!(Mona12Hash, "Sha1/dec+enc");
    }

    #[test]
    fn mona_dec_enc() {
        test_dec_enc!(MonaHash, "Ten/bytes.");
        test_dec_enc!(MonaHash, "Twelve/bytes");
        test_dec_enc!(MonaHash, "???");
    }

    #[test]
    fn sc_15_dec_enc() {
        test_dec_enc!(Sc15Hash, "Fifteen!bytes!!");
        test_dec_enc!(Sc15Hash, "!!!!!!!!!!!!!!!");
    }

    #[test]
    fn sc_katakana_dec_enc() {
        test_dec_enc!(ScKatakanaHash, "ｩﾜｮｩｼﾞｮｯｮｨｧｪｫｬｭ");
        test_dec_enc!(ScKatakanaHash, "ｲﾛﾊﾆﾎﾍﾄﾁﾘﾇﾙｦﾜｶﾖ");
        test_dec_enc!(ScKatakanaHash, "ﾀﾚｿﾂﾈﾔﾗﾑｳｲﾉｵｸﾔﾏ");
        test_dec_enc!(ScKatakanaHash, "ｹﾌｺｴﾃｱｻｷﾕﾒﾐｼｴｲﾓ");
        test_dec_enc!(ScKatakanaHash, "ｾｽﾝ!!!!!!ﾟ｡｢｣､･");
        test_dec_enc!(ScKatakanaHash, "!!!!!!!!!!!!!!!");
    }

    #[test]
    fn sc_katakana_sjis_enc_dec() {
        let h = ScKatakanaHash(Sc15Hash(0x0123456789ABCDE0, 0xFEDCBA98));
        assert_eq!(h, ScKatakanaHash::decode_from_sjis(&h.encode_to_sjis()).unwrap());
    }

    #[test]
    fn sc_dec_enc() {
        test_dec_enc!(ScHash, "123456789.");
        test_dec_enc!(ScHash, "0123456789ab");
        test_dec_enc!(ScHash, "123456789abcdef");
        test_dec_enc!(ScHash, "ｶﾀｶﾅﾄﾘｯﾌﾟﾃｽﾄﾃﾞｽ");
        test_dec_enc!(ScHash, "!!!!!!!!!!!!!!!");
        test_dec_enc!(ScHash, "???");
    }

    #[test]
    fn sc_sjis_enc_dec() {
        use super::ScHash::*;

        let h = Sc15Hash(0x0123456789ABCDE0, 0xFEDCBA98);
        let w = Katakana(ScKatakanaHash(h));
        assert_eq!(w, ScHash::decode_from_sjis(&w.encode_to_sjis()).unwrap());
        let w = Fifteen(h);
        assert_eq!(w, ScHash::decode_from_sjis(&w.encode_to_sjis()).unwrap());
    }

    #[test]
    fn decode_fails() {
        assert!(Mona10Hash::decode("hocho.🔪").is_none());

        assert!(Mona12Hash::decode("fried🍤ebi").is_none());

        assert!(MonaHash::decode("123456789abcdef").is_none());
        assert!(MonaHash::decode("+++++++++.").is_none());
        assert!(MonaHash::decode("............").is_none());

        assert!(Sc15Hash::decode("Lorem ipsum dolor").is_none());
        assert!(Sc15Hash::decode("Fifteen?bytes??").is_none());
        assert!(Sc15Hash::decode("ｶﾀｶﾅﾄﾘｯﾌﾟﾃﾞｽﾖ!!").is_none());

        assert!(ScKatakanaHash::decode("カタカナトリップテストテスト！").is_none());
        assert!(ScKatakanaHash::decode("ﾐｼﾞｶｽｷﾞﾙ!ﾀﾝｼｮｳ").is_none());
        assert!(ScKatakanaHash::decode("ｶﾀｶﾅﾄﾘｯﾌﾟﾓﾄﾞｷ!!!").is_none());
        assert!(ScKatakanaHash::decode("FifteenCharTrip").is_none());

        assert!(ScKatakanaHash::decode_from_sjis(b"0123456789abcde").is_none());

        assert!(ScHash::decode("abcdeｲﾛﾊﾆﾎ12345").is_none());
        assert!(ScHash::decode("0123456789abcdef").is_none());
        assert!(ScHash::decode("!!!").is_none());
    }
}
