// Copyright 2016 Huton. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(test)]

extern crate test;
extern crate tripcode;

use test::Bencher;
use tripcode::*;
use tripcode::hash::TripcodeHash;

macro_rules! bencher {
    ($name:ident, $passwords:ident, $method:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            b.iter(|| {
                for p in &::samples::$passwords {
                    $method(p);
                }
            });
        }
    };

    ($generate:ident, $append:ident, $hash:ident, $generator:ty, $passwords:ident,
        $m_gen:ident, $m_app:ident, $m_hash:ident
    ) => {
        bencher!($generate, $passwords, <$generator>::$m_gen);
        bencher!($hash, $passwords, <$generator>::$m_hash);

        #[bench]
        fn $append(b: &mut Bencher) {
            let mut tripcode = String::with_capacity(45);

            b.iter(|| {
                for p in &::samples::$passwords {
                    <$generator>::$m_app(p, &mut tripcode);
                    tripcode.clear();
                }
            });
        }
    };

    ($generate:ident, $append:ident, $hash:ident,
        $generator:ty, $passwords:ident
    ) => {
        bencher!($generate, $append, $hash, $generator, $passwords,
            generate, append, hash
        );
    };
}

macro_rules! search_fn {
    ($name:ident, $method:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            let in_list: Vec<_> = ::samples::search::IN_LIST.iter()
                .map(|p| $method(p)).collect();
            let not_in_list: Vec<_> = ::samples::search::NOT_IN_LIST.iter()
                .map(|p| $method(p)).collect();

            b.iter(|| {
                let tripcodes: Vec<_> = ::samples::search::PASSWORDS.iter().map(|p| $method(p)).collect();

                for q in &in_list {
                    assert!(tripcodes.iter().find(|&t| t == q).is_some());
                }

                for q in &not_in_list {
                    assert!(tripcodes.iter().find(|&t| t == q).is_none());
                }
            });
        }
    };

    ($generator:ty, $normal:ident, $hash:ident) => {
        search_fn!($normal, <$generator>::generate);
        search_fn!($hash, <$generator>::hash);
    }
}

bencher!(mona_10, mona_10_append, mona_10_hash,
    Mona10, PASSWORDS_10
);
bencher!(mona_10_nonescaping, mona_10_append_nonescaping, mona_10_hash_nonescaping,
    Mona10Nonescaping, PASSWORDS_10
);
bencher!(mona_raw, mona_raw_append, mona_raw_hash,
    MonaRaw, PASSWORDS_RAW,
    try_generate, try_append, try_hash
);
bencher!(mona_12, mona_12_append, mona_12_hash,
    Mona12, PASSWORDS_12
);
bencher!(mona_12_nonescaping, mona_12_append_nonescaping, mona_12_hash_nonescaping,
    Mona12Nonescaping, PASSWORDS_12
);

search_fn!(Mona, search, search_hash);

macro_rules! bench_encode {
    ($generator:ty, $passwords:ident, $name:ident) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            let hashes: Vec<_> = ::samples::$passwords.iter().map(<$generator>::hash).collect();

            b.iter(|| {
                for &h in &hashes {
                    h.encode();
                }
            })
        }
    }
}

bench_encode!(Mona10, PASSWORDS_10, encode_mona_10_hash);
bench_encode!(Mona12, PASSWORDS_12, encode_mona_12_hash);

mod samples {
    pub const PASSWORDS_10: [&'static str; 10] = [
        r"",
        r"k",
        r"[K",
        r"2 V",
        r"|TB~",
        r"(~A5|",
        r"[6??Pz",
        r"ErdxpJ$",
        r"dib3Q_4x",
        r"R=/IuxM%t",
    ];

    pub const PASSWORDS_RAW: [&'static str; 10] = [
        r"0000000000000000..",
        r"84BD535FA2BB720Ect",
        r"50759C209220D7E4v0",
        r"F6A30D6E803D43D4lp",
        r"97B53106556CE7DA2y",
        r"6C8038390588412DsO",
        r"2B6FA4AFFCB038FE7J",
        r"C75F358B438A058BHd",
        r"D3CB3B895A6F78A1wY",
        r"D2F40002DAD1BB7FQG",
    ];

    pub const PASSWORDS_12: [&'static str; 10] = [
        r"            ",
        r"HEF/fcuwiE$]",
        r"mfQ[8K?$m7eoKD",
        r"{POO{YFdbQFc0^U",
        r"%N/m0#t(w('4v$#-V",
        r"G!-2m%$zgV&-PX*E*rG#",
        r"%xi)DBAQ+zo}dk#tqYL'*H",
        r"31V9)^s6(3ioM%2r8ek,%T6m)",
        r"%&=T0W4F?n_[c~7g46u%&w *$K?_",
        r"%30rEcU:,k*VsT]vVQ=uWoK7L?cyHakr",
    ];

    pub mod search {
        pub const PASSWORDS: [&'static str; 10] = [
            "tripcode",
            "password",
            "twelve bytes",
            "トリップ",
            "酉",
            "AAAAA",
            "あああああ",
            "PASSWORD",
            "999999999",
            "\"\"",
        ];

        pub const IN_LIST: [&'static str; 5] = [
            "tripcode",
            "twelve bytes",
            "酉",
            "あああああ",
            "\"\"",
        ];

        pub const NOT_IN_LIST: [&'static str; 5] = [
            "absent",
            "DoesNotExist",
            "none",
            "nothing",
            "missing",
        ];
    }
}
