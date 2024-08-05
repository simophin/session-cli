mod codec;
mod lang;

use lang::Language;

pub static ENGLISH: Language = Language::new(include_str!("english.txt"), 3);
pub use codec::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codec_works() {
        let pub_key = "1ad6bd85bcb57c2a47688112f6ac9d04";

        let mnemonic = encode(pub_key, &ENGLISH);

        assert_eq!(mnemonic, "knee cider polar erosion trendy aloof imbalance taunts upkeep lexicon siren soggy siren");
        let decoded_pub_key = decode(&mnemonic, &ENGLISH).expect("to decode");
        assert_eq!(pub_key, decoded_pub_key);
    }
}
