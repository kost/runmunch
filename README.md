# Runmunch

A fast, efficient Rust implementation of hunspell's unmunch tool for expanding dictionary words using morphological affix rules. Generate all possible word forms from dictionaries or expand individual words interactively.

##  Features

- **High Performance** - Processes 23K+ words/second (German), optimized Rust implementation
- **Dual Interface** - Use as both a library and command-line tool
- **Word Expansion** - Interactive word expansion using affix rules (`--expand` mode)
- **Base Word Finding** - Find base forms from inflected words (`--find-base` mode)
- **Dictionary Unmunching** - Batch processing of entire dictionary files
- **Unicode Support** - Full support for international languages (German, Croatian, etc.)
- **Hunspell Compatible** - Works with standard hunspell .aff and .dic files
- **Flag Alias Support** - Handles complex affix flag systems (AF directive)
- **Memory Efficient** - Optimized for large dictionaries and complex morphology

## Installation

```bash
cargo install runmunch
```

Or build from source:

```bash
git clone <repository>
cd runmunch
cargo build --release
```

## Usage

### Command Line Interface

#### Unmunch a dictionary (expand all words from a dictionary file):

```bash
runmunch affix_file.aff dictionary_file.dic
```

Example:
```bash
runmunch hr_HR.aff hr_HR.dic > expanded_words.txt
```

#### Expand specific words using affix rules (`-e`/`--expand` mode):

**Without dictionary** (tries all possible rules):
```bash
echo -e "word1\nword2\nword3" | runmunch -e affix_file.aff
# or
echo -e "word1\nword2\nword3" | runmunch --expand affix_file.aff
```

**With dictionary** (uses word-specific flags for better results):
```bash
echo -e "word1\nword2\nword3" | runmunch -e affix_file.aff dictionary_file.dic
# or
echo -e "word1\nword2\nword3" | runmunch --expand affix_file.aff dictionary_file.dic
```

#### Find base words and expand them (`-b`/`--find-base` mode):

**Find base forms from inflected words and expand them** (requires dictionary):
```bash
echo -e "cats\nwalked\nbooks" | runmunch -e -b affix_file.aff dictionary_file.dic
# or
echo -e "cats\nwalked\nbooks" | runmunch --expand --find-base affix_file.aff dictionary_file.dic
```

This mode:
1. Analyzes inflected forms (e.g., "cats", "walked", "books")
2. Finds their base words (e.g., "cat", "walk", "book")
3. Expands the base words using their dictionary flags
4. Returns all possible forms of the base words

**Examples:**
```bash
# German words - expand base forms
echo -e "Haus\nAuto\nKind" | runmunch -e de.aff de.dic

# Croatian words - expand base forms
echo -e "kuća\nkava\ngrad" | runmunch -e hr_HR.aff hr_HR.dic

# English inflected forms - find base and expand
echo -e "cats\nwalked\nbooks" | runmunch -e -b en.aff en.dic
```

### Library Usage

```rust
use runmunch::{Runmunch, WordExpander, AffixFile};

// Create a new Runmunch instance
let mut runmunch = Runmunch::new();

// Load affix and dictionary files
runmunch.load_affix_file("path/to/file.aff")?;
runmunch.load_dictionary("path/to/file.dic")?;

// Expand all words from the dictionary
let expanded_words = runmunch.unmunch()?;
for word in expanded_words {
    println!("{}", word);
}

// Or expand specific words
let word_forms = runmunch.expand_word("example")?;
for form in word_forms {
    println!("{}", form);
}

// Find base word and expand it
let expanded_forms = runmunch.find_base_and_expand("examples")?;
for form in expanded_forms {
    println!("{}", form);
}
```

#### Using the WordExpander directly:

```rust
use runmunch::{WordExpander, AffixFile};

// Load affix file
let affix_file = AffixFile::load("path/to/file.aff")?;

// Create expander and set affix file
let mut expander = WordExpander::new();
expander.set_affix_file(&affix_file);

// Expand a word with specific flags
let expanded = expander.expand_with_flags("work", &["ED".to_string()])?;
// Results might include: ["work", "worked"]

// Find base word from inflected form
let base_words = expander.find_base_word("worked", &dictionary)?;
// Results might include: ["work"]

// Find base and expand
let all_forms = expander.find_base_and_expand("worked", &dictionary)?;
// Results might include: ["work", "worked", ...]
```

## File Formats

### Affix Files (.aff)

Runmunch supports hunspell affix file format with features like:

- Prefix rules (`PFX`)
- Suffix rules (`SFX`)
- Cross-product flags for combining prefixes and suffixes
- Condition patterns using regular expressions
- Long flags (`FLAG long`)

Example affix file:
```
FLAG long

PFX UN Y 1
PFX UN 0 un .

SFX ED Y 1
SFX ED 0 ed .

SFX S Y 1
SFX S 0 s .
```

### Dictionary Files (.dic)

Standard hunspell dictionary format:
```
3
hello/ED
world
test/UN,S
```

- First line contains word count
- Each subsequent line contains a word optionally followed by flags after `/`

## Examples

### Basic Word Expansion

```bash
# Create a simple affix file
cat > simple.aff << EOF
PFX UN Y 1
PFX UN 0 un .

SFX ED Y 1
SFX ED 0 ed .
EOF

# Test expansion
echo "happy" | runmunch --expand simple.aff
# Output: happy, unhappy
```

### Croatian Language Example

```bash
# Expand Croatian words
echo -e "kuća\nčitati" | runmunch --expand hunspell-hr/hr_HR.aff

# Unmunch Croatian dictionary
runmunch hunspell-hr/hr_HR.aff hunspell-hr/hr_HR.dic | wc -l
# Shows total expanded words
```

## 🚀 Performance Benchmarks

Runmunch delivers excellent performance across different languages and use cases:

### Dictionary Unmunching (Full Expansion)

| Language | Input Words | Output Words | Time | Speed | Expansion Ratio |
|----------|-------------|--------------|------|-------|----------------|
| **German** | 75,888 | 1,226,445 | 3.23s | 23,493 w/s | 16.16x |
| **Croatian** | 53,712 | 28,428,780 | 52.63s | 1,020 w/s | 529.28x |

### Word Expansion (`--expand` mode)

| Mode | Language | Input | Output | Time | Speed |
|------|----------|-------|--------|------|-------|
| **No Dict** | German | 10 words | 528 forms | 0.023s | 435 w/s |
| **No Dict** | Croatian | 10 words | 2,015 forms | 0.027s | 370 w/s |
| **With Dict** | German | 10 words | 236 forms | 0.073s | 137 w/s |
| **With Dict** | Croatian | 10 words | 1,515 forms | 0.081s | 123 w/s |

### Key Performance Features
- **Zero-cost abstractions** - Leverages Rust's performance guarantees
- **Memory efficient** - Optimized data structures and algorithms
- **Unicode aware** - Proper handling of international characters
- **Scalable** - Performance scales reasonably with morphological complexity

## Compatibility

- **Hunspell format**: Full compatibility with standard hunspell .aff and .dic files
- **Languages**: Extensively tested with German (de) and Croatian (hr_HR), supports any hunspell language
- **Flag systems**: Supports single flags, long flags (`FLAG long`), and flag aliases (`AF` directive)
- **Morphology**: Handles simple (Germanic) to complex (Slavic) morphological systems
- **Platforms**: Cross-platform (Linux, macOS, Windows)
- **Unicode**: Full UTF-8 support for international characters

## API Documentation

The main components of the library:

### `Runmunch`
The main interface combining affix files and dictionaries.

### `WordExpander`
Core word expansion logic using affix rules.

### `AffixFile`
Parser and representation of hunspell affix files.

### `Dictionary`
Parser and representation of hunspell dictionary files.

## Error Handling

Runmunch uses comprehensive error handling with descriptive error messages:

```rust
use runmunch::RunmunchError;

match runmunch.load_affix_file("invalid.aff") {
    Ok(_) => println!("Success!"),
    Err(RunmunchError::Io(e)) => eprintln!("IO error: {}", e),
    Err(RunmunchError::InvalidAffix(msg)) => eprintln!("Invalid affix: {}", msg),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Contributing

Contributions are welcome! Please feel free to:

1. Report bugs
2. Suggest features
3. Submit pull requests
4. Improve documentation

## License

Licensed under MIT OR Apache-2.0.

## Acknowledgments

- Based on hunspell's unmunch tool by Németh László and contributors
- Inspired by Lingua::Spelling::Alternative Perl module by Dobrica Pavlinušić
