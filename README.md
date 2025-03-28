![Build](https://github.com/Testspieler09/regex_engine/actions/workflows/rust.yml/badge.svg)

# Regex Engine

> [!WARNING]
> This project was done for educational purposes only.

This is a very simple regex engine implemented in Rust. It is designed to parse and execute regular expressions.

## Features

- Supports basic regex syntax including character classes, quantifiers (`*`, `+`), and the `.` wildcard.
- Converts regex patterns to finite automata for efficient matching.

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
  - Finds the first match in the text.

## Contributing

Contributions are welcome! Please follow these steps to contribute:

1. Fork the repository.
2. Create your feature branch: `git checkout -b feature/my-feature`
3. Commit your changes: `git commit -am 'Add my feature'`
4. Push to the branch: `git push origin feature/my-feature`
5. Create a new Pull Request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
