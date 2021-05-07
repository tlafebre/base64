use std::collections::HashMap;

const BASE64_ALPHABET: [char; 64] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l',
    'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9', '+', '/',
];

fn data_to_octets(data: &[u8]) -> Vec<Vec<u8>> {
    let octets = data
        .chunks(3)
        .map(|c| match c.len() {
            1 => vec![c[0]],
            2 => vec![c[0], c[1]],
            _ => vec![c[0], c[1], c[2]],
        })
        .collect();
    octets
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

pub fn encode(data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    let encoding_table: HashMap<usize, &char> = BASE64_ALPHABET.iter().enumerate().collect();
    let octets = data_to_octets(data);
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

    let base64_string: String = chars
        .into_iter()
        .map(|o| {
            if o.is_some() {
                encoding_table[&usize::from(o.unwrap())]
            } else {
                &'='
            }
        })
        .collect();

    Ok(base64_string)
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

pub fn decode(b64_string: &[u8]) -> Result<Vec<u8>, &str> {
    let decoding_table: HashMap<char, usize> = BASE64_ALPHABET
        .iter()
        .enumerate()
        .map(|(idx, c)| (*c, idx))
        .collect();
    let mut bytes: Vec<Option<u8>> = Vec::new();

    if b64_string.len() % 4 != 0 {
        return Err("Invalid data");
    }

    for byte in b64_string {
        if *byte != b'=' {
            bytes.push(Some(decoding_table[&(*byte as char)] as u8));
        } else {
            bytes.push(None)
        }
    }

    let sextet_groups = base64_bytes_to_sextets(bytes);
    let mut v: Vec<u8> = Vec::new();
    for group in sextet_groups {
        v.push(compose_first_byte(group[0], group[1]));
        if group[2].is_some() {
            v.push(compose_second_byte(group[1], group[2]));
        }
        if group[3].is_some() {
            v.push(compose_third_byte(group[2], group[3]));
        }
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decode_empty_string() {
        assert_eq!(decode(b"").unwrap(), b"");
    }

    #[test]
    fn decode_unpadded() {
        assert_eq!(decode(b"Zm9v").unwrap(), b"foo");
        assert_eq!(decode(b"Zm9vYmFy").unwrap(), b"foobar");
    }

    #[test]
    fn decode_with_double_pad() {
        assert_eq!(decode(b"Zg==").unwrap(), b"f");
        assert_eq!(decode(b"Zm9vYg==").unwrap(), b"foob");
    }

    #[test]
    fn decode_with_single_pad() {
        assert_eq!(decode(b"Zm8=").unwrap(), b"fo");
        assert_eq!(decode(b"Zm9vYmE=").unwrap(), b"fooba");
    }

    #[test]
    fn decode_invalid_input() {
        assert_eq!(decode(b"Zm9vYmE"), Err("Invalid data"));
    }

    #[test]
    fn decode_to_binary_data() {
        assert_eq!(decode(b"MIID").unwrap(), [48, 130, 3]);
    }
}

#[test]
fn test_empty_string_to_octets() {
    let s = String::from("");
    let expected: Vec<Vec<u8>> = vec![];
    assert_eq!(data_to_octets(s.as_bytes()), expected);
}

#[test]
fn test_string_to_octets() {
    let s = String::from("foobar");
    let expected: Vec<Vec<u8>> = vec![vec![102, 111, 111], vec![98, 97, 114]];
    assert_eq!(data_to_octets(s.as_bytes()), expected);
}

#[test]
fn test_string_to_less_than_perfect_octets() {
    let s = String::from("foob");
    let expected: Vec<Vec<u8>> = vec![vec![102, 111, 111], vec![98]];
    assert_eq!(data_to_octets(s.as_bytes()), expected);
}

#[test]
fn encode_empty_string() {
    assert_eq!(encode(String::from("").as_bytes()).unwrap(), "");
}

#[test]
fn encode_unpadded() {
    assert_eq!(encode(String::from("foo").as_bytes()).unwrap(), "Zm9v");
    assert_eq!(encode(String::from("AMT").as_bytes()).unwrap(), "QU1U");
    assert_eq!(
        encode(String::from("foobar").as_bytes()).unwrap(),
        "Zm9vYmFy"
    );
}

#[test]
fn encode_with_double_pad() {
    assert_eq!(encode(String::from("f").as_bytes()).unwrap(), "Zg==");
    assert_eq!(encode(String::from("foob").as_bytes()).unwrap(), "Zm9vYg==");
}

#[test]
fn encode_with_single_pad() {
    assert_eq!(encode(String::from("fo").as_bytes()).unwrap(), "Zm8=");
    assert_eq!(
        encode(String::from("fooba").as_bytes()).unwrap(),
        "Zm9vYmE="
    );
}

#[test]
fn encode_with_newlines() {
    assert_eq!(
        encode(String::from("foo\nbar").as_bytes()).unwrap(),
        "Zm9vCmJhcg=="
    );
}

#[test]
fn encode_binary_data() {
    let binary_data: [u8; 5] = [222, 216, 77, 179, 186];

    assert_eq!(encode(&binary_data[..1]).unwrap(), String::from("3g=="));
    assert_eq!(encode(&binary_data[..2]).unwrap(), String::from("3tg="));
    assert_eq!(encode(&binary_data[..3]).unwrap(), String::from("3thN"));
    assert_eq!(encode(&binary_data[..]).unwrap(), String::from("3thNs7o="));
}
