# fnr - File Name Rename 🚀

*Because life's too short to rename files one by one like a caveman*

## What is this sorcery?

`fnr` is a blazingly fast™ file and directory renaming tool that will make you question why you ever used `mv` in a for loop like some kind of shell script peasant. It's like `sed` for filenames, but with more colors, gitignore support, and less existential dread.

## Why does this exist?

Have you ever had 47 files named `component_something.rs` and needed to rename them all to `ui_something.rs`? Have you ever stared at a directory full of `test_*.py` files and wished they were `spec_*.py` instead? Have you ever wanted to batch rename files without writing a bash script that looks like it was written by a caffeinated squirrel?

**This is your salvation.**

## Installation

```bash
cargo install fnr-tool
# Or clone this repo and `cargo build --release` like the cool kids do
```

## Usage

### Basic Renaming (The Bread and Butter)

```bash
# Rename all files containing "old" to "new"
fnr "old" "new"

# Preview what would happen (because trust issues)
fnr "old" "new" --dry-run

# Use specific glob patterns
fnr "component" "ui" "**/*.rs"

# Multiple glob patterns (the power move)
fnr "test" "spec" "**/*.py" "**/*.js" "!node_modules/**"

# With base directory
fnr "old" "new" "**/*.rs" --base-dir /path/to/project
```

### Advanced Wizardry (Multiple Patterns & Exclusions)

```bash
# Match multiple file types at once
fnr "component" "ui" "*.{rs,ts,js}" "*.toml"

# Exclude specific directories
fnr "old" "new" "**/*" "!target/**" "!node_modules/**"

# Search from a different base directory
fnr "config" "settings" "**/*.ini" --base-dir /path/to/configs

# Control search depth
fnr "test" "spec" "**/*.py" --max-depth 3 --min-depth 1
```

### Regex Mode (For the Regex Wizards)

```bash
# Use regex because you're fancy
fnr --regex "test_(.+)" "spec_$1" "**/*.py"

# Rename all your poorly named components
fnr --regex "component_(.+)" "ui_$1" "src/**/*.rs"
```

### Interactive Mode (For the Cautious)

By default, `fnr` will ask you about each rename because it respects your trust issues:

```
./src/old_component.rs -> ./src/new_component.rs
Replace filename/dirname? [Y]es/[n]o/[a]ll/[q]uit:
```

Just press a single key (no Enter required, we're not animals):
- `y` - Yes, rename this file
- `n` - No, skip this one
- `a` - Yes to ALL remaining files (YOLO mode)
- `q` - Quit and pretend this never happened

### Flags for the Flag Enthusiasts

```bash
--dry-run              # See what would happen without commitment
--no-interactive       # YOLO mode (renames everything without asking)
--regex                # Enable regex patterns for the power users
--type=file            # Only rename files
--type=dir             # Only rename directories
--type=both            # Rename everything (default)
--no-recursive         # Stay in current directory like a hermit
--case-sensitive       # Because "Test" ≠ "test" (obviously)
--hidden               # Include hidden files (the secret ones)
--no-color             # Remove all joy from your terminal
--no-symlink           # Don't follow symbolic links (symbolic links are just fancy lies)
--no-skip-gitignore    # Ignore .gitignore files (embrace the chaos, rename ALL the things)
--max-depth N          # Maximum directory depth (because rabbit holes have limits)
--min-depth N          # Minimum directory depth (surface-level peasants need not apply)
--base-dir PATH        # Base directory to search from (teleport your search elsewhere like a wizard)
```

## Examples That Will Change Your Life

### The Classic "I Hate My Naming Convention" Scenario
```bash
# You have: component_button.rs, component_input.rs, component_modal.rs
# You want: ui_button.rs, ui_input.rs, ui_modal.rs
fnr "component" "ui" "**/*.rs"
```

### The "My Tests Are Lying About Being Tests" Fix
```bash
# Convert test_*.py to spec_*.py because you're fancy now
fnr "test_" "spec_" "**/*.py"
```

### The "Multiple File Types" Power Move
```bash
# Rename across multiple file types at once
fnr "old_api" "new_api" "**/*.{rs,ts,js,py}" "**/*.toml" "!target/**"
```

### The "Regex Flex" Move
```bash
# Convert CamelCase to snake_case (sort of)
fnr --regex "([A-Z])" "_$1" --case-sensitive "**/*.rs"
```

### The "I'm Feeling Dangerous" Approach
```bash
# Rename everything with "old" to "new" in the entire project
fnr "old" "new" "**/*" --no-interactive
# (Use with caution, we are not responsible for your life choices)
```

### The "Gitignore Respecting Professional" Method
```bash
# Respects .gitignore by default (like a civilized human)
fnr "component" "ui" "**/*.rs"

# Chaos mode: ignore .gitignore
fnr "component" "ui" "**/*.rs" --no-skip-gitignore
```

## Features That Will Make You Popular at Parties

- 🌈 **Colorized output** - Because monochrome is for printers
- ⚡ **Blazingly fast** - Uses all your CPU cores (probably)
- 🎯 **Smart sorting** - Renames files before directories to avoid chaos
- 🔍 **Multiple glob patterns** - `"**/*.rs" "**/*.toml" "!target/**"` works like magic
- 🎨 **Regex support** - For when you want to feel superior
- 🛡️ **Safe by default** - Interactive mode prevents disasters
- 🚫 **Gitignore aware** - Respects your `.gitignore` automatically
- 📁 **Base directory support** - Search from anywhere
- 🎛️ **Depth control** - `--max-depth` and `--min-depth` for precision
- ⚡ **Single-key interaction** - No Enter key required in interactive mode

## Syntax

```bash
fnr [OPTIONS] <PATTERN> [REPLACEMENT] [GLOB_PATTERNS...] [--base-dir BASE_DIR]
```

### Examples:
```bash
# Single pattern
fnr "old" "new" "**/*.rs"

# Multiple patterns
fnr "component" "ui" "**/*.rs" "**/*.ts" "**/*.js"

# With exclusions
fnr "test" "spec" "**/*.py" "!venv/**" "!__pycache__/**"

# Different base directory
fnr "config" "settings" "**/*.ini" --base-dir /path/to/configs

# Brace expansion support
fnr "old" "new" "*.{rs,toml,lock}"
```

## Color Scheme (Because Aesthetics Matter)

- **White**: File paths and unchanged parts
- **Yellow**: The parts being changed (both old and new)
- **Cyan**: Action prompts
- **Green/Blue**: File type indicators (f for files, d for directories)

## Gitignore Support 🎉

`fnr` respects `.gitignore` files by default! This means:
- No more accidentally renaming files in `target/`, `node_modules/`, or `.git/`
- Follows the same ignore rules as your favorite tools
- Use `--no-skip-gitignore` if you want to live dangerously

## Performance Benchmarks

- **One by one in Finder/Explorer**: Slow death by a thousand clicks
- **Bash script you found on Stack Overflow**: 5 files per minute (plus debugging time)
- **Copying that old script from 2019**: 3 hours to remember how it works
- **fnr**: Instant gratification (dopamine included)

## Hall of Fame (Real User Stories)

*"I renamed 847 React components in 3 seconds. My manager thinks I'm a wizard now."* - Anonymous Frontend Dev

*"Accidentally renamed everything to 'banana' once. Still faster than doing it manually."* - Senior Developer (allegedly)

*"Used fnr to rename 10,000 files during my lunch break. Had time for dessert."* - Performance Artist

*"fnr is so fast, I renamed files that didn't even exist yet."* - Time Traveler

## Performance Notes

- **Multiple patterns**: Uses `globset` for efficient simultaneous pattern matching
- **Directory traversal**: Uses the `ignore` crate for fast, gitignore-aware walking
- **Smart ordering**: Files are renamed before their parent directories
- **Memory efficient**: Streams results instead of loading everything into memory

## Warning Signs You Need This Tool

- You've written `for file in *.txt; do mv "$file" "${file%.txt}.bak"; done` more than once (and felt proud about it)
- You have a folder called "New Folder (47)" and you're not even embarrassed
- You've ever used a GUI file manager to rename files one by one like some kind of masochist
- You think `rename` is a Perl script (it is, and that's the problem)
- You've considered learning sed just for filename manipulation (don't do it, seek help)
- You manually exclude `node_modules` and `target` directories every time like a broken record
- You've rage-quit after accidentally renaming your entire project to "untitled"
- You dream in bash loops and wake up screaming

## Contributing

Found a bug? Want a feature? Think the colors are wrong? Open an issue or PR!

Just remember: this tool was built by someone who got tired of renaming files manually, so the bar for "useful contribution" is pretty low.

## License

MIT - Because sharing is caring, and lawyers are expensive.

## Disclaimer

`fnr` is not responsible for:
- Accidentally renaming your entire home directory to "banana"
- Making your coworkers jealous of your file organization skills
- Causing you to become overly obsessed with perfect naming conventions
- Any existential crises caused by realizing how much time you've wasted renaming files manually
- Addiction to using multiple glob patterns for everything
- Sudden urges to rename random files just because you can
- Your boss asking why you're so productive all of a sudden
- Carpal tunnel syndrome from excessive flag usage

---

*Made with ❤️ and an unhealthy obsession with file organization and rational fear of the sed command*

**Remember**: With great power comes great responsibility. Use `--dry-run` first, kids.
