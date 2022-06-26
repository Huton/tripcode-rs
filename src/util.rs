// Copyright 2016 Huton. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{cmp, ptr};

/// Helper macro for generating HTML escaping macros.
macro_rules! escape {
    ($c:expr, $escaped:ident, $and_then:expr, $or_else:expr, $([$from:pat, $to:expr]),*) => {
        match $c {
            $(
                $from => {
                    let $escaped = $to;
                    $and_then
                }
            ),*
            _ => $or_else,
        }
    };
}

/// HTML ecaping macro for passwords of 4chan's tripcodes.
macro_rules! fourchan_escape {
    ($c:expr, |$escaped:ident| $and_then:expr, || $or_else:expr) => {
        escape!($c, $escaped, $and_then, $or_else, [b'"', b"&quot;"], [b'&', b"&amp;"], [b'<', b"&lt;"], [b'>', b"&gt;"])
    };
}

/// HTML ecaping macro for passwords of 2channel's tripcodes.
macro_rules! mona_escape {
    ($c:expr, |$escaped:ident| $and_then:expr, || $or_else:expr) => {
        escape!($c, $escaped, $and_then, $or_else, [b'"', b"&quot;"], [b'<', b"&lt;"], [b'>', b"&gt;"])
    };
}

/// Reinterprets the byte array as 64-bit big-endian unsigned integral value and returns it.
/// Any out-of-bounds byte will be treated as if being zero.
pub fn pack_u64_be(bytes: &[u8]) -> u64 {
    let mut ret = 0u64;
    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), &mut ret as *mut _ as *mut u8, cmp::min(bytes.len(), 8));
    }
    u64::from_be(ret)
}

/// Converts password for used by the DES cipher.
pub fn secret_to_key(password: &[u8]) -> u64 {
    pack_u64_be(password) << 1 & 0xFEFE_FEFE_FEFE_FEFE
}

/// Returns `true` if the second character (next to `'$'` sign) of `password` is
/// a half-width katakana.
pub fn sc_password_starts_with_katakana(password: &[u8]) -> bool {
    // Trie for UTF-8 half-width katakanas:
    //
    //          /->(0xBD)->(0xA1-0xBF)
    // ()->(0xEF)
    //          \->(0xBE)->(0x80-0x9F)
    if password[1] == 0xEF {
        match password[2] {
            0xBD => {
                let b = password[3];
                return 0xA1 <= b && b <= 0xBF;
            },
            0xBE => {
                let b = password[3];
                return 0x80 <= b && b <= 0x9F;
            },
            _ => (),
        }
    }
    false
}

/// Converts a hexadecimal character into number.
/// Returns `0x10` when an invalid character is passed.
pub fn hex_to_i(c: u8) -> u8 {
    const HEX_DECODING: &'static [u8; 0x100] = b"\
        \x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\
        \x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\
        \x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\
        \x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x10\x10\x10\x10\x10\x10\
        \x10\x0a\x0b\x0c\x0d\x0e\x0f\x10\x10\x10\x10\x10\x10\x10\x10\x10\
        \x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\
        \x10\x0a\x0b\x0c\x0d\x0e\x0f\x10\x10\x10\x10\x10\x10\x10\x10\x10\
        \x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\
        \x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\
        \x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\
        \x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\
        \x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\
        \x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\
        \x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\
        \x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\
        \x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10";

    HEX_DECODING[c as usize]
}

/// Decodes a pair of salt characters in a tripcode password.
pub fn decode_salt(salt1: u8, salt2: u8) -> u32 {
    const SALT_DECODING: [u32; 256] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11,
        0x12, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A,
        0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A,
        0x2B, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32, 0x33, 0x34,
        0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    (SALT_DECODING[salt1 as usize] << 26) | (SALT_DECODING[salt2 as usize] << 18)
}

/// Decodes a pair of salt characters in a password of a nama key tripcode (生キートリップ).
/// Returns `None` when an invalid salt character is passed.
pub fn decode_salt_strict(salt1: u8, salt2: u8) -> Option<u32> {
    const SALT_DECODING_STRICT: [u32; 256] = [
        0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
        0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
        0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x00, 0x01,
        0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
        0x40, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A,
        0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x40, 0x40, 0x40, 0x40, 0x40,
        0x40, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32, 0x33, 0x34,
        0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x40, 0x40, 0x40, 0x40, 0x40,
        0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
        0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
        0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
        0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
        0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
        0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
        0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
        0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    ];

    macro_rules! try_dec {
        ($c:expr) => {
            match SALT_DECODING_STRICT[$c as usize] {
                d @ 0..=0x3F => d,
                _            => return None,
            }
        };
    }

    Some((try_dec!(salt1) << 26) | (try_dec!(salt2) << 18))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_with_katakana() {
        assert!(sc_password_starts_with_katakana("$｡春".as_bytes()));
        assert!(sc_password_starts_with_katakana("$｢は".as_bytes()));
        assert!(sc_password_starts_with_katakana("$｣あ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$､げ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$･ぽ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｦよー".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｧa".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｨb".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｩc".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｪdefg".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｫ ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｬ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｭ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｮ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｯ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｰ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｱ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｲ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｳ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｴ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｵ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｶ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｷ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｸ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｹ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｺ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｻ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｼ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｽ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｾ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ｿ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾀ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾁ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾂ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾃ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾄ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾅ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾆ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾇ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾈ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾉ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾊ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾋ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾌ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾍ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾎ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾏ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾐ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾑ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾒ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾓ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾔ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾕ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾖ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾗ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾘ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾙ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾚ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾛ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾜ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾝ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾞ".as_bytes()));
        assert!(sc_password_starts_with_katakana("$ﾟ".as_bytes()));

        assert!(!sc_password_starts_with_katakana("$｠".as_bytes()));
        assert!(!sc_password_starts_with_katakana("$ﾠ".as_bytes()));
        assert!(!sc_password_starts_with_katakana("$ ｱｱｱｱｱ".as_bytes()));
    }
}
