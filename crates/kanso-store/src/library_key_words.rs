//! The library key as 24 words (issue #243 step 11).
//!
//! BIP-39: the 32-byte key, its first SHA-256 byte as a checksum, cut into 24
//! groups of 11 bits, each an index into the 2048-word English list. The list
//! is the `bip39` crate's; the arithmetic is here so that nothing but
//! wiped buffers ever holds the key. A mistyped word fails the checksum
//! rather than opening a different library.

use sha2::{Digest as _, Sha256};
use zeroize::Zeroizing;

use crate::{KEY_BYTES, LibraryKey};

/// Words in a library key.
pub const KEY_WORDS: usize = 24;

/// Bits in one word.
const WORD_BITS: usize = 11;

/// Why some words are not a library key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum WordsError {
    /// Not 24 words.
    #[error("A library key is 24 words. This has {0}.")]
    WrongCount(usize),
    /// A word that isn't in the list: its place, counting from 1.
    #[error("Word {0} isn't one of the library key words. Check how it is spelled.")]
    UnknownWord(usize),
    /// 24 real words that don't belong together.
    #[error("These words don't make a library key. Check each word and their order.")]
    BadChecksum,
}

fn list() -> &'static [&'static str; 2048] {
    bip39::Language::English.word_list()
}

impl LibraryKey {
    /// The key as 24 words, single-spaced. Wiped when dropped.
    pub fn to_words(&self) -> Zeroizing<String> {
        let mut bits = Zeroizing::new([0u8; KEY_BYTES + 1]);
        bits[..KEY_BYTES].copy_from_slice(self.as_bytes());
        bits[KEY_BYTES] = Sha256::digest(self.as_bytes())[0];
        let words = list();
        let mut text = Zeroizing::new(String::with_capacity(KEY_WORDS * 9));
        for ix in 0..KEY_WORDS {
            if ix > 0 {
                text.push(' ');
            }
            text.push_str(words[index_at(&bits, ix)]);
        }
        text
    }

    /// The key written as 24 words. Case, spacing, commas and line breaks
    /// don't matter, and neither do numbers ("1. abandon" reads as "abandon").
    pub fn from_words(text: &str) -> Result<LibraryKey, WordsError> {
        let given: Vec<String> = text
            .split(|c: char| !c.is_ascii_alphabetic())
            .filter(|word| !word.is_empty())
            .map(str::to_ascii_lowercase)
            .collect();
        let result = parse(&given);
        // The lowercased copies are the key too.
        let mut given = given;
        for word in &mut given {
            zeroize::Zeroize::zeroize(word);
        }
        result
    }
}

fn parse(given: &[String]) -> Result<LibraryKey, WordsError> {
    if given.len() != KEY_WORDS {
        return Err(WordsError::WrongCount(given.len()));
    }
    let words = list();
    let mut indexes = Zeroizing::new([0u16; KEY_WORDS]);
    for (place, word) in given.iter().enumerate() {
        let found = words
            .binary_search(&word.as_str())
            .map_err(|_| WordsError::UnknownWord(place + 1))?;
        indexes[place] = u16::try_from(found).expect("an index under 2048");
    }
    let mut bits = Zeroizing::new([0u8; KEY_BYTES + 1]);
    for (place, index) in indexes.iter().enumerate() {
        for bit in 0..WORD_BITS {
            if index >> (WORD_BITS - 1 - bit) & 1 == 1 {
                let at = place * WORD_BITS + bit;
                bits[at / 8] |= 0x80 >> (at % 8);
            }
        }
    }
    let mut key = [0u8; KEY_BYTES];
    key.copy_from_slice(&bits[..KEY_BYTES]);
    let key = LibraryKey::from_bytes(key);
    if Sha256::digest(key.as_bytes())[0] != bits[KEY_BYTES] {
        return Err(WordsError::BadChecksum);
    }
    Ok(key)
}

/// The `ix`th group of 11 bits.
fn index_at(bits: &[u8; KEY_BYTES + 1], ix: usize) -> usize {
    let mut value = 0usize;
    for bit in 0..WORD_BITS {
        let at = ix * WORD_BITS + bit;
        value = value << 1 | usize::from(bits[at / 8] >> (7 - at % 8) & 1);
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::{Config, RngAlgorithm, RngSeed};

    fn key(byte: u8) -> LibraryKey {
        LibraryKey::from_bytes([byte; KEY_BYTES])
    }

    /// The BIP-39 test vectors for 256-bit entropy.
    #[test]
    fn the_standard_vectors_come_out() {
        let zero = format!("{}art", "abandon ".repeat(23));
        assert_eq!(key(0x00).to_words().as_str(), zero);
        let ones = format!("{}vote", "zoo ".repeat(23));
        assert_eq!(key(0xff).to_words().as_str(), ones);
        assert_eq!(LibraryKey::from_words(&zero), Ok(key(0x00)));
        assert_eq!(LibraryKey::from_words(&ones), Ok(key(0xff)));
    }

    #[test]
    fn each_mistake_says_what_is_wrong() {
        let good = key(7).to_words();
        let mut words: Vec<&str> = good.split(' ').collect();
        assert_eq!(
            LibraryKey::from_words(&words[..23].join(" ")),
            Err(WordsError::WrongCount(23))
        );
        assert_eq!(LibraryKey::from_words(""), Err(WordsError::WrongCount(0)));
        let original = words[4];
        words[4] = "kanso";
        assert_eq!(
            LibraryKey::from_words(&words.join(" ")),
            Err(WordsError::UnknownWord(5))
        );
        words[4] = original;
        words.swap(0, 23);
        assert_ne!(LibraryKey::from_words(&words.join(" ")), Ok(key(7)));
    }

    #[test]
    fn typing_style_does_not_matter() {
        let good = key(42).to_words();
        let spaced = good
            .split(' ')
            .enumerate()
            .map(|(ix, word)| format!("{}. {}", ix + 1, word.to_uppercase()))
            .collect::<Vec<_>>()
            .join(",\n  ");
        assert_eq!(LibraryKey::from_words(&spaced), Ok(key(42)));
    }

    proptest! {
        #![proptest_config(Config {
            cases: 256,
            rng_seed: RngSeed::Fixed(0x006b_616e_736f),
            rng_algorithm: RngAlgorithm::ChaCha,
            failure_persistence: None,
            ..Config::default()
        })]

        /// Invariants: any key written as words reads back as the same key,
        /// in 24 words of the list.
        #[test]
        fn prop_words_round_trip(bytes in any::<[u8; KEY_BYTES]>()) {
            let key = LibraryKey::from_bytes(bytes);
            let words = key.to_words();
            prop_assert_eq!(words.split(' ').count(), KEY_WORDS);
            prop_assert!(words.split(' ').all(|w| list().contains(&w)));
            prop_assert_eq!(LibraryKey::from_words(&words), Ok(key));
        }

        /// Invariants: changing one word never gives back the same key (a
        /// different key, or an error), and a wrong word that is not on the
        /// list is named by its place.
        #[test]
        fn prop_a_changed_word_is_never_the_same_key(
            bytes in any::<[u8; KEY_BYTES]>(),
            place in 0usize..KEY_WORDS,
            other in 0usize..2048,
        ) {
            let key = LibraryKey::from_bytes(bytes);
            let words = key.to_words();
            let mut parts: Vec<&str> = words.split(' ').collect();
            prop_assume!(parts[place] != list()[other]);
            parts[place] = list()[other];
            prop_assert_ne!(LibraryKey::from_words(&parts.join(" ")), Ok(key));
            parts[place] = "zzzz";
            prop_assert_eq!(
                LibraryKey::from_words(&parts.join(" ")),
                Err(WordsError::UnknownWord(place + 1))
            );
        }

        /// Invariants: whatever is typed, reading it doesn't panic and a
        /// wrong count is reported as the count of words found.
        #[test]
        fn prop_any_text_is_an_answer_not_a_panic(text in ".{0,300}") {
            match LibraryKey::from_words(&text) {
                Ok(_) | Err(WordsError::BadChecksum | WordsError::UnknownWord(_)) => {}
                Err(WordsError::WrongCount(n)) => prop_assert_ne!(n, KEY_WORDS),
            }
        }
    }
}
