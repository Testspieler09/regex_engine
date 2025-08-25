![Build](https://github.com/Testspieler09/regex_engine/actions/workflows/rust.yml/badge.svg)

# Regex Engine

> [!WARNING]
> This project was done for educational purposes only.

This is a very simple regex engine implemented in Rust. It is designed to parse and execute regular expressions.

## Features

- Supports basic regex syntax including character classes, quantifiers (`*`, `+`), and the `.` wildcard.
- Converts regex patterns to finite automata for efficient matching.

> [!NOTE]
> The following characters are supported:
>
> `|`: Or / Union
>
> `(` and `)`: Group
>
> `*`: Kleene star (0 to $$\infty$$)
>
> `+`: Match previous group 1 to $$\infty$$ times
>
> `.`: Dot wildcard that can match any character.

## Usage

Here is a basic example of how to use the regex engine in your Rust code:

```rust
use regex_engine::Regex;

fn main() {
    let pattern = "a*b+";
    let text = "aaabbb";
    let engine = Regex::new(pattern);

    if engine.is_match(text) {
        println!("The text matches the pattern!");
    } else {
        println!("No match found.");
    }
}
```

## API

### `RegexEngine`

- `fn new(pattern: &str) -> Self`
  - Creates a new `RegexEngine` with the provided pattern.

- `fn is_match(&self, text: &str) -> bool`
  - Checks if the text matches the regex pattern.

- `fn find(&self, text: &str) -> Option<&str>`
  - Finds the first match in the text.

- `fn findall(&self, text: &str) -> Vec<&str>`
  - Finds all non overlapping matches in the specified text.

## Contributing

Contributions are welcome! Please follow these steps to contribute:

1. Fork the repository.
2. Create your feature branch: `git checkout -b feature/my-feature`
3. Commit your changes: `git commit -am 'Add my feature'`
4. Push to the branch: `git push origin feature/my-feature`
5. Create a new Pull Request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Using the fuzzer

### Prerequisites

1. **Install rustup** (if you don't have it):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Install cargo-fuzz**:
   ```bash
   cargo install cargo-fuzz
   ```

### Running the fuzzer

Run the fuzzer with nightly Rust (required for libFuzzer support):

```bash
# Fuzz Thompson construction for 60 seconds
cargo +nightly fuzz run regex_thompson -- -max_total_time=60

# Fuzz Glushkov construction for 60 seconds
cargo +nightly fuzz run regex_glushkov -- -max_total_time=60

# List all available fuzz targets
cargo +nightly fuzz list

# Run indefinitely (stop with Ctrl+C)
cargo +nightly fuzz run regex_thompson
```

### Analyzing crashes

If the fuzzer finds crashes, they'll be saved in `fuzz/artifacts/`:

```bash
# View a crash file
hexdump -C fuzz/artifacts/regex_thompson/crash-<hash>

# Reproduce a specific crash
cargo +nightly fuzz run regex_thompson fuzz/artifacts/regex_thompson/crash-<hash>

# Minimize a crashing input
cargo +nightly fuzz tmin regex_thompson fuzz/artifacts/regex_thompson/crash-<hash>
```

### Useful options

```bash
# Run with multiple workers (parallel fuzzing)
cargo +nightly fuzz run regex_thompson -- -workers=4

# Run for specific number of iterations
cargo +nightly fuzz run regex_thompson -- -runs=10000

# Show final statistics
cargo +nightly fuzz run regex_thompson -- -print_final_stats=1
```
