use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use globset::{Glob, GlobSetBuilder};
use ignore::WalkBuilder;
use regex::Regex;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    name = "fnr",
    about = "Fast file and directory name search and rename tool",
    long_about = "A high-performance tool for searching and batch renaming files and directories \
                   with regex support, interactive confirmation, and safety features."
)]
struct Cli {
    #[arg(help = "Pattern to search for (or old pattern for rename)")]
    pattern: String,

    #[arg(help = "New pattern for rename (if provided, enables rename mode)")]
    replacement: Option<String>,

    #[arg(
        help = "Glob patterns to match (e.g., '*.rs', '**/*.{h,cpp}', '!target/**')"
    )]
    glob_patterns: Vec<String>,

    #[arg(
        short = 'd',
        long = "base-dir",
        default_value = ".",
        help = "Base directory to search from"
    )]
    base_dir: PathBuf,

    #[arg(
        short = 'r',
        long = "regex",
        help = "Enable regular expression matching"
    )]
    regex: bool,

    #[arg(
        short = 't',
        long = "type",
        value_enum,
        default_value = "both",
        help = "Filter by file type"
    )]
    file_type: FileType,

    #[arg(
        long = "dry-run",
        help = "Show what would be renamed without executing"
    )]
    dry_run: bool,

    #[arg(
        long = "no-interactive",
        help = "Apply all changes without prompts"
    )]
    no_interactive: bool,

    #[arg(
        long = "no-recursive",
        help = "Don't search subdirectories"
    )]
    no_recursive: bool,

    #[arg(
        long = "case-sensitive",
        help = "Case-sensitive matching"
    )]
    case_sensitive: bool,

    #[arg(
        long = "hidden",
        help = "Include hidden files and directories"
    )]
    hidden: bool,

    #[arg(
        long = "no-color",
        help = "Disable colored output"
    )]
    no_color: bool,

    #[arg(
        long = "no-symlink",
        help = "Disable symbolic link follow"
    )]
    no_symlink: bool,

    #[arg(
        long = "no-skip-gitignore",
        help = "Disable .gitignore skip"
    )]
    no_skip_gitignore: bool,

    #[arg(
        long = "max-depth",
        help = "Maximum depth to search"
    )]
    max_depth: Option<usize>,

    #[arg(
        long = "min-depth",
        help = "Minimum depth to search"
    )]
    min_depth: Option<usize>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum FileType {
    File,
    Dir,
    Both,
}

#[derive(Debug)]
enum ConfirmResult {
    Yes,
    No,
    All,
    Quit,
}

#[derive(Debug)]
struct Match {
    path: PathBuf,
    new_name: String,
    is_dir: bool,
    pattern: String,
    replacement: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(replacement) = &cli.replacement {
        // Rename mode
        rename_mode(&cli, replacement)
    } else {
        // Search mode
        search_mode(&cli)
    }
}

fn search_mode(cli: &Cli) -> Result<()> {
    let matches = find_matches(cli, None)?;
    
    for m in matches {
        let type_indicator = if m.is_dir { "d" } else { "f" };
        let path_str = m.path.display().to_string();
        
        if cli.no_color {
            println!("[{}] {}", type_indicator, path_str);
        } else {
            let colored_type = if m.is_dir {
                type_indicator.blue().bold()
            } else {
                type_indicator.green().bold()
            };
            println!("[{}] {}", colored_type, path_str.white());
        }
    }
    
    Ok(())
}

fn rename_mode(cli: &Cli, replacement: &str) -> Result<()> {
    let matches = find_matches(cli, Some(replacement))?;
    
    if matches.is_empty() {
        println!("No matches found.");
        return Ok(());
    }

    if cli.dry_run {
        let header = if cli.no_color {
            "Dry run - showing what would be renamed:"
        } else {
            &"Dry run - showing what would be renamed:".yellow().to_string()
        };
        println!("{}", header);
        
        for m in matches {
            if cli.no_color {
                println!("    {}", m.path.display());
                println!(" -> {}", m.new_name);
            } else {
                let old_filename = m.path.file_name().unwrap().to_str().unwrap();
                let parent_path = if let Some(parent) = m.path.parent() {
                    format!("{}/", parent.display())
                } else {
                    String::new()
                };
                
                println!("    {}{}", 
                    parent_path.white(),
                    highlight_pattern(old_filename, &cli.pattern, cli.no_color)
                );
                println!(" -> {}{}", 
                    parent_path.white(),
                    highlight_replacement(&m.new_name, old_filename, &cli.pattern, replacement, cli.no_color)
                );
            }
        }
        return Ok(());
    }

    if !cli.no_interactive {
        let mut apply_all = false;
        for m in matches {
            if !apply_all {
                match confirm_rename(&m, cli.no_color)? {
                    ConfirmResult::Yes => {},
                    ConfirmResult::No => continue,
                    ConfirmResult::All => apply_all = true,
                    ConfirmResult::Quit => return Ok(()),
                }
            }
            perform_rename(&m, cli.no_color)?;
        }
    } else {
        for m in matches {
            perform_rename(&m, cli.no_color)?;
        }
    }

    Ok(())
}

fn find_matches(cli: &Cli, replacement: Option<&str>) -> Result<Vec<Match>> {
    let mut matches = Vec::new();
    
    let regex = if cli.regex {
        Some(build_regex(&cli.pattern, cli.case_sensitive)?)
    } else {
        None
    };

    // Build glob set from patterns
    let mut glob_builder = GlobSetBuilder::new();
    let patterns = if cli.glob_patterns.is_empty() {
        vec!["**/*".to_string()]
    } else {
        cli.glob_patterns.clone()
    };
    
    for pattern in &patterns {
        glob_builder.add(Glob::new(pattern)?);
    }
    let glob_set = glob_builder.build()?;

    // Build walker with gitignore support
    let mut walker_builder = WalkBuilder::new(&cli.base_dir);
    walker_builder
        .follow_links(!cli.no_symlink)
        .git_ignore(!cli.no_skip_gitignore)
        .hidden(cli.hidden);
    
    if cli.no_recursive {
        walker_builder.max_depth(Some(1));
    } else if let Some(max_depth) = cli.max_depth {
        walker_builder.max_depth(Some(max_depth));
    }
    
    let walker = walker_builder.build();

    for result in walker {
        let entry = match result {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Warning: {}", e);
                continue;
            }
        };
        
        let path = entry.path();
        
        // Check if path matches any glob pattern
        if !glob_set.is_match(path) {
            continue;
        }
        
        let is_dir = path.is_dir();
        
        // Filter by type
        match cli.file_type {
            FileType::File if is_dir => continue,
            FileType::Dir if !is_dir => continue,
            _ => {}
        }

        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if let Some(new_name) = check_match(filename, &cli.pattern, replacement, &regex, cli.case_sensitive) {
            matches.push(Match {
                path: path.to_path_buf(),
                new_name,
                is_dir,
                pattern: cli.pattern.clone(),
                replacement: replacement.unwrap_or("").to_string(),
            });
        }
    }

    // Sort matches: files first, then directories (by depth, deepest first)
    matches.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (false, true) => std::cmp::Ordering::Less,  // Files before dirs
            (true, false) => std::cmp::Ordering::Greater, // Dirs after files
            _ => {
                // Same type: sort by depth (deepest first for dirs, any order for files)
                let a_depth = a.path.components().count();
                let b_depth = b.path.components().count();
                if a.is_dir {
                    b_depth.cmp(&a_depth) // Deepest dirs first
                } else {
                    a_depth.cmp(&b_depth) // Shallowest files first
                }
            }
        }
    });

    Ok(matches)
}

fn build_regex(pattern: &str, case_sensitive: bool) -> Result<Regex> {
    let mut builder = regex::RegexBuilder::new(pattern);
    builder.case_insensitive(!case_sensitive);
    builder.build().context("Invalid regex pattern")
}

fn check_match(
    filename: &str,
    pattern: &str,
    replacement: Option<&str>,
    regex: &Option<Regex>,
    case_sensitive: bool,
) -> Option<String> {
    if let Some(regex) = regex {
        if let Some(replacement) = replacement {
            if regex.is_match(filename) {
                Some(regex.replace_all(filename, replacement).to_string())
            } else {
                None
            }
        } else {
            if regex.is_match(filename) {
                Some(filename.to_string())
            } else {
                None
            }
        }
    } else {
        // Simple glob-like matching
        let matches = if case_sensitive {
            simple_match(filename, pattern)
        } else {
            simple_match(&filename.to_lowercase(), &pattern.to_lowercase())
        };

        if matches {
            if let Some(replacement) = replacement {
                Some(simple_replace(filename, pattern, replacement, case_sensitive))
            } else {
                Some(filename.to_string())
            }
        } else {
            None
        }
    }
}

fn simple_match(text: &str, pattern: &str) -> bool {
    if pattern.contains('*') {
        // Basic glob matching
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            text.starts_with(parts[0]) && text.ends_with(parts[1])
        } else {
            text.contains(&pattern.replace('*', ""))
        }
    } else {
        text.contains(pattern)
    }
}

fn simple_replace(text: &str, pattern: &str, replacement: &str, case_sensitive: bool) -> String {
    if case_sensitive {
        text.replace(pattern, replacement)
    } else {
        // Case-insensitive replace (simple version)
        let lower_text = text.to_lowercase();
        let lower_pattern = pattern.to_lowercase();
        if let Some(pos) = lower_text.find(&lower_pattern) {
            let mut result = text.to_string();
            result.replace_range(pos..pos + pattern.len(), replacement);
            result
        } else {
            text.to_string()
        }
    }
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}

fn highlight_replacement(new_name: &str, old_name: &str, pattern: &str, replacement: &str, no_color: bool) -> String {
    if no_color {
        new_name.to_string()
    } else {
        // Find where the replacement happened
        if let Some(pos) = old_name.to_lowercase().find(&pattern.to_lowercase()) {
            let before = &old_name[..pos];
            let after = &old_name[pos + pattern.len()..];
            format!("{}{}{}", 
                before.white(), 
                replacement.yellow(), 
                after.white()
            )
        } else {
            new_name.white().to_string()
        }
    }
}

fn highlight_pattern(text: &str, pattern: &str, no_color: bool) -> String {
    if no_color {
        text.to_string()
    } else {
        if let Some(pos) = text.to_lowercase().find(&pattern.to_lowercase()) {
            let before = &text[..pos];
            let matched = &text[pos..pos + pattern.len()];
            let after = &text[pos + pattern.len()..];
            format!("{}{}{}", before.white(), matched.yellow(), after.white())
        } else {
            text.white().to_string()
        }
    }
}

fn confirm_rename(m: &Match, no_color: bool) -> Result<ConfirmResult> {
    if no_color {
        println!("    {}", m.path.display());
        println!(" -> {}", m.new_name);
        print!("Replace filename/dirname? [Y]es/[n]o/[a]ll/[q]uit: ");
    } else {
        let old_filename = m.path.file_name().unwrap().to_str().unwrap();
        let parent_path = if let Some(parent) = m.path.parent() {
            format!("{}/", parent.display())
        } else {
            String::new()
        };
        
        println!("    {}{}", 
            parent_path.white(),
            highlight_pattern(old_filename, &m.pattern, no_color)
        );
        println!(" -> {}{}", 
            parent_path.white(),
            highlight_replacement(&m.new_name, old_filename, &m.pattern, &m.replacement, no_color)
        );
        print!("{} ", "Replace filename/dirname? [Y]es/[n]o/[a]ll/[q]uit:".cyan());
    }
    io::stdout().flush()?;
    
    enable_raw_mode()?;
    let result = loop {
        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    print!("\ry");
                    io::stdout().flush()?;
                    break Ok(ConfirmResult::Yes);
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    print!("\rn");
                    io::stdout().flush()?;
                    break Ok(ConfirmResult::No);
                }
                KeyCode::Char('a') | KeyCode::Char('A') => {
                    print!("\ra");
                    io::stdout().flush()?;
                    break Ok(ConfirmResult::All);
                }
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    print!("\rq");
                    io::stdout().flush()?;
                    break Ok(ConfirmResult::Quit);
                }
                KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    print!("\r^C");
                    io::stdout().flush()?;
                    break Ok(ConfirmResult::Quit);
                }
                _ => continue,
            }
        }
    };
    disable_raw_mode()?;
    println!(); // Add newline after the choice
    result
}

fn perform_rename(m: &Match, no_color: bool) -> Result<()> {
    let parent = m.path.parent().unwrap_or(Path::new("."));
    let new_path = parent.join(&m.new_name);
    
    fs::rename(&m.path, &new_path)
        .with_context(|| format!("Failed to rename {} to {}", m.path.display(), new_path.display()))?;
    
    if no_color {
        println!("Renamed: {} -> {}", m.path.display(), new_path.display());
    } else {
        println!("{} {} {} {}", 
            "Renamed:".cyan().bold(),
            m.path.display().to_string().white(),
            "->".yellow().bold(),
            new_path.display().to_string().yellow().bold()
        );
    }
    Ok(())
}
