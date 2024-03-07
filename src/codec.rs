use itertools::Itertools as _;

const ZWC: &[char] = &[
    '\u{200c}', '\u{200d}', '\u{2061}', '\u{2062}', '\u{2063}', '\u{2064}',
];

const _: () = assert!(
    ZWC.len() % 2 == 0,
    "ZWC must contain an even number of characters"
);

const BASE: usize = 4;

fn is_zwc(c: char) -> bool {
    ZWC.contains(&c)
}

pub fn encode(data: &[u8]) -> String {
    // 4 chars per byte
    let required_zwc = data.len() * BASE;
    let mut string_buffer = String::with_capacity(required_zwc);
    let mut buffer = Vec::with_capacity(required_zwc);
    // LE
    for &byte in data {
        for b in (0..=6).step_by(2) {
            let bit1 = (byte >> b) & 0b1;
            let bit2 = (byte >> (b + 1)) & 0b1;

            let zwc_char = match (bit2, bit1) {
                (0, 0) => ZWC[0],
                (0, 1) => ZWC[1],
                (1, 0) => ZWC[2],
                (1, 1) => ZWC[3],
                _ => unreachable!(),
            };

            buffer.push(zwc_char);
        }
    }

    // now it's encoded, but we can compress it a little further

    // determine most common 2 characters in string
    let mut counts = [0; BASE];
    for &c in buffer.iter() {
        match c {
            _ if c == ZWC[0] => counts[0] += 1,
            _ if c == ZWC[1] => counts[1] += 1,
            _ if c == ZWC[2] => counts[2] += 1,
            _ if c == ZWC[3] => counts[3] += 1,
            _ => unreachable!(),
        }
    }

    // first common
    let mut working_buffer = String::from_iter(&buffer);
    if let Some(i) = counts.iter().position_max() {
        string_buffer.push(ZWC[i]);
        // reset counter so we can get the next highest
        counts[i] = 0;

        let zwc_char = ZWC[i];

        let mut tmp = [0u8; 4];
        let zwc_char_rep = ZWC[BASE].encode_utf8(&mut tmp);

        working_buffer = working_buffer.replace(&format!("{zwc_char}{zwc_char}"), zwc_char_rep);
    } else {
        // use BASE + 1 as a sentinel since it's dynamic and unreplaceable
        string_buffer.push(ZWC[BASE + 1]);
    }

    // second common
    if let Some(i) = counts.iter().position_max() {
        string_buffer.push(ZWC[i]);
        counts[i] = 0;

        let zwc_char = ZWC[i];

        let mut tmp = [0u8; 4];
        let zwc_char_rep = ZWC[BASE + 1].encode_utf8(&mut tmp);

        working_buffer = working_buffer.replace(&format!("{zwc_char}{zwc_char}"), zwc_char_rep);
    } else {
        // use BASE + 1 as a sentinel since it's dynamic and unreplaceable
        string_buffer.push(ZWC[BASE + 1]);
    }

    string_buffer.push_str(&working_buffer);

    string_buffer
}

pub fn decode(string: &str) -> Result<Vec<u8>, CodecError> {
    //let data = Vec::with_capacity(string.len().div_ceil(4));
    //let mut buffer = Vec::new();

    let Some(mut pos) = string.find(' ') else {
        return Err(CodecError::ZwcDataNotfound);
    };

    // skip space
    pos += 1;

    // minimum pos + 2 positional chars needed (each char is 3 bytes long)
    if string.len() - 1 <= pos + 6 {
        return Err(CodecError::MalformedData);
    }

    let mut string = string[pos..].to_owned();

    //
    // process first 2 dyn chars
    //
    let mut chars = string.chars();
    let Some((dyn1, dyn2)) = chars.next_tuple() else {
        return Err(CodecError::MalformedData);
    };

    let dyns = &ZWC[BASE..];

    if !dyns.contains(&dyn1) {
        string = string.replace(ZWC[BASE], &format!("{dyn1}{dyn1}"));
    }

    if !dyns.contains(&dyn2) {
        string = string.replace(ZWC[BASE + 1], &format!("{dyn2}{dyn2}"));
    }

    // slice of rest of the string data, ignoring the first 2 pos chars
    let chars = string.chars().skip(2).chunks(4);

    let mut bytes = Vec::with_capacity(string.len().div_ceil(BASE));
    for mut chunk in chars.into_iter() {
        let Some((one, two, three, four)) = chunk.next_tuple() else {
            break;
        };

        // detect end of stream, since all chars are not zwc anymore
        if !is_zwc(one) && !is_zwc(two) && !is_zwc(three) && !is_zwc(four) {
            break;
        }

        // however, if any others are not zwc, our stream is broken
        if !(is_zwc(one) && is_zwc(two) && is_zwc(three) && is_zwc(four)) {
            return Err(CodecError::MalformedData);
        }

        let mut byte = 0u8;

        for (b, c) in (0..=6).step_by(2).zip([one, two, three, four]) {
            let bit_pattern = match c {
                _ if c == ZWC[0] => 0b00u8,
                _ if c == ZWC[1] => 0b01u8,
                _ if c == ZWC[2] => 0b10u8,
                _ if c == ZWC[3] => 0b11u8,
                _ => unreachable!(),
            };

            byte |= bit_pattern << b;
        }

        bytes.push(byte);
    }

    Ok(bytes)
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum CodecError {
    #[error("Malformed decode stream")]
    MalformedData,
    #[error("Stream contains no zwc data")]
    ZwcDataNotfound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        let result = "\u{2062}\u{2061}\u{200c}\u{200c}\u{200c}\u{200c}\u{200d}\u{200c}\u{200c}\u{200c}\u{2061}\u{200c}\u{200c}\u{200c}\u{2062}\u{200c}\u{200c}\u{200c}\u{200c}\u{200d}\u{200c}\u{200c}\u{200d}\u{200d}\u{200c}\u{200c}\u{2061}\u{200d}\u{200c}\u{200c}\u{2062}\u{200d}\u{200c}\u{200c}\u{200c}\u{2061}\u{200c}\u{200c}\u{200d}\u{2061}\u{200c}\u{200c}\u{2064}\u{200c}\u{200c}\u{2062}\u{2061}\u{200c}\u{200c}\u{200c}\u{2062}\u{200c}\u{200c}\u{200d}\u{2062}\u{200c}\u{200c}\u{2061}\u{2062}\u{200c}\u{200c}\u{2063}\u{200c}\u{200c}\u{200c}\u{200c}\u{200d}\u{200c}\u{200d}\u{200c}\u{200d}\u{200c}\u{2061}\u{200c}\u{200d}\u{200c}\u{2062}\u{200c}\u{200d}\u{200c}\u{200c}\u{200d}\u{200d}\u{200c}\u{200d}\u{200d}\u{200d}\u{200c}\u{2061}\u{200d}\u{200d}\u{200c}\u{2062}\u{200d}\u{200d}\u{200c}\u{200c}\u{2061}\u{200d}\u{200c}\u{200d}\u{2061}\u{200d}\u{200c}\u{2064}\u{200d}\u{200c}\u{2062}\u{2061}\u{200d}\u{200c}\u{200c}\u{2062}\u{200d}\u{200c}\u{200d}\u{2062}\u{200d}\u{200c}\u{2061}\u{2062}\u{200d}\u{200c}\u{2063}\u{200d}\u{200c}\u{200c}\u{200c}\u{2061}\u{200c}\u{200d}\u{200c}\u{2061}\u{200c}\u{2061}\u{200c}\u{2061}\u{200c}\u{2062}\u{200c}\u{2061}\u{200c}\u{200c}\u{200d}\u{2061}\u{200c}\u{200d}\u{200d}\u{2061}\u{200c}\u{2061}\u{200d}\u{2061}\u{200c}\u{2062}\u{200d}\u{2061}\u{200c}\u{200c}\u{2064}\u{200c}\u{200d}\u{2064}\u{200c}\u{2064}\u{2061}\u{200c}\u{2062}\u{2064}\u{200c}\u{200c}\u{2062}\u{2061}\u{200c}\u{200d}\u{2062}\u{2061}\u{200c}\u{2061}\u{2062}\u{2061}\u{200c}\u{2063}\u{2061}\u{200c}\u{200c}\u{200c}\u{2062}\u{200c}\u{200d}\u{200c}\u{2062}\u{200c}\u{2061}\u{200c}\u{2062}\u{200c}\u{2062}\u{200c}\u{2062}\u{200c}\u{200c}\u{200d}\u{2062}\u{200c}\u{200d}\u{200d}\u{2062}\u{200c}\u{2061}\u{200d}\u{2062}\u{200c}\u{2062}\u{200d}\u{2062}\u{200c}\u{200c}\u{2061}\u{2062}\u{200c}\u{200d}\u{2061}\u{2062}\u{200c}\u{2064}\u{2062}\u{200c}\u{2062}\u{2061}\u{2062}\u{200c}\u{200c}\u{2063}\u{200c}\u{200d}\u{2063}\u{200c}\u{2061}\u{2063}\u{200c}\u{2063}\u{2062}\u{200c}\u{200c}\u{200c}\u{200c}\u{200d}\u{200d}\u{200c}\u{200c}\u{200d}\u{2061}\u{200c}\u{200c}\u{200d}\u{2062}\u{200c}\u{200c}\u{200d}\u{200c}\u{200d}\u{200c}\u{200d}\u{200d}\u{200d}\u{200c}\u{200d}\u{2061}\u{200d}\u{200c}\u{200d}\u{2062}\u{200d}\u{200c}\u{200d}\u{200c}\u{2061}\u{200c}\u{200d}\u{200d}\u{2061}\u{200c}\u{200d}\u{2064}\u{200c}\u{200d}\u{2062}\u{2061}\u{200c}\u{200d}\u{200c}\u{2062}\u{200c}\u{200d}\u{200d}\u{2062}\u{200c}\u{200d}\u{2061}\u{2062}\u{200c}\u{200d}\u{2063}\u{200c}\u{200d}\u{200c}\u{200c}\u{200d}\u{200d}\u{200d}\u{200c}\u{200d}\u{200d}\u{2061}\u{200c}\u{200d}\u{200d}\u{2062}\u{200c}\u{200d}\u{200d}\u{200c}\u{200d}\u{200d}\u{200d}\u{200d}\u{200d}\u{200d}\u{200d}\u{2061}\u{200d}\u{200d}\u{200d}\u{2062}\u{200d}\u{200d}\u{200d}\u{200c}\u{2061}\u{200d}\u{200d}\u{200d}\u{2061}\u{200d}\u{200d}\u{2064}\u{200d}\u{200d}\u{2062}\u{2061}\u{200d}\u{200d}\u{200c}\u{2062}\u{200d}\u{200d}\u{200d}\u{2062}\u{200d}\u{200d}\u{2061}\u{2062}\u{200d}\u{200d}\u{2063}\u{200d}\u{200d}\u{200c}\u{200c}\u{2061}\u{200d}\u{200d}\u{200c}\u{2061}\u{200d}\u{2061}\u{200c}\u{2061}\u{200d}\u{2062}\u{200c}\u{2061}\u{200d}\u{200c}\u{200d}\u{2061}\u{200d}\u{200d}\u{200d}\u{2061}\u{200d}\u{2061}\u{200d}\u{2061}\u{200d}\u{2062}\u{200d}\u{2061}\u{200d}\u{200c}\u{2064}\u{200d}\u{200d}\u{2064}\u{200d}\u{2064}\u{2061}\u{200d}\u{2062}\u{2064}\u{200d}\u{200c}\u{2062}\u{2061}\u{200d}\u{200d}\u{2062}\u{2061}\u{200d}\u{2061}\u{2062}\u{2061}\u{200d}\u{2063}\u{2061}\u{200d}\u{200c}\u{200c}\u{2062}\u{200d}\u{200d}\u{200c}\u{2062}\u{200d}\u{2061}\u{200c}\u{2062}\u{200d}\u{2062}\u{200c}\u{2062}\u{200d}\u{200c}\u{200d}\u{2062}\u{200d}\u{200d}\u{200d}\u{2062}\u{200d}\u{2061}\u{200d}\u{2062}\u{200d}\u{2062}\u{200d}\u{2062}\u{200d}\u{200c}\u{2061}\u{2062}\u{200d}\u{200d}\u{2061}\u{2062}\u{200d}\u{2064}\u{2062}\u{200d}\u{2062}\u{2061}\u{2062}\u{200d}\u{200c}\u{2063}\u{200d}\u{200d}\u{2063}\u{200d}\u{2061}\u{2063}\u{200d}\u{2063}\u{2062}\u{200d}\u{200c}\u{200c}\u{200c}\u{2061}\u{200d}\u{200c}\u{200c}\u{2064}\u{200c}\u{200c}\u{2061}\u{2062}\u{200c}\u{200c}\u{2061}\u{200c}\u{200d}\u{200c}\u{2061}\u{200d}\u{200d}\u{200c}\u{2064}\u{200d}\u{200c}\u{2061}\u{2062}\u{200d}\u{200c}\u{2061}\u{200c}\u{2061}\u{200c}\u{2061}\u{200d}\u{2061}\u{200c}\u{2064}\u{2061}\u{200c}\u{2061}\u{2062}\u{2061}\u{200c}\u{2061}\u{200c}\u{2062}\u{200c}\u{2061}\u{200d}\u{2062}\u{200c}\u{2064}\u{2062}\u{200c}\u{2061}\u{2063}\u{200c}\u{2061}\u{200c}\u{200c}\u{200d}\u{2061}\u{200d}\u{200c}\u{200d}\u{2064}\u{200c}\u{200d}\u{2061}\u{2062}\u{200c}\u{200d}\u{2061}\u{200c}\u{200d}\u{200d}\u{2061}\u{200d}\u{200d}\u{200d}\u{2064}\u{200d}\u{200d}\u{2061}\u{2062}\u{200d}\u{200d}\u{2061}\u{200c}\u{2061}\u{200d}\u{2061}\u{200d}\u{2061}\u{200d}\u{2064}\u{2061}\u{200d}\u{2061}\u{2062}\u{2061}\u{200d}\u{2061}\u{200c}\u{2062}\u{200d}\u{2061}\u{200d}\u{2062}\u{200d}\u{2064}\u{2062}\u{200d}\u{2061}\u{2063}\u{200d}\u{2061}\u{200c}\u{200c}\u{2064}\u{200d}\u{200c}\u{2064}\u{2061}\u{200c}\u{2064}\u{2062}\u{200c}\u{2064}\u{200c}\u{200d}\u{2064}\u{200d}\u{200d}\u{2064}\u{2061}\u{200d}\u{2064}\u{2062}\u{200d}\u{2064}\u{200c}\u{2064}\u{2061}\u{200d}\u{2064}\u{2064}\u{2064}\u{2061}\u{2062}\u{2064}\u{2061}\u{200c}\u{2062}\u{2064}\u{200d}\u{2062}\u{2064}\u{2061}\u{2062}\u{2064}\u{2063}\u{2064}\u{200c}\u{200c}\u{2062}\u{2061}\u{200d}\u{200c}\u{2062}\u{2064}\u{200c}\u{2062}\u{2061}\u{2062}\u{200c}\u{2062}\u{2061}\u{200c}\u{200d}\u{2062}\u{2061}\u{200d}\u{200d}\u{2062}\u{2064}\u{200d}\u{2062}\u{2061}\u{2062}\u{200d}\u{2062}\u{2061}\u{200c}\u{2061}\u{2062}\u{2061}\u{200d}\u{2061}\u{2062}\u{2064}\u{2061}\u{2062}\u{2061}\u{2062}\u{2061}\u{2062}\u{2061}\u{200c}\u{2063}\u{2061}\u{200d}\u{2063}\u{2064}\u{2063}\u{2061}\u{2063}\u{2062}\u{2061}\u{200c}\u{200c}\u{200c}\u{2062}\u{200d}\u{200c}\u{200c}\u{2062}\u{2061}\u{200c}\u{200c}\u{2063}\u{200c}\u{200c}\u{2062}\u{200c}\u{200d}\u{200c}\u{2062}\u{200d}\u{200d}\u{200c}\u{2062}\u{2061}\u{200d}\u{200c}\u{2063}\u{200d}\u{200c}\u{2062}\u{200c}\u{2061}\u{200c}\u{2062}\u{200d}\u{2061}\u{200c}\u{2062}\u{2064}\u{200c}\u{2063}\u{2061}\u{200c}\u{2062}\u{200c}\u{2062}\u{200c}\u{2062}\u{200d}\u{2062}\u{200c}\u{2062}\u{2061}\u{2062}\u{200c}\u{2063}\u{2062}\u{200c}\u{2062}\u{200c}\u{200c}\u{200d}\u{2062}\u{200d}\u{200c}\u{200d}\u{2062}\u{2061}\u{200c}\u{200d}\u{2063}\u{200c}\u{200d}\u{2062}\u{200c}\u{200d}\u{200d}\u{2062}\u{200d}\u{200d}\u{200d}\u{2062}\u{2061}\u{200d}\u{200d}\u{2063}\u{200d}\u{200d}\u{2062}\u{200c}\u{2061}\u{200d}\u{2062}\u{200d}\u{2061}\u{200d}\u{2062}\u{2064}\u{200d}\u{2063}\u{2061}\u{200d}\u{2062}\u{200c}\u{2062}\u{200d}\u{2062}\u{200d}\u{2062}\u{200d}\u{2062}\u{2061}\u{2062}\u{200d}\u{2063}\u{2062}\u{200d}\u{2062}\u{200c}\u{200c}\u{2061}\u{2062}\u{200d}\u{200c}\u{2061}\u{2062}\u{2061}\u{200c}\u{2061}\u{2063}\u{200c}\u{2061}\u{2062}\u{200c}\u{200d}\u{2061}\u{2062}\u{200d}\u{200d}\u{2061}\u{2062}\u{2061}\u{200d}\u{2061}\u{2063}\u{200d}\u{2061}\u{2062}\u{200c}\u{2064}\u{2062}\u{200d}\u{2064}\u{2062}\u{2064}\u{2061}\u{2063}\u{2064}\u{2062}\u{200c}\u{2062}\u{2061}\u{2062}\u{200d}\u{2062}\u{2061}\u{2062}\u{2061}\u{2062}\u{2061}\u{2063}\u{2062}\u{2061}\u{2062}\u{200c}\u{200c}\u{2063}\u{200d}\u{200c}\u{2063}\u{2061}\u{200c}\u{2063}\u{2062}\u{200c}\u{2063}\u{200c}\u{200d}\u{2063}\u{200d}\u{200d}\u{2063}\u{2061}\u{200d}\u{2063}\u{2062}\u{200d}\u{2063}\u{200c}\u{2061}\u{2063}\u{200d}\u{2061}\u{2063}\u{2064}\u{2063}\u{2062}\u{2061}\u{2063}\u{200c}\u{2063}\u{2062}\u{200d}\u{2063}\u{2062}\u{2061}\u{2063}\u{2063}\u{2063}\u{2062}";

        // 0-255 vec
        let data = (0..=255).collect::<Vec<_>>();

        let data = encode(&data);
        assert_eq!(data, result);
    }

    #[test]
    fn test_decode() {
        let input = "cover \u{2062}\u{2061}\u{200c}\u{200c}\u{200c}\u{200c}\u{200d}\u{200c}\u{200c}\u{200c}\u{2061}\u{200c}\u{200c}\u{200c}\u{2062}\u{200c}\u{200c}\u{200c}\u{200c}\u{200d}\u{200c}\u{200c}\u{200d}\u{200d}\u{200c}\u{200c}\u{2061}\u{200d}\u{200c}\u{200c}\u{2062}\u{200d}\u{200c}\u{200c}\u{200c}\u{2061}\u{200c}\u{200c}\u{200d}\u{2061}\u{200c}\u{200c}\u{2064}\u{200c}\u{200c}\u{2062}\u{2061}\u{200c}\u{200c}\u{200c}\u{2062}\u{200c}\u{200c}\u{200d}\u{2062}\u{200c}\u{200c}\u{2061}\u{2062}\u{200c}\u{200c}\u{2063}\u{200c}\u{200c}\u{200c}\u{200c}\u{200d}\u{200c}\u{200d}\u{200c}\u{200d}\u{200c}\u{2061}\u{200c}\u{200d}\u{200c}\u{2062}\u{200c}\u{200d}\u{200c}\u{200c}\u{200d}\u{200d}\u{200c}\u{200d}\u{200d}\u{200d}\u{200c}\u{2061}\u{200d}\u{200d}\u{200c}\u{2062}\u{200d}\u{200d}\u{200c}\u{200c}\u{2061}\u{200d}\u{200c}\u{200d}\u{2061}\u{200d}\u{200c}\u{2064}\u{200d}\u{200c}\u{2062}\u{2061}\u{200d}\u{200c}\u{200c}\u{2062}\u{200d}\u{200c}\u{200d}\u{2062}\u{200d}\u{200c}\u{2061}\u{2062}\u{200d}\u{200c}\u{2063}\u{200d}\u{200c}\u{200c}\u{200c}\u{2061}\u{200c}\u{200d}\u{200c}\u{2061}\u{200c}\u{2061}\u{200c}\u{2061}\u{200c}\u{2062}\u{200c}\u{2061}\u{200c}\u{200c}\u{200d}\u{2061}\u{200c}\u{200d}\u{200d}\u{2061}\u{200c}\u{2061}\u{200d}\u{2061}\u{200c}\u{2062}\u{200d}\u{2061}\u{200c}\u{200c}\u{2064}\u{200c}\u{200d}\u{2064}\u{200c}\u{2064}\u{2061}\u{200c}\u{2062}\u{2064}\u{200c}\u{200c}\u{2062}\u{2061}\u{200c}\u{200d}\u{2062}\u{2061}\u{200c}\u{2061}\u{2062}\u{2061}\u{200c}\u{2063}\u{2061}\u{200c}\u{200c}\u{200c}\u{2062}\u{200c}\u{200d}\u{200c}\u{2062}\u{200c}\u{2061}\u{200c}\u{2062}\u{200c}\u{2062}\u{200c}\u{2062}\u{200c}\u{200c}\u{200d}\u{2062}\u{200c}\u{200d}\u{200d}\u{2062}\u{200c}\u{2061}\u{200d}\u{2062}\u{200c}\u{2062}\u{200d}\u{2062}\u{200c}\u{200c}\u{2061}\u{2062}\u{200c}\u{200d}\u{2061}\u{2062}\u{200c}\u{2064}\u{2062}\u{200c}\u{2062}\u{2061}\u{2062}\u{200c}\u{200c}\u{2063}\u{200c}\u{200d}\u{2063}\u{200c}\u{2061}\u{2063}\u{200c}\u{2063}\u{2062}\u{200c}\u{200c}\u{200c}\u{200c}\u{200d}\u{200d}\u{200c}\u{200c}\u{200d}\u{2061}\u{200c}\u{200c}\u{200d}\u{2062}\u{200c}\u{200c}\u{200d}\u{200c}\u{200d}\u{200c}\u{200d}\u{200d}\u{200d}\u{200c}\u{200d}\u{2061}\u{200d}\u{200c}\u{200d}\u{2062}\u{200d}\u{200c}\u{200d}\u{200c}\u{2061}\u{200c}\u{200d}\u{200d}\u{2061}\u{200c}\u{200d}\u{2064}\u{200c}\u{200d}\u{2062}\u{2061}\u{200c}\u{200d}\u{200c}\u{2062}\u{200c}\u{200d}\u{200d}\u{2062}\u{200c}\u{200d}\u{2061}\u{2062}\u{200c}\u{200d}\u{2063}\u{200c}\u{200d}\u{200c}\u{200c}\u{200d}\u{200d}\u{200d}\u{200c}\u{200d}\u{200d}\u{2061}\u{200c}\u{200d}\u{200d}\u{2062}\u{200c}\u{200d}\u{200d}\u{200c}\u{200d}\u{200d}\u{200d}\u{200d}\u{200d}\u{200d}\u{200d}\u{2061}\u{200d}\u{200d}\u{200d}\u{2062}\u{200d}\u{200d}\u{200d}\u{200c}\u{2061}\u{200d}\u{200d}\u{200d}\u{2061}\u{200d}\u{200d}\u{2064}\u{200d}\u{200d}\u{2062}\u{2061}\u{200d}\u{200d}\u{200c}\u{2062}\u{200d}\u{200d}\u{200d}\u{2062}\u{200d}\u{200d}\u{2061}\u{2062}\u{200d}\u{200d}\u{2063}\u{200d}\u{200d}\u{200c}\u{200c}\u{2061}\u{200d}\u{200d}\u{200c}\u{2061}\u{200d}\u{2061}\u{200c}\u{2061}\u{200d}\u{2062}\u{200c}\u{2061}\u{200d}\u{200c}\u{200d}\u{2061}\u{200d}\u{200d}\u{200d}\u{2061}\u{200d}\u{2061}\u{200d}\u{2061}\u{200d}\u{2062}\u{200d}\u{2061}\u{200d}\u{200c}\u{2064}\u{200d}\u{200d}\u{2064}\u{200d}\u{2064}\u{2061}\u{200d}\u{2062}\u{2064}\u{200d}\u{200c}\u{2062}\u{2061}\u{200d}\u{200d}\u{2062}\u{2061}\u{200d}\u{2061}\u{2062}\u{2061}\u{200d}\u{2063}\u{2061}\u{200d}\u{200c}\u{200c}\u{2062}\u{200d}\u{200d}\u{200c}\u{2062}\u{200d}\u{2061}\u{200c}\u{2062}\u{200d}\u{2062}\u{200c}\u{2062}\u{200d}\u{200c}\u{200d}\u{2062}\u{200d}\u{200d}\u{200d}\u{2062}\u{200d}\u{2061}\u{200d}\u{2062}\u{200d}\u{2062}\u{200d}\u{2062}\u{200d}\u{200c}\u{2061}\u{2062}\u{200d}\u{200d}\u{2061}\u{2062}\u{200d}\u{2064}\u{2062}\u{200d}\u{2062}\u{2061}\u{2062}\u{200d}\u{200c}\u{2063}\u{200d}\u{200d}\u{2063}\u{200d}\u{2061}\u{2063}\u{200d}\u{2063}\u{2062}\u{200d}\u{200c}\u{200c}\u{200c}\u{2061}\u{200d}\u{200c}\u{200c}\u{2064}\u{200c}\u{200c}\u{2061}\u{2062}\u{200c}\u{200c}\u{2061}\u{200c}\u{200d}\u{200c}\u{2061}\u{200d}\u{200d}\u{200c}\u{2064}\u{200d}\u{200c}\u{2061}\u{2062}\u{200d}\u{200c}\u{2061}\u{200c}\u{2061}\u{200c}\u{2061}\u{200d}\u{2061}\u{200c}\u{2064}\u{2061}\u{200c}\u{2061}\u{2062}\u{2061}\u{200c}\u{2061}\u{200c}\u{2062}\u{200c}\u{2061}\u{200d}\u{2062}\u{200c}\u{2064}\u{2062}\u{200c}\u{2061}\u{2063}\u{200c}\u{2061}\u{200c}\u{200c}\u{200d}\u{2061}\u{200d}\u{200c}\u{200d}\u{2064}\u{200c}\u{200d}\u{2061}\u{2062}\u{200c}\u{200d}\u{2061}\u{200c}\u{200d}\u{200d}\u{2061}\u{200d}\u{200d}\u{200d}\u{2064}\u{200d}\u{200d}\u{2061}\u{2062}\u{200d}\u{200d}\u{2061}\u{200c}\u{2061}\u{200d}\u{2061}\u{200d}\u{2061}\u{200d}\u{2064}\u{2061}\u{200d}\u{2061}\u{2062}\u{2061}\u{200d}\u{2061}\u{200c}\u{2062}\u{200d}\u{2061}\u{200d}\u{2062}\u{200d}\u{2064}\u{2062}\u{200d}\u{2061}\u{2063}\u{200d}\u{2061}\u{200c}\u{200c}\u{2064}\u{200d}\u{200c}\u{2064}\u{2061}\u{200c}\u{2064}\u{2062}\u{200c}\u{2064}\u{200c}\u{200d}\u{2064}\u{200d}\u{200d}\u{2064}\u{2061}\u{200d}\u{2064}\u{2062}\u{200d}\u{2064}\u{200c}\u{2064}\u{2061}\u{200d}\u{2064}\u{2064}\u{2064}\u{2061}\u{2062}\u{2064}\u{2061}\u{200c}\u{2062}\u{2064}\u{200d}\u{2062}\u{2064}\u{2061}\u{2062}\u{2064}\u{2063}\u{2064}\u{200c}\u{200c}\u{2062}\u{2061}\u{200d}\u{200c}\u{2062}\u{2064}\u{200c}\u{2062}\u{2061}\u{2062}\u{200c}\u{2062}\u{2061}\u{200c}\u{200d}\u{2062}\u{2061}\u{200d}\u{200d}\u{2062}\u{2064}\u{200d}\u{2062}\u{2061}\u{2062}\u{200d}\u{2062}\u{2061}\u{200c}\u{2061}\u{2062}\u{2061}\u{200d}\u{2061}\u{2062}\u{2064}\u{2061}\u{2062}\u{2061}\u{2062}\u{2061}\u{2062}\u{2061}\u{200c}\u{2063}\u{2061}\u{200d}\u{2063}\u{2064}\u{2063}\u{2061}\u{2063}\u{2062}\u{2061}\u{200c}\u{200c}\u{200c}\u{2062}\u{200d}\u{200c}\u{200c}\u{2062}\u{2061}\u{200c}\u{200c}\u{2063}\u{200c}\u{200c}\u{2062}\u{200c}\u{200d}\u{200c}\u{2062}\u{200d}\u{200d}\u{200c}\u{2062}\u{2061}\u{200d}\u{200c}\u{2063}\u{200d}\u{200c}\u{2062}\u{200c}\u{2061}\u{200c}\u{2062}\u{200d}\u{2061}\u{200c}\u{2062}\u{2064}\u{200c}\u{2063}\u{2061}\u{200c}\u{2062}\u{200c}\u{2062}\u{200c}\u{2062}\u{200d}\u{2062}\u{200c}\u{2062}\u{2061}\u{2062}\u{200c}\u{2063}\u{2062}\u{200c}\u{2062}\u{200c}\u{200c}\u{200d}\u{2062}\u{200d}\u{200c}\u{200d}\u{2062}\u{2061}\u{200c}\u{200d}\u{2063}\u{200c}\u{200d}\u{2062}\u{200c}\u{200d}\u{200d}\u{2062}\u{200d}\u{200d}\u{200d}\u{2062}\u{2061}\u{200d}\u{200d}\u{2063}\u{200d}\u{200d}\u{2062}\u{200c}\u{2061}\u{200d}\u{2062}\u{200d}\u{2061}\u{200d}\u{2062}\u{2064}\u{200d}\u{2063}\u{2061}\u{200d}\u{2062}\u{200c}\u{2062}\u{200d}\u{2062}\u{200d}\u{2062}\u{200d}\u{2062}\u{2061}\u{2062}\u{200d}\u{2063}\u{2062}\u{200d}\u{2062}\u{200c}\u{200c}\u{2061}\u{2062}\u{200d}\u{200c}\u{2061}\u{2062}\u{2061}\u{200c}\u{2061}\u{2063}\u{200c}\u{2061}\u{2062}\u{200c}\u{200d}\u{2061}\u{2062}\u{200d}\u{200d}\u{2061}\u{2062}\u{2061}\u{200d}\u{2061}\u{2063}\u{200d}\u{2061}\u{2062}\u{200c}\u{2064}\u{2062}\u{200d}\u{2064}\u{2062}\u{2064}\u{2061}\u{2063}\u{2064}\u{2062}\u{200c}\u{2062}\u{2061}\u{2062}\u{200d}\u{2062}\u{2061}\u{2062}\u{2061}\u{2062}\u{2061}\u{2063}\u{2062}\u{2061}\u{2062}\u{200c}\u{200c}\u{2063}\u{200d}\u{200c}\u{2063}\u{2061}\u{200c}\u{2063}\u{2062}\u{200c}\u{2063}\u{200c}\u{200d}\u{2063}\u{200d}\u{200d}\u{2063}\u{2061}\u{200d}\u{2063}\u{2062}\u{200d}\u{2063}\u{200c}\u{2061}\u{2063}\u{200d}\u{2061}\u{2063}\u{2064}\u{2063}\u{2062}\u{2061}\u{2063}\u{200c}\u{2063}\u{2062}\u{200d}\u{2063}\u{2062}\u{2061}\u{2063}\u{2063}\u{2063}\u{2062}";

        let result = (0..=255).collect::<Vec<_>>();

        let data = decode(input).unwrap();
        assert_eq!(data, result);
    }

    #[test]
    fn test_decode_data_not_found_no_spaces() {
        assert_eq!(decode(""), Err(CodecError::ZwcDataNotfound));
    }

    #[test]
    fn test_decode_malformed_short() {
        assert_eq!(decode("cover \u{2062}"), Err(CodecError::MalformedData));
    }

    #[test]
    fn test_decode_malformed_just_right() {
        assert_eq!(
            decode("cover \u{2062}\u{2062}"),
            Err(CodecError::MalformedData)
        );
    }

    #[test]
    fn test_decode_stream_end_break() {
        assert_eq!(decode("cover \u{2062}\u{2062}aaa"), Ok(vec![]));
    }

    #[test]
    fn test_decode_stream_end_break_more_text() {
        assert_eq!(decode("cover \u{2062}\u{2062}aaabbbcccddd"), Ok(vec![]));
    }

    #[test]
    fn test_midstream_malformed() {
        assert_eq!(
            decode(
                "cover \u{2062}\u{2062}\u{2062}\u{2062}\u{2062}\u{2062}\u{2062}a\u{2062}\u{2062}"
            ),
            Err(CodecError::MalformedData)
        );
    }
}
