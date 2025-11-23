pub mod affix;
pub mod dictionary;
pub mod expander;
pub mod error;

pub use affix::{AffixFile, AffixRule, AffixType};
pub use dictionary::Dictionary;
pub use expander::WordExpander;
pub use error::{RunmunchError, Result};

use std::collections::HashSet;

pub struct Runmunch {
    affix_file: Option<AffixFile>,
    dictionary: Option<Dictionary>,
    expander: WordExpander,
}

impl Runmunch {
    pub fn new() -> Self {
        Self {
            affix_file: None,
            dictionary: None,
            expander: WordExpander::new(),
        }
    }

    pub fn load_affix_file<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        let affix_file = AffixFile::load(path)?;
        self.expander.set_affix_file(&affix_file);
        self.affix_file = Some(affix_file);
        Ok(())
    }

    pub fn load_dictionary<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        let dictionary = Dictionary::load(path)?;
        self.dictionary = Some(dictionary);
        Ok(())
    }

    pub fn expand_word(&self, word: &str) -> Result<Vec<String>> {
        self.expander.expand(word)
    }

    pub fn find_base_and_expand(&self, inflected_word: &str) -> Result<Vec<String>> {
        let dictionary = self.dictionary.as_ref()
            .ok_or_else(|| RunmunchError::NoDictionary)?;
        self.expander.find_base_and_expand(inflected_word, dictionary)
    }

    pub fn expand_words(&self, words: &[String]) -> Result<Vec<String>> {
        let mut result = Vec::new();
        let mut seen = HashSet::new();

        for word in words {
            let expanded = self.expand_word(word)?;
            for expanded_word in expanded {
                if seen.insert(expanded_word.clone()) {
                    result.push(expanded_word);
                }
            }
        }

        Ok(result)
    }

    pub fn unmunch(&self) -> Result<Vec<String>> {
        let dictionary = self.dictionary.as_ref()
            .ok_or_else(|| RunmunchError::NoDictionary)?;

        let mut result = Vec::new();
        let mut seen = HashSet::new();

        for (word, flags) in dictionary.entries() {
            let expanded = if flags.is_empty() {
                vec![word.clone()]
            } else {
                self.expander.expand_with_flags(word, flags)?
            };

            for expanded_word in expanded {
                if seen.insert(expanded_word.clone()) {
                    result.push(expanded_word);
                }
            }
        }

        Ok(result)
    }
}

impl Default for Runmunch {
    fn default() -> Self {
        Self::new()
    }
}
