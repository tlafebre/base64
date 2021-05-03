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

#[cfg(test)]
mod tests {
    use super::*;
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
}
