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

#[cfg(feature = "i18n")]
use i18n_embed;

mod i18n;
#[cfg(feature = "i18n")]
use i18n::tr; // otherwise, i18n null mod puts a noop tr!() into crate root

use getopts::Options;
use std::env;
use std::io::{self, BufRead, BufReader, Write};
use std::process;
use tripcode::*;

/// Command line arguments
struct Args {
    /// print passwords along with tripcodes prefixed with #
    print_passwords: bool,
    /// print the tripcode prefix ! (or ◆ on JP locale)
    with_prefixes: bool,
}

fn main() {
    i18n_init!();
    let mut args = env::args();
    let program = args.next().unwrap();
    let brief = tr!("Usage: tripcode [options] [--] [passwords]");
    let mut stdout = io::stdout();

    let mut opts = Options::new();
    opts.optopt( "t",  "type",     &tr!("specify the type of tripcodes. defaults to `4chan`"), "{4[chan]|2[ch]|s[c]}")
        .optflag("f",  "filter",   &tr!("read passwords from standard input"))
        .optflag("h",  "help",     &tr!("print this help message and exit"))
        .optflag("p",  "password", &tr!("print passwords along with tripcodes"))
        .optflag("!",  "prefix",   &tr!("print the tripcode prefix !"));

    macro_rules! print_usage {
        ($out:expr) => (write!($out, stringify!({}), opts.usage_with_format(|opts| {format!("{}\n\n{}\n{}\n", brief, tr!("Options:"), opts.collect::<Vec<String>>().join("\n"))})));
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
        Err(f) => fail!("{}", f)
    };

    if matches.opt_present("h") {
        print_usage!(&mut stdout).unwrap();
        return;
    }

    let (opt_f, opt_p, opt_t, opt_pfx) = (
        matches.opt_present("f"),
        matches.opt_present("p"),
        matches.opt_str("t"),
        matches.opt_present("!")
    );
    let mut free = matches.free;

    let mut passwords: Box<dyn Iterator<Item=Vec<u8>>> = Box::new(
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
    (match opt_t.map(|s|{ bind = s; bind.as_str() }).unwrap_or("4chan") {
        "2ch"   | "2" => generate::<Mona,     _, _>,
        "4chan" | "4" => generate::<Fourchan, _, _>,
        "sc"    | "s" => generate::<ScSjis,   _, _>,
        code_type     => {fail!("{} `{}`", tr!("unknown tripcode type"), code_type)},
    })(&mut stdout, passwords, &Args{print_passwords: opt_p, with_prefixes: opt_pfx})
}

fn generate<G, W, I>(dst: &mut W, passwords: I, args: &Args)
    where G: TripcodeGenerator, W: Write, I: Iterator<Item=Vec<u8>>
{
    for p in passwords {
        if args.with_prefixes {
            dst.write_all(tr!("!").as_bytes()).unwrap();
        }
        G::write(&p, dst).unwrap();
        if args.print_passwords {
            dst.write_all(&[b'#']).unwrap();
            dst.write_all(&p).unwrap();
        }
        dst.write_all(&[b'\n']).unwrap();
    }
}
