use crate::error::{Result, RunmunchError};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum AffixType {
    Prefix,
    Suffix,
}

#[derive(Debug, Clone)]
pub struct AffixRule {
    pub flag: String,
    pub cross_product: bool,
    pub strip: String,
    pub affix: String,
    pub condition: Option<Regex>,
    pub conditions_raw: String,
}

impl AffixRule {
    fn new(flag: String, cross_product: bool, strip: String, affix: String, condition_str: String) -> Result<Self> {
        let condition = if condition_str == "." || condition_str.is_empty() {
            None
        } else {
            Some(Self::parse_condition(&condition_str)?)
        };

        Ok(AffixRule {
            flag,
            cross_product,
            strip,
            affix,
            condition,
            conditions_raw: condition_str,
        })
    }

    fn parse_condition(condition_str: &str) -> Result<Regex> {
        let mut regex_str = String::new();
        let chars: Vec<char> = condition_str.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            match chars[i] {
                '[' => {
                    let mut bracket_content = String::new();
                    i += 1;
                    
                    let mut negate = false;
                    if i < chars.len() && chars[i] == '^' {
                        negate = true;
                        i += 1;
                    }

                    while i < chars.len() && chars[i] != ']' {
                        bracket_content.push(chars[i]);
                        i += 1;
                    }

                    if i >= chars.len() {
                        return Err(RunmunchError::InvalidAffix("Unclosed bracket in condition".to_string()));
                    }

                    if negate {
                        regex_str.push_str("[^");
                    } else {
                        regex_str.push('[');
                    }
                    regex_str.push_str(&regex::escape(&bracket_content));
                    regex_str.push(']');
                }
                '.' => {
                    regex_str.push_str(".");
                }
                c => {
                    regex_str.push_str(&regex::escape(&c.to_string()));
                }
            }
            i += 1;
        }

        Regex::new(&regex_str).map_err(RunmunchError::Regex)
    }

    pub fn can_apply(&self, word: &str, affix_type: &AffixType) -> bool {
        if word.len() < self.strip.len() {
            return false;
        }

        match affix_type {
            AffixType::Prefix => {
                if !self.strip.is_empty() && !word.starts_with(&self.strip) {
                    return false;
                }
                if let Some(ref condition) = self.condition {
                    // For prefix rules, condition applies after the stripped part
                    // Need to handle Unicode correctly
                    let chars: Vec<char> = word.chars().collect();
                    let strip_len = self.strip.chars().count();
                    let condition_len = self.conditions_raw.chars().count();
                    let check_start = strip_len;
                    let check_end = std::cmp::min(chars.len(), check_start + condition_len);
                    if check_start < chars.len() {
                        let check_part: String = chars[check_start..check_end].iter().collect();
                        condition.is_match(&check_part)
                    } else {
                        false
                    }
                } else {
                    true
                }
            }
            AffixType::Suffix => {
                if !self.strip.is_empty() && !word.ends_with(&self.strip) {
                    return false;
                }
                if let Some(ref condition) = self.condition {
                    // For suffix rules, condition applies to the end of the original word
                    // Need to handle Unicode correctly
                    let chars: Vec<char> = word.chars().collect();
                    let condition_len = self.conditions_raw.chars().count();
                    let suffix_start = chars.len().saturating_sub(condition_len);
                    let check_part: String = chars[suffix_start..].iter().collect();
                    condition.is_match(&check_part)
                } else {
                    true
                }
            }
        }
    }

    pub fn apply(&self, word: &str, affix_type: &AffixType) -> String {
        match affix_type {
            AffixType::Prefix => {
                // Handle Unicode correctly for prefix
                let chars: Vec<char> = word.chars().collect();
                let strip_len = self.strip.chars().count();
                let remaining: String = chars[strip_len..].iter().collect();
                format!("{}{}", self.affix, remaining)
            }
            AffixType::Suffix => {
                // Handle Unicode correctly for suffix
                let chars: Vec<char> = word.chars().collect();
                let strip_len = self.strip.chars().count();
                let remaining: String = chars[..chars.len().saturating_sub(strip_len)].iter().collect();
                format!("{}{}", remaining, self.affix)
            }
        }
    }

    pub fn reverse_apply(&self, word: &str, affix_type: &AffixType) -> Option<String> {
        match affix_type {
            AffixType::Prefix => {
                // For prefix reverse: remove the affix and add back the stripped part
                if !self.affix.is_empty() && !word.starts_with(&self.affix) {
                    return None;
                }
                let chars: Vec<char> = word.chars().collect();
                let affix_len = self.affix.chars().count();
                let remaining: String = chars[affix_len..].iter().collect();
                Some(format!("{}{}", self.strip, remaining))
            }
            AffixType::Suffix => {
                // For suffix reverse: remove the affix and add back the stripped part
                if !self.affix.is_empty() && !word.ends_with(&self.affix) {
                    return None;
                }
                let chars: Vec<char> = word.chars().collect();
                let affix_len = self.affix.chars().count();
                let remaining: String = chars[..chars.len().saturating_sub(affix_len)].iter().collect();
                Some(format!("{}{}", remaining, self.strip))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct AffixFile {
    pub prefixes: HashMap<String, Vec<AffixRule>>,
    pub suffixes: HashMap<String, Vec<AffixRule>>,
    pub flag_type: FlagType,
    pub fullstrip: bool,
    pub flag_aliases: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub enum FlagType {
    Single,
    Long,
    Numeric,
    Utf8,
}

impl AffixFile {
    pub fn new() -> Self {
        AffixFile {
            prefixes: HashMap::new(),
            suffixes: HashMap::new(),
            flag_type: FlagType::Single,
            fullstrip: false,
            flag_aliases: HashMap::new(),
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Self::parse(&content)
    }

    pub fn parse(content: &str) -> Result<Self> {
        let mut affix_file = AffixFile::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();
            
            if line.is_empty() || line.starts_with('#') {
                i += 1;
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            
            match parts.get(0) {
                Some(&"FLAG") => {
                    if let Some(flag_type) = parts.get(1) {
                        affix_file.flag_type = match *flag_type {
                            "long" => FlagType::Long,
                            "num" => FlagType::Numeric,
                            "UTF-8" => FlagType::Utf8,
                            _ => FlagType::Single,
                        };
                    }
                }
                Some(&"FULLSTRIP") => {
                    affix_file.fullstrip = true;
                }
                Some(&"AF") => {
                    if parts.len() >= 2 {
                        // Look for the alias index in the comment (# number)
                        let alias_index = if let Some(comment_pos) = line.find('#') {
                            let comment_part = &line[comment_pos + 1..].trim();
                            comment_part.parse::<u32>().unwrap_or((affix_file.flag_aliases.len() + 1) as u32).to_string()
                        } else {
                            (affix_file.flag_aliases.len() + 1).to_string()
                        };
                        
                        let flags_str = parts[1].to_string(); // Take just the first part (before #)
                        
                        // For long flags, split by pairs; for single flags, split by character
                        let flags = match affix_file.flag_type {
                            FlagType::Long => {
                                flags_str.chars()
                                    .collect::<Vec<_>>()
                                    .chunks(2)
                                    .map(|chunk| chunk.iter().collect::<String>())
                                    .collect()
                            },
                            _ => {
                                flags_str.chars().map(|c| c.to_string()).collect()
                            }
                        };
                        
                        affix_file.flag_aliases.insert(alias_index, flags);
                    }
                }
                Some(&"PFX") | Some(&"SFX") => {
                    if parts.len() >= 4 && (parts[2] == "Y" || parts[2] == "N") {
                        // This is a header line
                        let affix_type = if parts[0] == "PFX" { AffixType::Prefix } else { AffixType::Suffix };
                        let advance = affix_file.parse_affix_block(&lines, i, affix_type)?;
                        i += advance;
                        continue;
                    }
                    // Otherwise, it's a rule line that we'll skip
                }
                _ => {}
            }
            
            i += 1;
        }

        Ok(affix_file)
    }

    fn parse_affix_block(&mut self, lines: &[&str], start: usize, affix_type: AffixType) -> Result<usize> {
        if start >= lines.len() {
            return Ok(0);
        }

        let header_parts: Vec<&str> = lines[start].split_whitespace().collect();
        if header_parts.len() < 3 {
            return Err(RunmunchError::InvalidAffix(format!("Invalid affix header: {:?}", header_parts)));
        }

        let flag = header_parts[1].to_string();
        let cross_product = header_parts[2] == "Y";
        let count: usize = if header_parts.len() >= 4 {
            header_parts[3].parse()
                .map_err(|_| RunmunchError::InvalidAffix(format!("Invalid rule count: {}", header_parts[3])))?
        } else {
            return Err(RunmunchError::InvalidAffix(format!("Missing rule count in line: {}", lines[start])));
        };

        let mut rules = Vec::new();
        let mut processed = 1;

        for i in 1..=count {
            if start + i >= lines.len() {
                break;
            }

            let rule_line = lines[start + i].trim();
            if rule_line.is_empty() || rule_line.starts_with('#') {
                processed = i;
                continue;
            }

            let rule_parts: Vec<&str> = rule_line.split_whitespace().collect();
            if rule_parts.len() >= 4 && rule_parts[0] == header_parts[0] && rule_parts[1] == flag {
                let strip = if rule_parts[2] == "0" { String::new() } else { rule_parts[2].to_string() };
                let affix_str = if rule_parts[3] == "0" { String::new() } else {
                    rule_parts[3].split('/').next().unwrap_or("").to_string()
                };
                let condition = rule_parts.get(4).unwrap_or(&".").to_string();

                let rule = AffixRule::new(flag.clone(), cross_product, strip, affix_str, condition)?;
                rules.push(rule);
            }
            processed = i;
        }

        match affix_type {
            AffixType::Prefix => {
                self.prefixes.insert(flag, rules);
            }
            AffixType::Suffix => {
                self.suffixes.insert(flag, rules);
            }
        }

        Ok(processed)
    }

    pub fn get_prefix_rules(&self, flag: &str) -> Option<&Vec<AffixRule>> {
        self.prefixes.get(flag)
    }

    pub fn get_suffix_rules(&self, flag: &str) -> Option<&Vec<AffixRule>> {
        self.suffixes.get(flag)
    }

    pub fn resolve_flag_alias(&self, alias: &str) -> Vec<String> {
        self.flag_aliases.get(alias).cloned().unwrap_or_else(|| vec![alias.to_string()])
    }

    pub fn expand_flags(&self, flags: &[String]) -> Vec<String> {
        let mut expanded = Vec::new();
        for flag in flags {
            if flag.chars().all(|c| c.is_ascii_digit()) {
                // This is a numeric alias
                expanded.extend(self.resolve_flag_alias(flag));
            } else {
                expanded.push(flag.clone());
            }
        }
        expanded
    }
}

impl Default for AffixFile {
    fn default() -> Self {
        Self::new()
    }
}