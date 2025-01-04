use clap::Parser;
use colored::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use regex::Regex;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Parser, Debug)]
#[command(
    name = "Rusty Scout",
    about = "A blazingly fast file search tool written in Rust",
    version = "1.0.0",
    author = "Your Name"
)]
struct Args {
    /// Directory to search in
    #[arg(short, long, default_value = ".")]
    directory: String,

    /// Pattern to search for
    #[arg(short, long)]
    pattern: String,

    /// File extensions to search (comma-separated)
    #[arg(short, long, default_value = "*")]
    extensions: String,

    /// Use regex for pattern matching
    #[arg(short, long)]
    regex: bool,

    /// Ignore case when searching
    #[arg(short, long)]
    ignore_case: bool,
}

#[derive(Debug)]
struct SearchResult {
    file_path: PathBuf,
    line_number: usize,
    line: String,
    matches: Vec<(usize, usize)>, // (start, end) positions of matches
}

#[derive(Debug)]
struct SearchError {
    kind: String,
    message: String,
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl Error for SearchError {}

struct FileSearcher {
    pattern: String,
    regex: Option<Regex>,
    ignore_case: bool,
    extensions: Vec<String>,
    results: Arc<Mutex<Vec<SearchResult>>>,
    progress: Arc<MultiProgress>,
}

impl FileSearcher {
    fn new(args: &Args) -> Result<Self, Box<dyn Error>> {
        let regex = if args.regex {
            Some(Regex::new(&args.pattern)?)
        } else {
            None
        };

        let extensions = args
            .extensions
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        Ok(FileSearcher {
            pattern: args.pattern.clone(),
            regex,
            ignore_case: args.ignore_case,
            extensions,
            results: Arc::new(Mutex::new(Vec::new())),
            progress: Arc::new(MultiProgress::new()),
        })
    }

    fn search_file(&self, path: &Path) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        let contents = fs::read_to_string(path)?;
        let mut file_results = Vec::new();

        for (line_number, line) in contents.lines().enumerate() {
            let matches = if let Some(regex) = &self.regex {
                regex
                    .find_iter(line)
                    .map(|m| (m.start(), m.end()))
                    .collect::<Vec<_>>()
            } else {
                let search_line = if self.ignore_case {
                    line.to_lowercase()
                } else {
                    line.to_string()
                };
                let search_pattern = if self.ignore_case {
                    self.pattern.to_lowercase()
                } else {
                    self.pattern.clone()
                };

                let mut matches = Vec::new();
                let mut start = 0;
                while let Some(pos) = search_line[start..].find(&search_pattern) {
                    let abs_pos = start + pos;
                    matches.push((abs_pos, abs_pos + search_pattern.len()));
                    start = abs_pos + 1;
                }
                matches
            };

            if !matches.is_empty() {
                file_results.push(SearchResult {
                    file_path: path.to_path_buf(),
                    line_number: line_number + 1,
                    line: line.to_string(),
                    matches,
                });
            }
        }

        Ok(file_results)
    }

    fn should_search_file(&self, path: &Path) -> bool {
        if self.extensions.contains(&"*".to_string()) {
            return true;
        }

        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| self.extensions.contains(&ext.to_string()))
            .unwrap_or(false)
    }

    fn search_dir(&self, dir: &Path) -> Result<(), Box<dyn Error>> {
        let walker = ignore::WalkBuilder::new(dir)
            .hidden(true)
            .git_ignore(true)
            .build();

        let pb = self.progress.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );

        let files: Vec<_> = walker
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().map_or(false, |ft| ft.is_file()))
            .filter(|entry| self.should_search_file(entry.path()))
            .collect();

        pb.set_message(format!("Searching {} files...", files.len()));

        files.par_iter().for_each(|entry| {
            if let Ok(results) = self.search_file(entry.path()) {
                if !results.is_empty() {
                    let mut all_results = self.results.lock().unwrap();
                    all_results.extend(results);
                }
            }
        });

        pb.finish_with_message(format!("Searched {} files", files.len()));
        Ok(())
    }

    fn display_results(&self) {
        let results = self.results.lock().unwrap();
        
        if results.is_empty() {
            println!("{}", "No matches found.".yellow());
            return;
        }

        println!(
            "\n{} {} matches found:\n",
            "✓".green(),
            results.len().to_string().green()
        );

        for result in results.iter() {
            println!(
                "{}:{}:",
                result.file_path.display().to_string().blue(),
                result.line_number.to_string().yellow()
            );

            let mut line = result.line.clone();
            // Highlight matches in reverse order to not invalidate positions
            for (start, end) in result.matches.iter().rev() {
                line.insert_str(*end, "\x1b[0m");
                line.insert_str(*start, "\x1b[31m");
            }
            println!("    {}", line);
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    
    println!(
        "{} {} v{}",
        "🔍".green(),
        "Rusty Scout".bright_green(),
        env!("CARGO_PKG_VERSION")
    );

    let searcher = FileSearcher::new(&args)?;
    searcher.search_dir(Path::new(&args.directory))?;
    searcher.display_results();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_file_search() -> Result<(), Box<dyn Error>> {
        let dir = tempdir()?;
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path)?;
        writeln!(file, "Hello, world!\nThis is a test file.\nHello again!")?;

        let args = Args {
            directory: dir.path().to_string_lossy().to_string(),
            pattern: "Hello".to_string(),
            extensions: "*".to_string(),
            regex: false,
            ignore_case: false,
        };

        let searcher = FileSearcher::new(&args)?;
        let results = searcher.search_file(&file_path)?;

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].line_number, 1);
        assert_eq!(results[1].line_number, 3);

        Ok(())
    }

    #[test]
    fn test_case_insensitive_search() -> Result<(), Box<dyn Error>> {
        let dir = tempdir()?;
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path)?;
        writeln!(file, "Hello, HELLO, hello")?;

        let args = Args {
            directory: dir.path().to_string_lossy().to_string(),
            pattern: "hello".to_string(),
            extensions: "*".to_string(),
            regex: false,
            ignore_case: true,
        };

        let searcher = FileSearcher::new(&args)?;
        let results = searcher.search_file(&file_path)?;

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].matches.len(), 3);

        Ok(())
    }
}