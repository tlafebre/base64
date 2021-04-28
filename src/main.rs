// move to library
// make it work on stdin

use std::collections::HashMap;

const BASE64_ALPHABET: [char; 64] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l',
    'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9', '+', '-',
];

fn str_to_octets(string: String) -> Vec<Vec<u8>> {
    let octets: Vec<Vec<u8>> = string
        .as_bytes()
        .chunks(3)
        .map(|c| match c.len() {
            1 => vec![c[0]],
            2 => vec![c[0], c[1]],
            _ => vec![c[0], c[1], c[2]],
        })
        .collect();
    octets
}

fn base64_bytes_to_sextets(bytes: Vec<u8>) -> Vec<Vec<Option<u8>>> {
    let mut sextets: Vec<Vec<Option<u8>>> = Vec::new();

    for chunk in bytes.chunks(4) {
        let mut chunk_bytes: Vec<Option<u8>> = Vec::new();
        for byte in chunk {
            chunk_bytes.push(Some(*byte));
        }
        match chunk_bytes.len() {
            4 => (),
            3 => chunk_bytes.push(None),
            2 => chunk_bytes.append(&mut vec![None, None]),
            _ => (),
        }
        sextets.push(chunk_bytes);
    }

    sextets
}

fn octets_to_str(octets: Vec<Vec<u8>>) -> String {
    let mut string = String::new();
    for octet_group in octets {
        let octets = octet_group.chunks(3);
        for octet in octets {
            //println!("{}", std::str::from_utf8(octet).unwrap().to_string());
        }
    }
    string
}

fn extract_first_char_bits(first_byte: u8) -> u8 {
    (0b11111100 & first_byte) >> 2
}

fn extract_second_char_bits(first_byte: u8, second_byte: u8) -> u8 {
    (0b00000011 & first_byte) << 4 | (0b11110000 & second_byte) >> 4
}

fn extract_third_char_bits(second_byte: u8, third_byte: u8) -> u8 {
    (0b00001111 & second_byte) << 2 | (0b11000000 & third_byte) >> 6
}

fn extract_fourth_char_bits(third_byte: u8) -> u8 {
    0b00111111 & third_byte
}

fn compose_first_byte(first_sextet: Option<u8>, second_sextet: Option<u8>) -> u8 {
    (first_sextet.unwrap() & 0b111111) << 2 | (second_sextet.unwrap() & 0b110000) >> 4
}

fn compose_second_byte(second_sextet: Option<u8>, third_sextet: Option<u8>) -> u8 {
    match third_sextet {
        Some(t) => (second_sextet.unwrap() & 0b001111) << 4 | (t & 0b111100) >> 2,
        None => (second_sextet.unwrap() & 0b001111) << 4 | (0b000000) >> 2,
    }
}

fn compose_third_byte(third_sextet: Option<u8>, fourth_sextet: Option<u8>) -> u8 {
    match third_sextet {
        Some(t) => match fourth_sextet {
            Some(f) => (t & 0b000011) << 6 | (f & 0b111111),
            None => (t & 0b000011) << 6 | 0b000000,
        },
        None => return 0b00000000,
    }
}

fn compose_sextets_to_bytes(sextets: Vec<u8>) -> Vec<u32> {
    let bytearray = u32::from(sextets[0]) << 18
        | u32::from(sextets[1]) << 12
        | u32::from(sextets[2]) << 6
        | u32::from(sextets[3]);
    vec![
        (bytearray & 0b111111110000000000000000) >> 16,
        (bytearray & 0b000000001111111100000000) >> 8,
        (bytearray & 0b000000000000000011111111),
    ]
}

fn encode(string: String) -> String {
    let encoding_table: HashMap<usize, &char> = BASE64_ALPHABET.iter().enumerate().collect();
    let octets = str_to_octets(string);
    let mut chars: Vec<u8> = Vec::new();

    for octet_group in octets {
        match octet_group.len() {
            3 => {
                chars.push(extract_first_char_bits(octet_group[0]));
                chars.push(extract_second_char_bits(octet_group[0], octet_group[1]));
                chars.push(extract_third_char_bits(octet_group[1], octet_group[2]));
                chars.push(extract_fourth_char_bits(octet_group[2]));
            }
            2 => {
                chars.push(extract_first_char_bits(octet_group[0]));
                chars.push(extract_second_char_bits(octet_group[0], octet_group[1]));
                chars.push(extract_third_char_bits(octet_group[1], 0));
                chars.push(64);
            }
            _ => {
                chars.push(extract_first_char_bits(octet_group[0]));
                chars.push(extract_second_char_bits(octet_group[0], 0));
                chars.push(64);
                chars.push(64);
            }
        }
    }
    chars
        .into_iter()
        .map(|c| encoding_table[&usize::from(c)])
        .collect::<String>()
}

fn decode(b64_string: String) -> String {
    let decoding_table: HashMap<char, usize> = BASE64_ALPHABET
        .iter()
        .enumerate()
        .map(|(idx, c)| (*c, idx))
        .collect();
    let mut bytes: Vec<u8> = Vec::new();

    for c in b64_string.chars() {
        if c != '=' {
            bytes.push(decoding_table[&c] as u8);
        }
    }

    let sextet_groups = base64_bytes_to_sextets(bytes);
    let mut v: Vec<char> = Vec::new();
    for group in sextet_groups {
        v.push(compose_first_byte(group[0], group[1]) as char);
        v.push(compose_second_byte(group[1], group[2]) as char);
        v.push(compose_third_byte(group[2], group[3]) as char);
    }

    //for sextet_group in base64_bytes_to_sextets(bytes) {
    //    let byte_group = compose_sextets_to_bytes(sextet_group);
    //    for byte in byte_group {
    //        v.push(byte.clone() as u8 as char);
    //    }
    //let u8s: Vec<u8> = byte_group.iter().map(|i| *i as u8).collect();
    //let test: Vec<char> = u8s.iter().map(|n| *n as char).collect();

    //let word: String = test.iter().collect();
    //println!("{:?}", word);

    //string.push_str(&byte.to_string());

    //string.push_str(&std::str::from_utf8(&byte_group[..]).unwrap().to_string());
    //}

    v.iter().collect::<String>()
}

fn main() {
    //println!("{:?}", compose_first_byte(25, 32));
    //println!("{:?}", compose_second_byte(32, None));
    //println!("{:?}", compose_third_byte(Some(61), Some(47)));

    //println!("{:?}", compose_first_byte(25, 38));
    //println!("{:#b}", compose_second_byte(38, None));
    let s = String::from("Zm9vYmFy");
    println!("{}", decode(s));
    let s2 = String::from("Zm9vYmE=");
    println!("{}", decode(s2));
    let s3 = String::from("Zg==");
    println!("{}", decode(s3));
    let s4 = String::from("VGplZXJkIGlzIGRlIGJlc3RlIHJ1c3QgcHJvZ3JhbW1ldXI=");
    println!("{}", decode(s4));

    //println!("{}", decode(s));
    //println!(
    //    "{:?}",
    //    base64_bytes_to_sextets(vec![25, 38, 61, 47, 24, 38, 5, 50])
    //);
    //println!("{:?}", compose_sextets_to_bytes(vec![25, 38, 61, 47]));

    //println!("{:?}", &std::str::from_utf8(&[65, 66, 84]));

    //assert_eq!(encode(String::from("")), "");
    //assert_eq!(encode(String::from("f")), "Zg==");
    //assert_eq!(encode(String::from("fo")), "Zm8=");
    //assert_eq!(encode(String::from("foo")), "Zm9v");
    //assert_eq!(encode(String::from("foob")), "Zm9vYg==");
    //assert_eq!(encode(String::from("fooba")), "Zm9vYmE=");
    //assert_eq!(encode(String::from("foobar")), "Zm9vYmFy");
}
