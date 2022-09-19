#![cfg(test)]

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
    let tripcode = Des::generate("password", b'a', b's');
    assert_eq!("ozOtJW9BFA", &tripcode);
    let tripcode = Des::generate("", b'H', b'.');
    assert_eq!("jPpg5.obl6", &tripcode);
}

#[test]
fn append() {
    let mut tripcode = String::new();

    Mona10::append("a", &mut tripcode);
    Mona10::append("b", &mut tripcode);
    Mona10::append("c", &mut tripcode);
    Mona12::append("0123456789ab", &mut tripcode);
    Mona::append("##0123456789", &mut tripcode);
    let k = SJIS.encode("$｡1008343131", EncoderTrap::Strict).unwrap();
    ScKatakana::append(&k, &mut tripcode);
    Sc15::append("$a9876543210", &mut tripcode);
    Des::append("de", b'e', b'H', &mut tripcode);

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
