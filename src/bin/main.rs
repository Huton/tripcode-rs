// Copyright 2016 Huton. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate getopts;
extern crate tripcode;

use getopts::Options;
use std::env;
use std::error::Error;
use std::io::{self, BufRead, BufReader, Write};
use std::process;
use tripcode::*;

fn main() {
    let mut args = env::args();
    let program = args.next().unwrap();
    let brief = format!("Usage: {} [options] [--] [passwords]", program);
    let mut stdout = io::stdout();

    let mut opts = Options::new();
    opts.optopt( "t", "type",     "specify the type of tripcodes. defaults to `4chan`", "{4[chan]|2[ch]|s[c]}")
        .optflag("f", "filter",   "read passwords from standard input")
        .optflag("h", "help",     "print this help message and exit")
        .optflag("p", "password", "print passwords along with tripcodes");

    macro_rules! print_usage {
        ($out:expr) => (write!($out, "{}", opts.usage(&brief)));
    }

    macro_rules! fail {
        ($($arg:tt)*) => {{
            writeln!(io::stderr(), "{}: {}\n", program, format!($($arg)*)).unwrap();
            print_usage!(io::stderr()).unwrap();
            process::exit(1);
        }};
    }

    let matches = match opts.parse(args.by_ref()) {
        Ok(m) => m,
        Err(f) => fail!("{}", f.description()),
    };

    if matches.opt_present("h") {
        print_usage!(&mut stdout).unwrap();
        return;
    }

    let (opt_f, opt_p, opt_t) = (
        matches.opt_present("f"),
        matches.opt_present("p"),
        matches.opt_str("t")
    );
    let mut free = matches.free;

    let mut passwords: Box<Iterator<Item=Vec<u8>>> = Box::new(
        free.drain(..).chain(args).map(|s| s.into_bytes())
    );
    if opt_f {
        passwords = Box::new(
            passwords.chain(
                BufReader::new(io::stdin())
                    .split(b'\n')
                    .map(|l| l.unwrap())
            )
        );
    }

    let mut bind = String::new();
    match opt_t.map(|s| { bind = s; bind.as_str() }).unwrap_or("4chan") {
        "2ch"   | "2" => generate::<Mona, _, _>(&mut stdout, passwords, opt_p),
        "4chan" | "4" => generate::<Fourchan, _, _>(&mut stdout, passwords, opt_p),
        "sc"    | "s" => generate::<ScSjis, _, _>(&mut stdout, passwords, opt_p),
        code_type     => fail!("unknown tripcode type `{}`", code_type),
    }
}

fn generate<G, W, I>(dst: &mut W, passwords: I, opt_p: bool)
    where G: TripcodeGenerator, W: Write, I: Iterator<Item=Vec<u8>>
{
    if opt_p {
        for p in passwords {
            G::write(&p, dst).unwrap();
            dst.write_all(b"#").unwrap();
            dst.write_all(&p).unwrap();
            dst.write_all(b"\n").unwrap();
        }
    } else {
        for p in passwords {
            G::write(&p, dst).unwrap();
            dst.write_all(b"\n").unwrap();
        }
    }
}
