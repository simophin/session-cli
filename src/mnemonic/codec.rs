use super::lang::Language;
use anyhow::{bail, Context};
use crc::CRC_32_ISO_HDLC;

pub fn encode(hex_encoded_string: &str, lang: &Language<'_>) -> String {
    let mut string = hex_encoded_string.to_string();
    let word_set = lang.words();
    let prefix_length = lang.prefix_length();
    let mut result = Vec::new();
    let n = word_set.len() as u64;
    let character_count = string.len();
    for chunk_start_index in (0..character_count).step_by(8) {
        let chunk_end_index = chunk_start_index + 8;
        let p1 = &string[0..chunk_start_index];
        let p2 = swap(&string[chunk_start_index..chunk_end_index]);
        let p3 = &string[chunk_end_index..character_count];
        string = format!("{}{}{}", p1, p2, p3);
    }
    for chunk_start_index in (0..character_count).step_by(8) {
        let chunk_end_index = chunk_start_index + 8;
        let x = u64::from_str_radix(&string[chunk_start_index..chunk_end_index], 16).unwrap();
        let w1 = x % n;
        let w2 = ((x / n) + w1) % n;
        let w3 = (((x / n) / n) + w2) % n;
        result.push(word_set[w1 as usize]);
        result.push(word_set[w2 as usize]);
        result.push(word_set[w3 as usize]);
    }
    let checksum_index = determine_checksum_index(&result, prefix_length);
    let checksum_word = result[checksum_index];
    result.push(checksum_word);
    result.join(" ")
}

pub fn decode(mnemonic: &str, language: &Language<'_>) -> anyhow::Result<String> {
    let mut words = mnemonic.split_whitespace().collect::<Vec<_>>();
    let word_set = language.truncated_words();
    let prefix_length = language.prefix_length();
    let mut result = String::new();
    let n = word_set.len() as u64;
    if words.len() < 12 {
        bail!("InputTooShort");
    }
    if words.len() % 3 == 0 {
        bail!("MissingLastWord");
    }
    let checksum_word = words.pop().context("NoChecksumWord")?;
    for chunk_start_index in (0..words.len()).step_by(3) {
        let word1 = &words[chunk_start_index][..prefix_length];
        let w1 = word_set
            .iter()
            .position(|&r| r == word1)
            .context("Verification fail")? as u64;

        let word2 = &words[chunk_start_index + 1][..prefix_length];
        let w2 = word_set
            .iter()
            .position(|&r| r == word2)
            .context("Verification fail")? as u64;

        let word3 = &words[chunk_start_index + 2][..prefix_length];
        let w3 = word_set
            .iter()
            .position(|&r| r == word3)
            .context("Verification fail")? as u64;

        let x = w1 + n * ((n - w1 + w2) % n) + n * n * ((n - w2 + w3) % n);
        if x % n != w1 {
            bail!("Generic");
        }
        let string = format!("0000000{:x}", x);
        result += &swap(&string[string.len() - 8..string.len()]);
    }
    let checksum_index = determine_checksum_index(&words, prefix_length);
    let expected_checksum_word = words[checksum_index];
    if &expected_checksum_word[..prefix_length] != &checksum_word[..prefix_length] {
        bail!("VerificationFailed");
    }
    Ok(result)
}

fn swap(x: &str) -> String {
    let p1 = &x[6..8];
    let p2 = &x[4..6];
    let p3 = &x[2..4];
    let p4 = &x[0..2];
    format!("{}{}{}{}", p1, p2, p3, p4)
}

fn determine_checksum_index(x: &Vec<&str>, prefix_length: usize) -> usize {
    let bytes = x
        .iter()
        .map(|s| &s[..prefix_length.min(s.len())])
        .collect::<Vec<_>>()
        .join("")
        .into_bytes();
    let checksum = crc::Crc::<u32>::new(&CRC_32_ISO_HDLC).checksum(&bytes);
    checksum as usize % x.len()
}
