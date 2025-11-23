use crate::error::{Result, RunmunchError};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DictionaryEntry {
    pub word: String,
    pub flags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Dictionary {
    entries: Vec<DictionaryEntry>,
    word_to_entry: HashMap<String, usize>,
}

impl Dictionary {
    pub fn new() -> Self {
        Dictionary {
            entries: Vec::new(),
            word_to_entry: HashMap::new(),
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Self::parse(&content)
    }

    pub fn parse(content: &str) -> Result<Self> {
        let mut dictionary = Dictionary::new();
        let lines: Vec<&str> = content.lines().collect();

        if lines.is_empty() {
            return Err(RunmunchError::InvalidDictionary("Empty dictionary file".to_string()));
        }

        let word_count: usize = lines[0].trim().parse()
            .map_err(|_| RunmunchError::InvalidDictionary("Invalid word count".to_string()))?;

        for (_line_idx, line) in lines.iter().enumerate().skip(1) {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let (word, flags) = Self::parse_entry(line)?;
            let entry = DictionaryEntry { word: word.clone(), flags };
            
            dictionary.word_to_entry.insert(word, dictionary.entries.len());
            dictionary.entries.push(entry);
        }

        if dictionary.entries.len() > word_count {
            eprintln!("Warning: Dictionary contains more entries ({}) than declared ({})", 
                     dictionary.entries.len(), word_count);
        }

        Ok(dictionary)
    }

    fn parse_entry(line: &str) -> Result<(String, Vec<String>)> {
        if let Some(slash_pos) = line.find('/') {
            let word = line[..slash_pos].trim().to_string();
            let flags_str = line[slash_pos + 1..].trim();
            let flags = Self::parse_flags(flags_str);
            Ok((word, flags))
        } else {
            Ok((line.trim().to_string(), Vec::new()))
        }
    }

    fn parse_flags(flags_str: &str) -> Vec<String> {
        if flags_str.is_empty() {
            return Vec::new();
        }

        // We need to make a best guess about flag format without the affix file context
        if flags_str.chars().any(|c| !c.is_ascii_alphabetic()) {
            // Mixed format - handle digits and special characters
            let mut flags = Vec::new();
            let mut chars = flags_str.chars().peekable();

            while let Some(c) = chars.next() {
                if c.is_ascii_digit() {
                    let mut num = c.to_string();
                    while let Some(&next_c) = chars.peek() {
                        if next_c.is_ascii_digit() {
                            num.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    flags.push(num);
                } else {
                    flags.push(c.to_string());
                }
            }
            flags
        } else if flags_str.len() <= 2 {
            // Short alphabetic strings are likely single flags (UN, ED, etc.)
            vec![flags_str.to_string()]
        } else if flags_str.len() % 2 == 0 && flags_str.chars().all(|c| c.is_ascii_uppercase()) {
            // Longer even-length uppercase strings might be long flags (pairs)
            flags_str.chars()
                .collect::<Vec<_>>()
                .chunks(2)
                .map(|chunk| chunk.iter().collect::<String>())
                .collect()
        } else {
            // Default to single character flags for other cases
            flags_str.chars().map(|c| c.to_string()).collect()
        }
    }

    pub fn entries(&self) -> impl Iterator<Item = (&String, &Vec<String>)> {
        self.entries.iter().map(|entry| (&entry.word, &entry.flags))
    }

    pub fn get_entry(&self, word: &str) -> Option<&DictionaryEntry> {
        self.word_to_entry.get(word).map(|&idx| &self.entries[idx])
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for Dictionary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_flags() {
        assert_eq!(Dictionary::parse_flags("abc"), vec!["a", "b", "c"]);
        assert_eq!(Dictionary::parse_flags("ABCD"), vec!["AB", "CD"]); // Long flags (pairs of uppercase)
        assert_eq!(Dictionary::parse_flags("123"), vec!["123"]); // Numeric flags are kept as single units
        assert_eq!(Dictionary::parse_flags(""), Vec::<String>::new());
        assert_eq!(Dictionary::parse_flags("A"), vec!["A"]);
        assert_eq!(Dictionary::parse_flags("AB"), vec!["AB"]); // Short strings are treated as single flags
        assert_eq!(Dictionary::parse_flags("UN"), vec!["UN"]); // Short strings are treated as single flags
        assert_eq!(Dictionary::parse_flags("ED"), vec!["ED"]); // Short strings are treated as single flags
    }

    #[test]
    fn test_parse_entry() {
        let (word, flags) = Dictionary::parse_entry("test/abc").unwrap();
        assert_eq!(word, "test");
        assert_eq!(flags, vec!["a", "b", "c"]);

        let (word, flags) = Dictionary::parse_entry("simple").unwrap();
        assert_eq!(word, "simple");
        assert!(flags.is_empty());
    }
}