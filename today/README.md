# today

A small Rust command-line app to collect and display events from multiple providers.

## Overview

`today` reads a TOML config file that defines event providers. Supported provider kinds are:

- `text` — plain text event files
- `csv` — CSV event files
- `sqlite` — SQLite database event source
- `web` — remote JSON web endpoint

The app prints events for a specific date (default is today) and supports filtering by categories, excluding categories, and text search.

## Configuration

`today` loads configuration from the OS config directory for the app name `today`.

- Linux/macOS: `$XDG_CONFIG_HOME/today/today.toml` or `~/.config/today/today.toml`
- Windows: `%APPDATA%\today\today.toml`

If the config file does not exist, the app will create an empty one on first startup.

### Example config

```toml

[[providers]]
name = "sqlite"
kind = "sqlite"
resource = "events.db"

[[providers]]
name = "Web"
kind = "web"
resource = *URL*

[[providers]]
name = "txt"
kind = "text"
resource = "test.txt"

[[providers]]
name = "csv"
kind = "csv"
resource = "test.csv"
```

### Resource paths

- For `text`, `csv`, and `sqlite` providers, `resource` is resolved relative to the config folder.
- For `web` providers, `resource` should be a full URL.

## Provider formats

### `text`

Text event files use one event per block of lines. Each event block should contain:

1. `date`
2. `description`
3. `category`
4. blank separator line

Example:

```text
2001-01-01
testing again
testing/test

--02-14
Valentine's Day
annual/holiday

first monday in january
Rule based event
rule-based/January

```

Supported date formats in text files:

- `YYYY-MM-DD` — singular dated event
- `--MM-DD` — annual recurring event
- rule text such as `first monday in january`

Categories are parsed as `primary` or `primary/secondary`.

### `csv`

CSV files must be comma-separated without headers, with one event per row:

```csv
1759-01-15,"The British Museum opens to the public.",culture/museum
```

Supported date formats in CSV files:

- `YYYY-MM-DD`
- `--MM-DD`
- rule text such as `first monday in january`

### `sqlite`

The SQLite provider expects the database to contain at least these tables and columns:

- `category(category_id, primary_name, secondary_name)`
- `event(event_date, event_description, category_id)`

The provider looks up categories by `category_id` and matches `event_date` using the month/day portion.

### `web`

The web provider calls the configured URL with a `date=MM-DD` query parameter.

The endpoint should return JSON like this:

```json
[
  {
    "date": "2026-01-01",
    "description": "New Year's Day",
    "category": "holiday"
  }
]
```

## Usage

```bash
today [OPTIONS] [COMMAND]
```

### Options

- `-d, --date MMDD` — use a specific month/day instead of today
- `-x, --exclude <categories>` — comma-separated categories to exclude
- `-c, --categories <categories>` — comma-separated categories to include
- `-t, --text <text>` — search text in event descriptions
- `-n, --no-birthday` — disable birthday message

### Category filter syntax

- `primary/secondary` — exact category match
- `primary` — matches any event with that category as primary or secondary category
- `primary/*` — wildcard secondary match for the given primary category

Examples:

- `computing` matches `computing` and `computing/technology` and `technology/computing`
- `culture/museum` matches only that exact category
- `testing/*` matches all `testing/...` categories

### Commands

#### `providers`

List configured provider names.

```bash
cargo run --release -- providers
```

#### `add`

Add an event to a provider that supports write mode (`text` and `csv`).

```bash
cargo run --release -- add \
  --provider-name test_txt \
  --date 2024-12-25 \
  --description "Christmas Day" \
  --category holiday/seasonal
```

You can add anniversary-style events by using a yearless date:

```bash
cargo run --release -- add \
  --provider-name test_txt \
  --date --02-14 \
  --description "Valentine's Day" \
  --category annual/holiday
```

## Examples

Show today's events:

```bash
cargo run
```

Show events for January 1:

```bash
cargo run -- --date 0101
```

Show only `culture/museum` events:

```bash
cargo run -- --categories culture/museum
```

Exclude `testing` and `annual` categories:

```bash
cargo run -- --exclude testing,annual
```

Search event descriptions for `museum`:

```bash
cargo run -- --text museum
```
