# abre

Shorten repetitive text for display. Pipe lines in, get disambiguated lines out.

```
$ cat paths | abre
/home/user/proj/foo/src/main.rs  →  /…/foo/src/main.rs
/home/user/proj/bar/src/main.rs  →  /…/bar/src/main.rs
/home/user/proj/bar/src/lib.rs   →  /…/bar/src/lib.rs
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
| `-s <chars>` | Separator characters — each char is a separator; `common` = `/_-` | `/` |
| `--suffix` | Shortest unique suffix strategy | |
| `--truncate [-n N]` | Shorten shared segments to N chars | collapse mode, N=1 |
| `--ellipsis <str>` | Replacement string for removed parts | `…` |
| `--json -k <key>` | JSON line mode, operate on key | |
| `--add-key <key>` | Write shortened to new key | |
| `--keep-original <key>` | Modify in place, save original to key | |

## Presets (`-p`)

Built-in `-c` shortcuts for common patterns:

| Preset | Shortens |
|--------|----------|
| `url-path` | URL path only, strips scheme and domain |
| `url-full` | Full URL including domain |
| `url-domain` | Domain only |
| `docker` | Image name, keeps tag |

```bash
cat urls | abre -p url-path
# https://github.com/org/repo/issues/1   →  /…/issues/1
# https://github.com/org/repo/pulls/2    →  /…/pulls/2
```

## Strategies

**collapse** (default) — trie-based: replaces non-branching middle segments with `…`, keeps segments needed to disambiguate:
```
/home/user/proj/foo/src/main.rs  →  /…/foo/src/main.rs
/home/user/proj/bar/src/main.rs  →  /…/bar/src/main.rs
/home/user/proj/bar/src/lib.rs   →  /…/bar/src/lib.rs
```

**suffix** — shortest unique trailing segment(s) per line:
```
/home/user/proj/foo/src/main.rs  →  /…/foo/src/main.rs
/home/user/proj/bar/src/main.rs  →  /…/bar/src/main.rs
/home/user/proj/bar/src/lib.rs   →  /…/lib.rs
```

**truncate** — shorten shared prefix segments to N chars (default 1):
```
/home/user/proj/foo/src/main.rs  →  /h/u/p/foo/src/main.rs
/home/user/proj/bar/src/main.rs  →  /h/u/p/bar/src/main.rs
```

## Multiple separators

Pass multiple characters to `-s` — each is treated as a separator. The original separator between each segment is preserved in the output.

```bash
# Split on both / and -
$ echo -e "build/my-app_v1\nbuild/my-app_v2" | abre -s /-
…-app_v1
…-app_v2

# "common" preset covers /_-
$ echo -e "release/my-app_v1/lib.so\nrelease/my-app_v2/lib.so" | abre -s common
…_v1/lib.so
…_v2/lib.so

# Java packages with dot separator
$ echo -e "com.example.foo.Bar\ncom.example.bar.Baz" | abre -s .
….foo.Bar
….bar.Baz
```

## Examples

```bash
# file paths
find . -name '*.rs' | abre

# URL paths (strips scheme + domain)
cat urls | abre -p url-path

# Docker images — shorten image names, keep tags
docker ps --format '{{.Image}}' | abre -p docker

# Java/Kotlin package names
cat packages | abre -s .

# Mixed separators (path + hyphen + underscore)
cat build-artifacts | abre -s common
```

## JSON mode

Operate on a key in JSON lines (one object per line).

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
{"id": 1, "title": "Issue #42",  "url": "/org/frontend/issues/42"}
{"id": 2, "title": "Issue #15",  "url": "/org/frontend/pulls/15"}
{"id": 3, "title": "Pipeline",   "url": "/org/backend/…/99"}
{"id": 4, "title": "MR !7",      "url": "/team/…/7"}
```

URLs 1 & 2 keep `frontend` because they share the repo — `issues` vs `pulls` is the branching point. URLs 3 & 4 diverge higher up so more gets collapsed.

### Add new key (`--add-key`)

```bash
$ cat items.jsonl | abre --json -k url -p url-path --add-key short
```
```json
{"id": 1, "title": "Issue #42",  "url": "https://github.com/org/frontend/issues/42", "short": "/org/frontend/issues/42"}
{"id": 2, "title": "Issue #15",  "url": "https://github.com/org/frontend/pulls/15",  "short": "/org/frontend/pulls/15"}
{"id": 3, "title": "Pipeline",   "url": "https://github.com/org/backend/actions/runs/99", "short": "/org/backend/…/99"}
{"id": 4, "title": "MR !7",      "url": "https://gitlab.com/team/infra/merge_requests/7", "short": "/team/…/7"}
```

Original stays intact, shortened value goes into `short`.

### Modify in place, keep original (`--keep-original`)

```bash
$ cat items.jsonl | abre --json -k url -p url-path --keep-original full_url
```
```json
{"id": 1, "title": "Issue #42",  "url": "/org/frontend/issues/42",  "full_url": "https://github.com/org/frontend/issues/42"}
{"id": 2, "title": "Issue #15",  "url": "/org/frontend/pulls/15",   "full_url": "https://github.com/org/frontend/pulls/15"}
{"id": 3, "title": "Pipeline",   "url": "/org/backend/…/99",        "full_url": "https://github.com/org/backend/actions/runs/99"}
{"id": 4, "title": "MR !7",      "url": "/team/…/7",                "full_url": "https://gitlab.com/team/infra/merge_requests/7"}
```

Shortened goes into `url`, original backed up to `full_url`.

## Planned

- **`--width N`** — smart collapse to fit within N chars. Unlike dumb truncation (`cut -c`), uses segment structure to collapse more aggressively from the left until it fits the budget. e.g. `--width 30` on `/org/frontend/issues/42` → `/org/…/issues/42`
