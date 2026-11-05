//! The library key as 24 words (issue #243 step 11).
//!
//! BIP-39: the 32-byte key, its first SHA-256 byte as a checksum, cut into 24
//! groups of 11 bits, each an index into the 2048-word English list. The list
//! is the `bip39` crate's; the arithmetic is here so that nothing but
//! wiped buffers ever holds the key. The tests check keys against the
//! crate's own encoding and the BIP-39 test vectors.
//!
//! What the checksum can and can't catch: BIP-39 fixes it at 8 bits for 24
//! words, so a changed word, or two swapped words, pass it about once in 256
//! times (a word that isn't on the list is always caught, by its place).
//! Words that pass are a different key, never this one, and the backup they
//! are typed for refuses them: SQLCipher checks every page against its key.
//! So a typo is always refused; the checksum makes most refusals say "check
//! the words" before the backup is read.

use sha2::{Digest as _, Sha256};
use unicode_normalization::UnicodeNormalization as _;
use zeroize::{Zeroize as _, Zeroizing};

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

/// Whether `word`, in any case, is one of the library key words. The app
/// uses it to keep a key written out as words out of error text.
pub fn is_key_word(word: &str) -> bool {
    let word = word.to_ascii_lowercase();
    list().binary_search(&word.as_str()).is_ok()
}

/// Characters a copy and paste can slip into a word without showing them:
/// zero-width spaces and joiners, the word joiner, the byte-order mark and
/// the soft hyphen. They are dropped, so "aban\u{200b}don" reads "abandon".
fn invisible(c: char) -> bool {
    matches!(
        c,
        '\u{00ad}' | '\u{200b}' | '\u{200c}' | '\u{200d}' | '\u{2060}' | '\u{feff}'
    )
}

impl LibraryKey {
    /// The key as 24 words, single-spaced. Wiped when dropped.
    pub fn to_words(&self) -> Zeroizing<String> {
        let mut bits = Zeroizing::new([0u8; KEY_BYTES + 1]);
        bits[..KEY_BYTES].copy_from_slice(self.as_bytes());
        bits[KEY_BYTES] = Sha256::digest(self.as_bytes())[0];
        let words = list();
        // Room for 24 of the longest words (8 letters) and the spaces, so
        // the string never grows and leaves a copy behind.
        let mut text = Zeroizing::new(String::with_capacity(KEY_WORDS * 9));
        for ix in 0..KEY_WORDS {
            if ix > 0 {
                text.push(' ');
            }
            text.push_str(words[index_at(&bits, ix)]);
        }
        text
    }

    /// The key written as 24 words. Case, spacing, commas, line breaks and
    /// numbering ("1. abandon") don't matter. Unicode is folded first (NFKC:
    /// full-width or ligature letters read as the plain ones) and invisible
    /// characters are dropped. A word with any other letter in it
    /// ("abandón") is named as not on the list, never split or skipped.
    pub fn from_words(text: &str) -> Result<LibraryKey, WordsError> {
        let folded: Zeroizing<String> =
            Zeroizing::new(text.nfkc().filter(|c| !invisible(*c)).collect());
        let mut given: Vec<Zeroizing<String>> = Vec::with_capacity(KEY_WORDS);
        // Anything but a letter separates words, digits included.
        for word in folded
            .split(|c: char| !c.is_alphabetic())
            .filter(|word| !word.is_empty())
        {
            let mut lower = Zeroizing::new(String::with_capacity(word.len() * 3));
            lower.extend(word.chars().flat_map(char::to_lowercase));
            given.push(lower);
        }
        parse(&given)
    }

    /// A short, one-way mark of the key: 16 hex digits of a SHA-256 of it.
    /// It tells whether the key someone said they saved is still the
    /// library's key, and nothing about the key itself.
    pub fn fingerprint(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"kanso library key fingerprint v1\0");
        hasher.update(self.as_bytes());
        hasher.finalize()[..8]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}

fn parse(given: &[Zeroizing<String>]) -> Result<LibraryKey, WordsError> {
    if given.len() != KEY_WORDS {
        return Err(WordsError::WrongCount(given.len()));
    }
    let words = list();
    let mut indexes = Zeroizing::new([0u16; KEY_WORDS]);
    for (place, word) in given.iter().enumerate() {
        let found = word
            .is_ascii()
            .then(|| words.binary_search(&word.as_str()).ok())
            .flatten()
            .ok_or(WordsError::UnknownWord(place + 1))?;
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
    let mut bytes = [0u8; KEY_BYTES];
    bytes.copy_from_slice(&bits[..KEY_BYTES]);
    let key = LibraryKey::from_bytes(bytes);
    bytes.zeroize();
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

    /// The BIP-39 test vectors for 256-bit entropy (trezor/python-mnemonic
    /// `vectors.json`, as the `bip39` crate's tests carry them).
    const VECTORS: &[(&str, &str)] = &[
        (
            "0000000000000000000000000000000000000000000000000000000000000000",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art",
        ),
        (
            "7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f",
            "legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth title",
        ),
        (
            "8080808080808080808080808080808080808080808080808080808080808080",
            "letter advice cage absurd amount doctor acoustic avoid letter advice cage absurd amount doctor acoustic avoid letter advice cage absurd amount doctor acoustic bless",
        ),
        (
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo vote",
        ),
        (
            "68a79eaca2324873eacc50cb9c6eca8cc68ea5d936f98787c60c7ebc74e6ce7c",
            "hamster diagram private dutch cause delay private meat slide toddler razor book happy fancy gospel tennis maple dilemma loan word shrug inflict delay length",
        ),
        (
            "9f6a2878b2520799a44ef18bc7df394e7061a224d2c33cd015b157d746869863",
            "panda eyebrow bullet gorilla call smoke muffin taste mesh discover soft ostrich alcohol speed nation flash devote level hobby quick inner drive ghost inside",
        ),
        (
            "066dca1a2bb7e8a1db2832148ce9933eea0f3ac9548d793112d9a95c9407efad",
            "all hour make first leader extend hole alien behind guard gospel lava path output census museum junior mass reopen famous sing advance salt reform",
        ),
        (
            "f585c11aec520db57dd353c69554b21a89b20fb0650966fa0a9d6f74fd989d8f",
            "void come effort suffer camp survey warrior heavy shoot primary clutch crush open amazing screen patrol group space point ten exist slush involve unfold",
        ),
    ];

    /// Every 256-bit BIP-39 test vector comes out word for word, and reads
    /// back as its entropy.
    #[test]
    fn the_standard_vectors_come_out() {
        for (hex, words) in VECTORS {
            let key = LibraryKey::from_hex(hex).unwrap();
            assert_eq!(key.to_words().as_str(), *words, "{hex}");
            assert_eq!(LibraryKey::from_words(words), Ok(key), "{hex}");
        }
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

    /// Full-width letters, no-break spaces, tabs and invisible characters a
    /// paste brings along read as the plain words.
    #[test]
    fn unicode_a_paste_brings_is_folded() {
        let good = key(9).to_words();
        let full_width: String = good
            .chars()
            .map(|c| match c {
                'a'..='z' => char::from_u32(u32::from(c) - u32::from('a') + 0xff41).unwrap(),
                ' ' => '\u{3000}',
                other => other,
            })
            .collect();
        assert_eq!(LibraryKey::from_words(&full_width), Ok(key(9)));
        let mut words: Vec<String> = good.split(' ').map(str::to_owned).collect();
        words[0].insert(1, '\u{200b}');
        words[1].insert(1, '\u{00ad}');
        words[2].insert(0, '\u{feff}');
        let pasted = words.join("\u{00a0}\t");
        assert_eq!(LibraryKey::from_words(&pasted), Ok(key(9)));
    }

    /// Regression: a letter outside the list's alphabet used to split a
    /// word in two ("aband" "n" for "abandön"), so the count, not the word,
    /// was blamed, and two short words could stand in for one. It is named
    /// as an unknown word at its place now.
    #[test]
    fn a_word_with_a_foreign_letter_is_named_not_split() {
        let good = key(0).to_words();
        let mut words: Vec<&str> = good.split(' ').collect();
        words[2] = "abandön";
        assert_eq!(
            LibraryKey::from_words(&words.join(" ")),
            Err(WordsError::UnknownWord(3))
        );
        // "ﬁ" is a ligature NFKC folds to "fi": "ﬁrst" is "first" and reads.
        let find = LibraryKey::from_hex(VECTORS[6].0).unwrap();
        let typed = VECTORS[6].1.replace("first", "ﬁrst");
        assert_eq!(LibraryKey::from_words(&typed), Ok(find));
        // "art" with a Cyrillic "а" looks the same and is not the word.
        let mut words: Vec<&str> = good.split(' ').collect();
        words[23] = "\u{0430}rt";
        assert_eq!(
            LibraryKey::from_words(&words.join(" ")),
            Err(WordsError::UnknownWord(24))
        );
    }

    /// What the checksum catches, counted over every one-word change and
    /// every swap of neighbours of a few keys: never the same key back, and
    /// about 1 in 256 slips past the checksum (BIP-39's 8 bits), so the
    /// backup's own check has to be the last word. The counts are pinned so
    /// a change to the encoding shows here.
    #[test]
    fn the_checksum_catches_about_255_in_256_mistakes() {
        let mut changed = 0usize;
        let mut slipped = 0usize;
        let mut swaps = 0usize;
        let mut swaps_slipped = 0usize;
        for seed in [0u8, 0x5a, 0xa5, 0xff] {
            let key = key(seed);
            let words = key.to_words();
            let parts: Vec<&str> = words.split(' ').collect();
            for place in 0..KEY_WORDS {
                for other in list() {
                    if *other == parts[place] {
                        continue;
                    }
                    let mut typo = parts.clone();
                    typo[place] = other;
                    changed += 1;
                    match LibraryKey::from_words(&typo.join(" ")) {
                        Ok(read) => {
                            assert_ne!(read, key);
                            slipped += 1;
                        }
                        Err(err) => assert_eq!(err, WordsError::BadChecksum),
                    }
                }
            }
            for place in 0..KEY_WORDS - 1 {
                if parts[place] == parts[place + 1] {
                    continue;
                }
                let mut swapped = parts.clone();
                swapped.swap(place, place + 1);
                swaps += 1;
                if let Ok(read) = LibraryKey::from_words(&swapped.join(" ")) {
                    assert_ne!(read, key);
                    swaps_slipped += 1;
                }
            }
        }
        assert_eq!(changed, 4 * KEY_WORDS * 2047);
        let rate = slipped as f64 / changed as f64;
        assert!((0.002..0.006).contains(&rate), "{slipped} of {changed}");
        assert!(swaps_slipped <= swaps / 16, "{swaps_slipped} of {swaps}");
    }

    #[test]
    fn the_fingerprint_names_the_key_and_not_its_bytes() {
        assert_eq!(key(1).fingerprint(), key(1).fingerprint());
        assert_ne!(key(1).fingerprint(), key(2).fingerprint());
        assert_eq!(key(1).fingerprint().len(), 16);
        assert!(!key(0).to_hex().contains(&key(0).fingerprint()));
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

        /// Invariants: every word of a key, in any case, is a key word, and
        /// `is_key_word` agrees with the list for any short word.
        #[test]
        fn prop_is_key_word_matches_the_list(
            bytes in any::<[u8; KEY_BYTES]>(),
            other in "[a-z]{3,8}",
        ) {
            let words = LibraryKey::from_bytes(bytes).to_words();
            prop_assert!(words.split(' ').all(is_key_word));
            prop_assert!(words.split(' ').all(|w| is_key_word(&w.to_ascii_uppercase())));
            prop_assert_eq!(is_key_word(&other), list().contains(&other.as_str()));
        }

        /// Invariants: the words are standard BIP-39. The `bip39` crate
        /// writes the same words for the same 32 bytes, and reads ours back
        /// as the same bytes.
        #[test]
        fn prop_the_words_are_the_bip39_crates(bytes in any::<[u8; KEY_BYTES]>()) {
            let ours = LibraryKey::from_bytes(bytes).to_words();
            let theirs = bip39::Mnemonic::from_entropy(&bytes).unwrap();
            prop_assert_eq!(ours.as_str(), theirs.to_string());
            let read = bip39::Mnemonic::parse_in_normalized(bip39::Language::English, &ours)
                .unwrap();
            prop_assert_eq!(read.to_entropy(), bytes.to_vec());
        }

        /// Invariants: changing one word, or swapping two neighbours, never
        /// gives back the same key (a different key, or an error), and a
        /// wrong word that is not on the list is named by its place.
        #[test]
        fn prop_a_changed_word_is_never_the_same_key(
            bytes in any::<[u8; KEY_BYTES]>(),
            place in 0usize..KEY_WORDS,
            other in 0usize..2048,
        ) {
            let key = LibraryKey::from_bytes(bytes);
            let words = key.to_words();
            let mut parts: Vec<&str> = words.split(' ').collect();
            if place + 1 < KEY_WORDS && parts[place] != parts[place + 1] {
                let mut swapped = parts.clone();
                swapped.swap(place, place + 1);
                prop_assert_ne!(LibraryKey::from_words(&swapped.join(" ")), Ok(key.clone()));
            }
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
        fn prop_any_text_is_an_answer_not_a_panic(text in "\\PC{0,300}") {
            match LibraryKey::from_words(&text) {
                Ok(_) | Err(WordsError::BadChecksum | WordsError::UnknownWord(_)) => {}
                Err(WordsError::WrongCount(n)) => prop_assert_ne!(n, KEY_WORDS),
            }
        }
    }
}
