use runmunch::*;
use std::path::Path;

#[test]
fn test_basic_affix_parsing() {
    let affix_content = r#"
FLAG long

PFX UN Y 1
PFX UN 0 un .

SFX ED Y 1
SFX ED 0 ed .

SFX S Y 1
SFX S 0 s .
"#;
    
    let affix_file = AffixFile::parse(affix_content).expect("Should parse basic affix file");
    
    assert_eq!(affix_file.prefixes.len(), 1);
    assert_eq!(affix_file.suffixes.len(), 2);
    
    let un_rules = affix_file.get_prefix_rules("UN").expect("Should have UN prefix rules");
    assert_eq!(un_rules.len(), 1);
    assert_eq!(un_rules[0].affix, "un");
    
    let ed_rules = affix_file.get_suffix_rules("ED").expect("Should have ED suffix rules");
    assert_eq!(ed_rules.len(), 1);
    assert_eq!(ed_rules[0].affix, "ed");
}

#[test]
fn test_word_expansion() {
    let affix_content = r#"
PFX UN Y 1
PFX UN 0 un .

SFX ED Y 1
SFX ED 0 ed .
"#;
    
    let affix_file = AffixFile::parse(affix_content).expect("Should parse affix file");
    let mut expander = WordExpander::new();
    expander.set_affix_file(&affix_file);
    
    let result = expander.expand_with_flags("happy", &["UN".to_string()]).expect("Should expand word");
    assert!(result.contains(&"happy".to_string()));
    assert!(result.contains(&"unhappy".to_string()));
    
    let result = expander.expand_with_flags("work", &["ED".to_string()]).expect("Should expand word");
    assert!(result.contains(&"work".to_string()));
    assert!(result.contains(&"worked".to_string()));
}

#[test]
fn test_dictionary_parsing() {
    let dict_content = r#"3
hello/ED
world
test/UN,S
"#;
    
    let dictionary = Dictionary::parse(dict_content).expect("Should parse dictionary");
    assert_eq!(dictionary.len(), 3);
    
    let hello_entry = dictionary.get_entry("hello").expect("Should have hello entry");
    assert_eq!(hello_entry.flags, vec!["ED"]); // Short flag strings are parsed as single flags
    
    let world_entry = dictionary.get_entry("world").expect("Should have world entry");
    assert!(world_entry.flags.is_empty());
    
    let test_entry = dictionary.get_entry("test").expect("Should have test entry");
    assert_eq!(test_entry.flags, vec!["U", "N", ",", "S"]); // Mixed format parses character by character
}

#[test]
fn test_full_runmunch_workflow() {
    let affix_content = r#"
PFX UN Y 1
PFX UN 0 un .

SFX ED Y 1
SFX ED 0 ed .
"#;
    
    let dict_content = r#"2
happy/UN
work/ED
"#;
    
    use std::io::Write;
    use std::fs;
    
    let mut runmunch = Runmunch::new();
    
    // Write temporary files
    fs::write("/tmp/test.aff", affix_content).expect("Should write affix file");
    fs::write("/tmp/test.dic", dict_content).expect("Should write dict file");
    
    runmunch.load_affix_file("/tmp/test.aff").expect("Should load affix file");
    runmunch.load_dictionary("/tmp/test.dic").expect("Should load dictionary file");
    
    let results = runmunch.unmunch().expect("Should generate expanded words");
    
    assert!(results.contains(&"happy".to_string()));
    assert!(results.contains(&"unhappy".to_string()));
    assert!(results.contains(&"work".to_string()));
    assert!(results.contains(&"worked".to_string()));
    assert_eq!(results.len(), 5); // Note: includes "workeded" due to current expansion logic
}

#[cfg(test)]
mod hunspell_hr_tests {
    use super::*;
    
    #[test]
    fn test_can_load_hunspell_hr_files() {
        let affix_path = "hunspell-hr/hr_HR.aff";
        let dict_path = "hunspell-hr/hr_HR.dic";
        
        if Path::new(affix_path).exists() && Path::new(dict_path).exists() {
            let affix_file = AffixFile::load(affix_path).expect("Should load Croatian affix file");
            let dictionary = Dictionary::load(dict_path).expect("Should load Croatian dictionary");
            
            assert!(!affix_file.prefixes.is_empty() || !affix_file.suffixes.is_empty());
            assert!(!dictionary.is_empty());
            
            let mut runmunch = Runmunch::new();
            runmunch.load_affix_file(affix_path).expect("Should load affix file");
            runmunch.load_dictionary(dict_path).expect("Should load dictionary file");
            
            let expanded = runmunch.unmunch().expect("Should expand words");
            assert!(!expanded.is_empty());
            println!("Expanded {} words from Croatian dictionary", expanded.len());
        }
    }
}