# abre

Shorten repetitive text for display. Pipe lines in, get disambiguated lines out.

```
$ cat paths | abre
/home/user/proj/foo/src/main.rs  →  …/foo/src/main.rs
/home/user/proj/bar/src/main.rs  →  …/bar/src/main.rs
/home/user/proj/bar/src/lib.rs   →  …/bar/src/lib.rs
```

## Install

```
cargo install abre
```

## Usage

```
<input> | abre [options]
```

| Flag | Description | Default |
|------|-------------|---------|
| `-c <regex>` | Capture group selects the part to shorten | entire line |
| `-p <preset>` | Use a built-in capture preset | |
| `-s <char>` | Segment separator | `/` |
| `--suffix` | Shortest unique suffix | |
| `--truncate [-n N]` | Shorten shared segments to N chars | collapse mode, N=1 |
| `--ellipsis <str>` | Replacement string for removed parts | `…` |
| `--json -k <key>` | JSON line mode, operate on key | |
| `--add-key <key>` | Write shortened to new key | |
| `--keep-original <key>` | Modify in place, save original to key | |

## Presets (`-p`)

Built-in `-c` shortcuts for common patterns:

| Preset | Equivalent `-c` | Shortens |
|--------|-----------------|----------|
| `url-path` | `https?://[^/]+(.*)` | path only, keeps domain |
| `url-full` | `(https?://.*)` | entire URL |
| `url-domain` | `https?://([^/]+)` | domain only |
| `docker` | `([^:]+):.+` | image name, keeps tag |

```bash
cat urls | abre -p url-path
# same as: abre -c 'https?://[^/]+(.*)'
```

## Strategies

**collapse** (default) — replace common prefix with `…`:
```
/home/user/proj/foo/src/main.rs  →  …/foo/src/main.rs
```

**suffix** — shortest unique trailing segments:
```
/home/user/proj/foo/src/main.rs  →  …/foo/src/main.rs
```

**truncate** — shorten shared segments to N chars:
```
/home/user/proj/foo/src/main.rs  →  /h/u/p/foo/src/main.rs
```

## Examples

```bash
# paths
find . -name '*.rs' | abre

# URLs — shorten path only
cat urls | abre -p url-path

# Java packages
cat pkgs | abre -s .
```

## JSON mode

Operate on keys in JSON lines (one object per line).

### Input

```json
{"id": 1, "title": "Issue #42",  "url": "https://github.com/org/frontend/issues/42"}
{"id": 2, "title": "Issue #15",  "url": "https://github.com/org/frontend/pulls/15"}
{"id": 3, "title": "Pipeline",   "url": "https://github.com/org/backend/actions/runs/99"}
{"id": 4, "title": "MR !7",      "url": "https://gitlab.com/team/infra/merge_requests/7"}
```

### Modify in place (default)

```bash
$ cat items.jsonl | abre --json -k url -p url-path
```
```json
{"id": 1, "title": "Issue #42",  "url": "https://github.com/org/frontend/issues/42"}
{"id": 2, "title": "Issue #15",  "url": "https://github.com/org/frontend/pulls/15"}
{"id": 3, "title": "Pipeline",   "url": "https://github.com/org/…/actions/runs/99"}
{"id": 4, "title": "MR !7",      "url": "https://gitlab.com/…/merge_requests/7"}
```

URLs 1 & 2 keep `frontend` because they share the repo — they need `issues` vs `pulls` to disambiguate. URL 3 & 4 are unique at a higher level so more gets collapsed.

### Add new key (`--add-key`)

```bash
$ cat items.jsonl | abre --json -k url -p url-path --add-key short
```
```json
{"id": 1, "title": "Issue #42",  "url": "https://github.com/org/frontend/issues/42", "short": "…/frontend/issues/42"}
{"id": 2, "title": "Issue #15",  "url": "https://github.com/org/frontend/pulls/15",  "short": "…/frontend/pulls/15"}
{"id": 3, "title": "Pipeline",   "url": "https://github.com/org/backend/actions/runs/99", "short": "…/backend/…/99"}
{"id": 4, "title": "MR !7",      "url": "https://gitlab.com/team/infra/merge_requests/7", "short": "…/infra/…/7"}
```

Original stays intact, shortened value goes into `short`.

### Modify in place, keep original (`--keep-original`)

```bash
$ cat items.jsonl | abre --json -k url -p url-path --keep-original full_url
```
```json
{"id": 1, "title": "Issue #42",  "url": "…/frontend/issues/42", "full_url": "https://github.com/org/frontend/issues/42"}
{"id": 2, "title": "Issue #15",  "url": "…/frontend/pulls/15",  "full_url": "https://github.com/org/frontend/pulls/15"}
{"id": 3, "title": "Pipeline",   "url": "…/backend/…/99",       "full_url": "https://github.com/org/backend/actions/runs/99"}
{"id": 4, "title": "MR !7",      "url": "…/infra/…/7",          "full_url": "https://gitlab.com/team/infra/merge_requests/7"}
```

Shortened goes into `url`, original backed up to `full_url`.

## Planned

- **`--width N`** — smart collapse to fit within N chars. Unlike dumb truncation (`cut -c`), uses segment structure to collapse more aggressively from the left until it fits the budget. e.g. `--width 30` on `https://github.com/org/frontend/issues/42` → `https://…/…/issues/42`
