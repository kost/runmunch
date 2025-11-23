use runmunch::{Runmunch, WordExpander, AffixFile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Runmunch Basic Usage Example ===\n");

    // Example 1: Using WordExpander directly
    println!("1. Word Expansion Example:");
    
    // Create a simple affix file content
    let affix_content = r#"
PFX UN Y 1
PFX UN 0 un .

SFX ED Y 1
SFX ED 0 ed .

SFX S Y 1
SFX S 0 s .
"#;

    // Parse the affix file
    let affix_file = AffixFile::parse(affix_content)?;
    
    // Create and configure expander
    let mut expander = WordExpander::new();
    expander.set_affix_file(&affix_file);
    
    // Expand words with flags
    println!("  Expanding 'happy' with UN flag:");
    let expanded = expander.expand_with_flags("happy", &["UN".to_string()])?;
    for word in &expanded {
        println!("    - {}", word);
    }
    
    println!("\n  Expanding 'work' with ED flag:");
    let expanded = expander.expand_with_flags("work", &["ED".to_string()])?;
    for word in &expanded {
        println!("    - {}", word);
    }
    
    println!("\n  Expanding 'cat' with multiple flags (S and ED):");
    let expanded = expander.expand_with_flags("cat", &["S".to_string(), "ED".to_string()])?;
    for word in &expanded {
        println!("    - {}", word);
    }

    // Example 2: Full dictionary unmunching
    println!("\n2. Dictionary Unmunching Example:");
    
    // Create sample dictionary content
    let dict_content = r#"3
happy/UN
work/ED,S
test
"#;

    // Write temporary files
    std::fs::write("/tmp/example.aff", affix_content)?;
    std::fs::write("/tmp/example.dic", dict_content)?;
    
    // Create Runmunch instance and load files
    let mut runmunch = Runmunch::new();
    runmunch.load_affix_file("/tmp/example.aff")?;
    runmunch.load_dictionary("/tmp/example.dic")?;
    
    // Expand all words from dictionary
    let all_expanded = runmunch.unmunch()?;
    println!("  Expanded {} words from dictionary:", all_expanded.len());
    for word in &all_expanded {
        println!("    - {}", word);
    }

    // Example 3: Single word expansion (without flags)
    println!("\n3. Single Word Expansion (no flags):");
    let simple_expansion = runmunch.expand_word("simple")?;
    for word in &simple_expansion {
        println!("    - {}", word);
    }

    // Clean up temporary files
    let _ = std::fs::remove_file("/tmp/example.aff");
    let _ = std::fs::remove_file("/tmp/example.dic");
    
    println!("\n=== Example Complete ===");
    
    Ok(())
}