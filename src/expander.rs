use crate::affix::{AffixFile, AffixType};
use crate::error::{Result, RunmunchError};
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Clone)]
pub struct WordExpander {
    affix_file: Option<AffixFile>,
}

impl WordExpander {
    pub fn new() -> Self {
        WordExpander { affix_file: None }
    }

    pub fn set_affix_file(&mut self, affix_file: &AffixFile) {
        self.affix_file = Some(affix_file.clone());
    }

    pub fn expand(&self, word: &str) -> Result<Vec<String>> {
        // If no flags provided, try to expand with all available rules
        self.expand_with_all_rules(word)
    }

    pub fn expand_with_all_rules(&self, word: &str) -> Result<Vec<String>> {
        let affix_file = self.affix_file.as_ref()
            .ok_or_else(|| RunmunchError::NoAffixFile)?;

        let mut results = HashSet::new();
        results.insert(word.to_string());

        // Try all prefix rules
        for (_flag, rules) in &affix_file.prefixes {
            for rule in rules {
                if rule.can_apply(word, &AffixType::Prefix) {
                    let expanded = rule.apply(word, &AffixType::Prefix);
                    results.insert(expanded);
                }
            }
        }

        // Try all suffix rules
        for (_flag, rules) in &affix_file.suffixes {
            for rule in rules {
                if rule.can_apply(word, &AffixType::Suffix) {
                    let expanded = rule.apply(word, &AffixType::Suffix);
                    results.insert(expanded);
                }
            }
        }

        let mut sorted_results: Vec<String> = results.into_iter().collect();
        sorted_results.sort();
        Ok(sorted_results)
    }

    pub fn expand_with_flags(&self, word: &str, flags: &[String]) -> Result<Vec<String>> {
        let affix_file = self.affix_file.as_ref()
            .ok_or_else(|| RunmunchError::NoAffixFile)?;

        // Expand flag aliases first
        let expanded_flags = affix_file.expand_flags(flags);

        let mut results = HashSet::new();
        let mut queue = VecDeque::new();
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 10000;

        results.insert(word.to_string());
        queue.push_back((word.to_string(), expanded_flags, false, 0));

        while let Some((current_word, current_flags, has_suffix, depth)) = queue.pop_front() {
            iterations += 1;
            if iterations > MAX_ITERATIONS || depth > 2 {
                break;
            }
            for flag in &current_flags {
                if let Some(suffix_rules) = affix_file.get_suffix_rules(flag) {
                    for rule in suffix_rules {
                        if rule.can_apply(&current_word, &AffixType::Suffix) {
                            let expanded = rule.apply(&current_word, &AffixType::Suffix);
                            if results.insert(expanded.clone()) {
                                if rule.cross_product && depth < 1 {
                                    queue.push_back((expanded, current_flags.clone(), true, depth + 1));
                                }
                            }
                        }
                    }
                }
            }

            if has_suffix {
                for flag in &current_flags {
                    if let Some(prefix_rules) = affix_file.get_prefix_rules(flag) {
                        for rule in prefix_rules {
                            if rule.cross_product && rule.can_apply(&current_word, &AffixType::Prefix) {
                                let expanded = rule.apply(&current_word, &AffixType::Prefix);
                                results.insert(expanded);
                            }
                        }
                    }
                }
            } else {
                for flag in &current_flags {
                    if let Some(prefix_rules) = affix_file.get_prefix_rules(flag) {
                        for rule in prefix_rules {
                            if rule.can_apply(&current_word, &AffixType::Prefix) {
                                let expanded = rule.apply(&current_word, &AffixType::Prefix);
                                results.insert(expanded);
                            }
                        }
                    }
                }
            }
        }

        let mut sorted_results: Vec<String> = results.into_iter().collect();
        sorted_results.sort();
        Ok(sorted_results)
    }

    pub fn expand_words_from_stdin(&self) -> Result<Vec<String>> {
        use std::io::{self, BufRead, BufReader};

        let stdin = io::stdin();
        let reader = BufReader::new(stdin.lock());
        let mut all_results = HashSet::new();

        for line in reader.lines() {
            let word = line?.trim().to_string();
            if !word.is_empty() {
                let expanded = self.expand(&word)?;
                for expanded_word in expanded {
                    all_results.insert(expanded_word);
                }
            }
        }

        let mut sorted_results: Vec<String> = all_results.into_iter().collect();
        sorted_results.sort();
        Ok(sorted_results)
    }

    pub fn find_base_word(&self, inflected_word: &str, dictionary: &crate::Dictionary) -> Result<Vec<String>> {
        let affix_file = self.affix_file.as_ref()
            .ok_or_else(|| RunmunchError::NoAffixFile)?;

        let mut base_words = HashSet::new();

        // First, check if the word itself is in the dictionary
        if let Some(entry) = dictionary.get_entry(inflected_word) {
            // Verify that the word can generate itself (should always be true)
            let expanded = self.expand_with_flags(inflected_word, &entry.flags)?;
            if expanded.contains(&inflected_word.to_string()) {
                base_words.insert(inflected_word.to_string());
            }
        }

        // Try removing suffixes to find base forms
        for (_flag, suffix_rules) in &affix_file.suffixes {
            for rule in suffix_rules {
                if let Some(candidate_base) = rule.reverse_apply(inflected_word, &AffixType::Suffix) {
                    if let Some(entry) = dictionary.get_entry(&candidate_base) {
                        // Check if this base word with its flags can generate the inflected word
                        let expanded = self.expand_with_flags(&candidate_base, &entry.flags)?;
                        if expanded.contains(&inflected_word.to_string()) {
                            base_words.insert(candidate_base);
                        }
                    }
                }
            }
        }

        // Try removing prefixes to find base forms
        for (_flag, prefix_rules) in &affix_file.prefixes {
            for rule in prefix_rules {
                if let Some(candidate_base) = rule.reverse_apply(inflected_word, &AffixType::Prefix) {
                    if let Some(entry) = dictionary.get_entry(&candidate_base) {
                        // Check if this base word with its flags can generate the inflected word
                        let expanded = self.expand_with_flags(&candidate_base, &entry.flags)?;
                        if expanded.contains(&inflected_word.to_string()) {
                            base_words.insert(candidate_base);
                        }
                    }
                }
            }
        }

        // Try removing both prefixes and suffixes (for complex morphology)
        for (_prefix_flag, prefix_rules) in &affix_file.prefixes {
            for prefix_rule in prefix_rules {
                if let Some(after_prefix_removal) = prefix_rule.reverse_apply(inflected_word, &AffixType::Prefix) {
                    for (_suffix_flag, suffix_rules) in &affix_file.suffixes {
                        for suffix_rule in suffix_rules {
                            if let Some(candidate_base) = suffix_rule.reverse_apply(&after_prefix_removal, &AffixType::Suffix) {
                                if let Some(entry) = dictionary.get_entry(&candidate_base) {
                                    let expanded = self.expand_with_flags(&candidate_base, &entry.flags)?;
                                    if expanded.contains(&inflected_word.to_string()) {
                                        base_words.insert(candidate_base);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut sorted_results: Vec<String> = base_words.into_iter().collect();
        sorted_results.sort();
        Ok(sorted_results)
    }

    pub fn find_base_and_expand(&self, inflected_word: &str, dictionary: &crate::Dictionary) -> Result<Vec<String>> {
        let base_words = self.find_base_word(inflected_word, dictionary)?;

        if base_words.is_empty() {
            // If no base word found, just return the original word
            return Ok(vec![inflected_word.to_string()]);
        }

        let mut all_expansions = HashSet::new();

        for base_word in base_words {
            if let Some(entry) = dictionary.get_entry(&base_word) {
                let expanded = self.expand_with_flags(&base_word, &entry.flags)?;
                for word in expanded {
                    all_expansions.insert(word);
                }
            }
        }

        let mut sorted_results: Vec<String> = all_expansions.into_iter().collect();
        sorted_results.sort();
        Ok(sorted_results)
    }

    pub fn has_affix_file(&self) -> bool {
        self.affix_file.is_some()
    }
}

impl Default for WordExpander {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::affix::AffixFile;

    fn create_test_affix() -> AffixFile {
        let affix_content = r#"
FLAG long

PFX UN Y 1
PFX UN 0 un .

SFX ED Y 1
SFX ED 0 ed .

SFX S Y 1
SFX S 0 s .
"#;
        AffixFile::parse(affix_content).unwrap()
    }

    #[test]
    fn test_expand_with_prefix() {
        let mut expander = WordExpander::new();
        let affix_file = create_test_affix();
        expander.set_affix_file(&affix_file);

        let result = expander.expand_with_flags("happy", &["UN".to_string()]).unwrap();
        assert!(result.contains(&"happy".to_string()));
        assert!(result.contains(&"unhappy".to_string()));
    }

    #[test]
    fn test_expand_with_suffix() {
        let mut expander = WordExpander::new();
        let affix_file = create_test_affix();
        expander.set_affix_file(&affix_file);

        let result = expander.expand_with_flags("work", &["ED".to_string()]).unwrap();
        assert!(result.contains(&"work".to_string()));
        assert!(result.contains(&"worked".to_string()));
    }

    #[test]
    fn test_expand_multiple_flags() {
        let mut expander = WordExpander::new();
        let affix_file = create_test_affix();
        expander.set_affix_file(&affix_file);

        let result = expander.expand_with_flags("cat", &["S".to_string(), "ED".to_string()]).unwrap();
        assert!(result.contains(&"cat".to_string()));
        assert!(result.contains(&"cats".to_string()));
        assert!(result.contains(&"cated".to_string())); // Note: This is grammatically incorrect but follows the rules
    }
}