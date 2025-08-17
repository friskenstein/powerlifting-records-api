# Powerlifting Records API

Fast, in-memory HTTP API for powerlifting records. On startup it reads CSV files from a directory, builds a normalized Records table, and exposes endpoints to fetch records and review any build-time errors.

## Quick Start

- Requirements: Rust 1.80+ (edition 2024), `cargo`.
- Copy `.env.example` to `.env` and set `CSV_DIR` to your CSV folder. Optionally set `PORT`.
- Place your CSV files under the directory pointed to by `CSV_DIR`.
- Run: `cargo run` (starts the API, builds records from CSVs).

## Configuration

- `PORT`: Port the API listens on (default `3000`).
- `CSV_DIR`: Absolute path to the directory containing CSV files.

Example `.env`:

```
PORT=3000
CSV_DIR=/absolute/path/to/records
```

## How Data Is Built

- The database is in-memory and initialized at process start.
- All `.csv` files in `CSV_DIR` are read and processed in lexicographic filename order.
- Each non-empty, non-comment line (lines starting with `#` are ignored) is parsed and, if valid, used to update the current best record for a specific key.
- A record only updates if the incoming weight is strictly higher than the current stored weight; otherwise the line is logged as an error.
- Any line with an invalid key format or a key that does not exist in the allowable combinations is logged as an error.
- Use the `/errors` endpoint to inspect build-time issues (invalid keys, non-increasing attempts, etc.).

## CSV File Naming Guidelines

- Files are applied in lexicographic order. To ensure chronological application of updates, use sortable prefixes, e.g.:
  - `2024-01-initial.csv`
  - `2024-03-open-meet.csv`
  - `2024-06-nationals.csv`
- Any filename is accepted as long as it ends with `.csv`, but using ISO date prefixes is strongly recommended.

## CSV Format

Each data line MUST contain 6 comma-separated fields. Double quotes may be used to wrap fields that contain commas. Lines starting with `#` are ignored.

Field order:

1. `key`: `sex|div|event|equip|class|lift`
2. `weight`: number (kg)
3. `name`: lifter name (string)
4. `date`: meet date (string, free-form; ISO recommended: `YYYY-MM-DD`)
5. `place`: meet/location name (string)
6. `comment`: free text (optional, currently ignored by the builder)

Example lines:

```
M|Open|SBD|Sleeves|82.5|S, 305.0, "John Doe", 2024-03-10, "City Open", ""
F|Open|B|Raw|60|B, 120.0, "Jane Smith", 2023-11-20, "Regional Bench", "National qualifier"
```

### Key Format and Allowed Values

- `sex`: `M` or `F`.
- `div` (division): one of `Open`, `Youth`, `T13-15`, `T16-17`, `T18-19`, `J20-23`, `M40-44`, `M45-49`, `M50-54`, `M55-59`, `M60-64`, `M65-69`, `M70-74`, `M75-79`, `M80+`, plus an optional drug-tested variant formed by appending `-D` (e.g., `Open-D`).
- `event`: one of `SBD`, `B`, `D`.
- `equip` (equipment): allowed per lift/event:
  - For `S` or `SBD` lifts (with `event=SBD`): `Bare`, `Sleeves`, `Wraps`, `Single-ply`, `Multi-ply`. (`Raw` is NOT allowed for squat or total.)
  - For `B` or `D` lifts (with `event=B` or `event=D` respectively): `Raw`, `Single-ply`, `Multi-ply`. (`Bare`/`Sleeves`/`Wraps` are NOT allowed for bench or deadlift.)
- `class` (weight class):
  - Male: `52`, `56`, `60`, `67.5`, `75`, `82.5`, `90`, `100`, `110`, `125`, `140`, `SHW`
  - Female: `44`, `48`, `52`, `56`, `60`, `67.5`, `75`, `82.5`, `90`, `100`, `110`, `SHW`
- `lift`: one of `S`, `B`, `D`, `SBD`, constrained by event:
  - `event=SBD` allows `lift=S`, `lift=SBD`.
  - `event=B` requires `lift=B`.
  - `event=D` requires `lift=D`.

Notes:

- The service pre-populates every valid key combination and only updates those entries. If a CSV line specifies a key that does not map to a valid combination, it is recorded in `/errors` with reason `invalid key (no matching record)`.
- If the `key` does not contain exactly 6 pipe-separated segments in the required order, it is recorded with reason `invalid key format`.

## API

Base URL: `http://localhost:${PORT}` (default `http://localhost:3000`).

### GET `/records`

Returns the current best records. Optional exact-match filters:

- `sex`, `div`, `event`, `equip`, `class`, `lift`

Example requests:

- All records: `/records`
- Filtered: `/records?sex=F&div=Open&event=SBD&equip=Sleeves&class=67.5&lift=S`

Response item fields:

- `sex`, `div`, `event`, `equip`, `class`, `lift`, `weight`, `name`, `date`, `place`

Notes:

- Exact string matching is used for filters.
- There is currently no ordering parameter; results reflect database order. (A dedicated ORDER BY is planned.)

### GET `/errors`

Returns build-time errors recorded while parsing CSVs. Sorted by `file`, `line`.

Response item fields:

- `file`, `line`, `key`, `weight`, `name`, `date`, `place`, `reason`

Typical reasons include:

- `invalid key format` (malformed key)
- `invalid key (no matching record)` (combination not allowed)
- `not higher than previous` (attempt does not exceed current record)

## Developer Guide

- Run locally: `cargo run` (reads `.env`, builds DB, starts server).
- Hot reload: not built-in; restart the server to rebuild from CSVs.
- Database: in-memory SQLite via `rusqlite`. Schema is defined in `schema.sql` and loaded at startup.
- Dependencies: Axum (HTTP), Tokio (async), Serde/JSON, rusqlite (bundled), dotenvy, csv (crate present; custom parsing is used in builder).

## Data Refresh

- Edit or add CSV files under `CSV_DIR` and restart the server to rebuild and apply changes.
- A rebuild endpoint is planned for future updates.

## Examples

Minimal dataset in a single file `2024-01-initial.csv`:

```
# Open Female 60 kg Bench, Raw
F|Open|B|Raw|60|B, 90.0, "Alice Example", 2024-01-15, "Winter Classic", "Initial record"

# Open Male 82.5 kg Squat, Sleeves (SBD event)
M|Open|SBD|Sleeves|82.5|S, 250.0, "Bob Example", 2024-01-20, "City Open", "PR"
```

