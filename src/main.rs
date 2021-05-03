// trim newlines
// newline before output

mod decode;
mod encode;

use clap::{App, Arg};
use std::env::args;
use std::fs::File;
use std::io::{self, Read, Write};

fn main() {
    let args = App::new("base64")
        .arg(
            Arg::with_name("d")
                .short("d")
                .long("decode")
                .help("Decodes base64 encoded data"),
        )
        .get_matches();

    let mut buf: Vec<u8> = Vec::new();
    let input = io::stdin().read_to_end(&mut buf);

    if args.is_present("d") {
        let sanitized: Vec<u8> = buf.into_iter().clone().filter(|b| *b != 10).collect();
        io::stdout().write_all(&decode::decode(&sanitized).unwrap());
    } else {
        io::stdout().write_all(encode::encode(&buf).unwrap().as_bytes());
    }
}
