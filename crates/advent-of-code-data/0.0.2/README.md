# advent-of-code-data

A Rust library for fetching puzzle inputs and submitting answers to Advent of Code.

## Quick Start

Install the crate:

```shell
$ cargo add advent-of-code-data
```

Fetch a puzzle input and submit an answer:

```rust,no_run
use advent_of_code_data::{get_input, submit_answer, Day, Part, Year};

fn main() -> anyhow::Result<()> {
    let input = get_input(Day(1), Year(2025))?;
    println!("{}", input);

    submit_answer(42.into(), Part::One, Day(1), Year(2025))?;
    Ok(())
}
```

Before running, you'll need to set your Advent of Code session cookie. See the [Setup](#setup) section below.

## Setup

Your puzzle inputs are personalized. To fetch them, you need your Advent of Code session cookie.

### Finding your Session Cookie

1. Go to https://adventofcode.com and log in
2. Open your browser's developer tools (F12)
3. Navigate to **Application** → **Storage** → **Cookies**
4. Find the **session** cookie and copy its value

### Configuring Your Session

The fastest way to get started is to set the `AOC_SESSION` environment variable:

```shell
export AOC_SESSION=your_session_cookie_here
```

Alternatively, you can create a configuration file. See the [Configuration](#configuration) section for details.

## Configuration

By default, puzzle data is cached locally and encrypted. You only need to specify your session cookie.

### Using Environment Variables

Set your session with the `AOC_SESSION` environment variable:

```shell
export AOC_SESSION=your_session_cookie_here
```

Other environment variables are available for advanced use:

- `AOC_PASSPHRASE`: Custom passphrase for encryption (uses hostname by default)
- `AOC_PUZZLE_DIR`: Custom directory for caching puzzles
- `AOC_SESSIONS_DIR`: Custom directory for session data
- `AOC_CONFIG_PATH`: Path to a specific config file (disables all other config locations)

### Using a Configuration File

Create a `aoc_settings.toml` file in one of these locations:

**Linux:**
- `$XDG_CONFIG_HOME/advent_of_code_data/aoc_settings.toml`
- `$HOME/.aoc_settings.toml`
- `.aoc_settings.toml` (current directory)

**macOS:**
- `$HOME/Library/Application Support/com.smacdo.advent_of_code_data/aoc_settings.toml`
- `$HOME/.aoc_settings.toml`
- `.aoc_settings.toml` (current directory)

**Windows:**
- `%APPDATA%/com.smacdo.advent_of_code_data/config/aoc_settings.toml`
- `%USERPROFILE%/.aoc_settings.toml`
- `.aoc_settings.toml` (current directory)

Example configuration file:

```toml
[client]
session_id = "your_session_cookie_here"
```

Configuration is loaded in this order, with later sources overriding earlier ones:

1. User configuration directory or the user's home directory
2. Current directory (`.aoc_settings.toml`)
3. Environment variables (highest priority)

This lets you keep global settings in your user directory, override them with project-specific settings, and override those with environment variables.

### Storing Puzzles in Your Project

If you want to cache puzzles directly in your project (instead of the system cache), set both `puzzle_dir` and a custom `passphrase`:

```toml
[client]
session_id = "your_session_cookie_here"
puzzle_dir = "./puzzles"
passphrase = "your_custom_passphrase"
```

**Important:** Do not commit your passphrase or session cookie to version control. Add your config file to `.gitignore` if it contains secrets.

## Troubleshooting

### Invalid or Expired Session

If you get a `BadSessionId` error, your session cookie has expired or is incorrect.

**Solution:**
- Log in to https://adventofcode.com in your browser to refresh your session
- Get a new session cookie and update your `AOC_SESSION` environment variable or config file

### Puzzle Not Available Yet

AoC puzzles unlock at midnight EST each day. If you try to fetch a puzzle before it's released, you'll get a `PuzzleNotFound` error.

**Solution:**
- Check https://adventofcode.com to see if the puzzle is available
- Try again after the puzzle unlocks

### Answer Submission Rate Limited

If you submit too many incorrect answers, AoC temporarily blocks further submissions and returns a `SubmitTimeOut` error.

**Solution:**
- Wait the amount of time indicated in the error message before trying again
- Consider testing your solution logic locally before submitting