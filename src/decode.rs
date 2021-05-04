use std::collections::HashMap;

const BASE64_ALPHABET: [char; 64] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l',
    'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9', '+', '/',
];

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

pub fn decode(b64_string: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let decoding_table: HashMap<char, usize> = BASE64_ALPHABET
        .iter()
        .enumerate()
        .map(|(idx, c)| (*c, idx))
        .collect();
    let mut bytes: Vec<Option<u8>> = Vec::new();

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
        v.push(compose_second_byte(group[1], group[2]));
        v.push(compose_third_byte(group[2], group[3]));
    }
    if let Some(n) = v.last() {
        if *n == 0 {
            v.pop();
        }
    }

    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_empty_string() {
        assert_eq!(decode(b"").unwrap(), "");
    }

    #[test]
    fn decode_unpadded() {
        assert_eq!(decode(b"Zm9v").unwrap(), "foo");
        assert_eq!(decode(b"Zm9vYmFy").unwrap(), "foobar");
    }

    #[test]
    fn decode_with_double_pad() {
        assert_eq!(decode(b"Zg==").unwrap(), "f");
        assert_eq!(decode(b"Zm9vYg==").unwrap(), "foob");
    }

    #[test]
    fn decode_with_single_pad() {
        assert_eq!(decode(b"Zm8=").unwrap(), "fo");
        assert_eq!(decode(b"Zm9vYmE=").unwrap(), "fooba");
    }
}
