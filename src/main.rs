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

fn base64_bytes_to_sextets(bytes: Vec<Option<u8>>) -> Vec<Vec<Option<u8>>> {
    let mut sextets: Vec<Vec<Option<u8>>> = Vec::new();

    for chunk in bytes.chunks(4) {
        let mut chunk_bytes: Vec<Option<u8>> = Vec::new();
        for byte in chunk {
            if let Some(b) = byte {
                chunk_bytes.push(Some(*b));
            } else {
                chunk_bytes.push(None);
            }
        }
        sextets.push(chunk_bytes);
    }

    sextets
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

fn encode(string: String) -> Result<String, Box<dyn std::error::Error>> {
    let encoding_table: HashMap<usize, &char> = BASE64_ALPHABET.iter().enumerate().collect();
    let octets = str_to_octets(string);
    let mut chars: Vec<Option<u8>> = Vec::new();

    for og in octets {
        match og.len() {
            3 => {
                chars.push(Some(extract_first_char_bits(og[0])));
                chars.push(Some(extract_second_char_bits(og[0], og[1])));
                chars.push(Some(extract_third_char_bits(og[1], og[2])));
                chars.push(Some(extract_fourth_char_bits(og[2])));
            }
            2 => {
                chars.push(Some(extract_first_char_bits(og[0])));
                chars.push(Some(extract_second_char_bits(og[0], og[1])));
                chars.push(Some(extract_third_char_bits(og[1], 0)));
                chars.append(&mut vec![None]);
            }
            _ => {
                chars.push(Some(extract_first_char_bits(og[0])));
                chars.push(Some(extract_second_char_bits(og[0], 0)));
                chars.append(&mut vec![None, None]);
            }
        }
    }
    let string = chars
        .into_iter()
        .map(|o| {
            if o.is_some() {
                encoding_table[&usize::from(o.unwrap())]
            } else {
                &'='
            }
        })
        .collect::<String>();
    Ok(string)
}

fn decode(b64_string: String) -> Result<String, Box<dyn std::error::Error>> {
    let decoding_table: HashMap<char, usize> = BASE64_ALPHABET
        .iter()
        .enumerate()
        .map(|(idx, c)| (*c, idx))
        .collect();
    let mut bytes: Vec<Option<u8>> = Vec::new();

    for c in b64_string.chars() {
        if c != '=' {
            bytes.push(Some(decoding_table[&c] as u8));
        } else {
            bytes.push(None)
        }
    }

    let sextet_groups = base64_bytes_to_sextets(bytes);
    let mut v: Vec<u8> = Vec::new();
    for group in sextet_groups {
        v.push(compose_first_byte(group[0], group[1]));
        v.push(compose_second_byte(group[1], group[2]));
        v.push(compose_third_byte(group[2], group[3]));
    }

    Ok(std::str::from_utf8(v.as_slice())?
        .trim_matches(char::from(0))
        .to_string())
}

fn main() {
    assert_eq!(encode(String::from("")).unwrap(), "");
    assert_eq!(encode(String::from("f")).unwrap(), "Zg==");
    assert_eq!(encode(String::from("fo")).unwrap(), "Zm8=");
    assert_eq!(encode(String::from("foo")).unwrap(), "Zm9v");
    assert_eq!(encode(String::from("foob")).unwrap(), "Zm9vYg==");
    assert_eq!(encode(String::from("fooba")).unwrap(), "Zm9vYmE=");
    assert_eq!(encode(String::from("foobar")).unwrap(), "Zm9vYmFy");

    assert_eq!(decode(String::from("")).unwrap(), "");
    assert_eq!(decode(String::from("Zg==")).unwrap(), "f");
    assert_eq!(decode(String::from("Zm8=")).unwrap(), "fo");
    assert_eq!(decode(String::from("Zm9v")).unwrap(), "foo");
    assert_eq!(decode(String::from("Zm9vYg==")).unwrap(), "foob");
    assert_eq!(decode(String::from("Zm9vYmE=")).unwrap(), "fooba");
    assert_eq!(decode(String::from("Zm9vYmFy")).unwrap(), "foobar");
}
