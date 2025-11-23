use clap::{Arg, Command};
use runmunch::{Runmunch, WordExpander};
use std::io::{self, BufRead, BufReader};
use std::process;

fn main() {
    let matches = Command::new("runmunch")
        .version("0.1.0")
        .author("Vlatko Kosturjak")
        .about("A Rust implementation of hunspell's unmunch tool for expanding dictionary words using affix files")
        .arg(
            Arg::new("affix")
                .help("Affix file (.aff)")
                .required(true)
                .value_name("AFFIX")
                .index(1),
        )
        .arg(
            Arg::new("dictionary")
                .help("Dictionary file (.dic)")
                .required_unless_present("expand")
                .value_name("DICTIONARY")
                .index(2),
        )
        .arg(
            Arg::new("expand")
                .short('e')
                .long("expand")
                .help("Expand words from stdin using affix rules (optionally with dictionary for flag lookup)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("find-base")
                .short('b')
                .long("find-base")
                .help("Find base word from inflected forms and expand using affix rules (requires dictionary)")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let affix_file = matches.get_one::<String>("affix").unwrap();

    if matches.get_flag("find-base") {
        let dictionary_file = matches.get_one::<String>("dictionary")
            .ok_or("Dictionary file is required for --find-base mode").unwrap();
        if let Err(e) = run_find_base_mode(affix_file, dictionary_file) {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    } else if matches.get_flag("expand") {
        let dictionary_file = matches.get_one::<String>("dictionary");
        if let Err(e) = run_expand_mode(affix_file, dictionary_file) {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    } else {
        let dictionary_file = matches.get_one::<String>("dictionary").unwrap();
        if let Err(e) = run_unmunch_mode(affix_file, dictionary_file) {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn run_expand_mode(affix_file: &str, dictionary_file: Option<&String>) -> Result<(), Box<dyn std::error::Error>> {
    let affix = runmunch::AffixFile::load(affix_file)?;
    let dictionary = if let Some(dict_path) = dictionary_file {
        Some(runmunch::Dictionary::load(dict_path)?)
    } else {
        None
    };
    
    let mut expander = WordExpander::new();
    expander.set_affix_file(&affix);
    
    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());
    
    for line in reader.lines() {
        let word = line?.trim().to_string();
        if !word.is_empty() {
            let expanded = if let Some(ref dict) = dictionary {
                // Look up the word in the dictionary to get its flags
                if let Some(entry) = dict.get_entry(&word) {
                    let expanded_flags = affix.expand_flags(&entry.flags);
                    expander.expand_with_flags(&word, &expanded_flags)?
                } else {
                    // Word not in dictionary, just return it as-is
                    vec![word.clone()]
                }
            } else {
                // No dictionary provided, just try to expand without flags
                expander.expand(&word)?
            };
            
            for expanded_word in expanded {
                println!("{}", expanded_word);
            }
        }
    }
    
    Ok(())
}

fn run_find_base_mode(affix_file: &str, dictionary_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut runmunch = Runmunch::new();
    runmunch.load_affix_file(affix_file)?;
    runmunch.load_dictionary(dictionary_file)?;

    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());

    for line in reader.lines() {
        let word = line?.trim().to_string();
        if !word.is_empty() {
            let expanded = runmunch.find_base_and_expand(&word)?;
            for expanded_word in expanded {
                println!("{}", expanded_word);
            }
        }
    }

    Ok(())
}

fn run_unmunch_mode(affix_file: &str, dictionary_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut runmunch = Runmunch::new();
    
    runmunch.load_affix_file(affix_file)?;
    runmunch.load_dictionary(dictionary_file)?;
    
    let expanded_words = runmunch.unmunch()?;
    
    for word in expanded_words {
        println!("{}", word);
    }
    
    Ok(())
}
