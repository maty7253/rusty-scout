# 🔍 Rusty Scout

A blazingly fast file search tool written in Rust that helps you find text patterns in your codebase with ease.

## Features

- 🚀 **Blazingly Fast**: Utilizes parallel processing for maximum performance
- 🎯 **Smart Search**: Supports regex patterns and case-insensitive search
- 🎨 **Syntax Highlighting**: Colorized output for better readability
- 📁 **Git-Aware**: Respects `.gitignore` rules
- 🔧 **Flexible**: Filter by file extensions and customize search patterns
- 💡 **User-Friendly**: Progress indicators and clear search results

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/maty7253/rusty-scout.git

# Navigate to the project directory
cd rusty-scout

# Build and install
cargo install --path .
```

## Usage

```shellscript
# Basic search
rusty-scout -p "pattern" -d "/path/to/search"

# Search with regex
rusty-scout -p "^Hello.*world$" -d "/path/to/search" -r

# Search specific file types
rusty-scout -p "TODO" -d "/path/to/search" -e "rs,txt,md"

# Case-insensitive search
rusty-scout -p "ERROR" -d "/path/to/search" -i
```

### Command Line Options

| Option | Description
|-----|-----
| `-d, --directory` | Directory to search in (default: current directory)
| `-p, --pattern` | Pattern to search for
| `-e, --extensions` | File extensions to search (comma-separated)
| `-r, --regex` | Use regex for pattern matching
| `-i, --ignore-case` | Ignore case when searching


## Examples

Search for a pattern in all files:

```shellscript
rusty-scout -p "TODO" -d "./src"
```

Search only in Rust files:

```shellscript
rusty-scout -p "unwrap()" -d "./src" -e "rs"
```

Case-insensitive regex search:

```shellscript
rusty-scout -p "error.*failed" -d "./logs" -r -i
```

## Development

### Prerequisites

- Rust 1.70 or higher
- Cargo


### Building

```shellscript
cargo build --release
```

### Running Tests

```shellscript
cargo test
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request


## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Uses [clap](https://github.com/clap-rs/clap) for CLI argument parsing
- Uses [rayon](https://github.com/rayon-rs/rayon) for parallel processing
- Uses [regex](https://github.com/rust-lang/regex) for pattern matching
