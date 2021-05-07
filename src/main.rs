mod decode;
mod encode;

use clap::{App, Arg};
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
    let _ = io::stdin().read_to_end(&mut buf);

    if args.is_present("d") {
        let sanitized: Vec<u8> = buf.into_iter().clone().filter(|b| *b != 10).collect();
        let decoded = &decode::decode(&sanitized);

        match decoded {
            Ok(d) => io::stdout().write_all(&d).unwrap(),
            Err(e) => println!("{}", e),
        }
    } else {
        let encoded = encode::encode(&buf);

        match encoded {
            Ok(e) => io::stdout().write_all(e.as_bytes()).unwrap(),
            Err(e) => println!("{}", e),
        }
    }
}
