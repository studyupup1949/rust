# aam-rs Python bindings

Python bindings for `aam-rs` with support for parsing, merging, reverse lookups, and deep alias resolution.

## Install

```bash
pip install aam-py
```

## Quick start

```python
from aam_py import AAML

doc = AAML.parse("host = localhost\nport = 8080")
print(doc.find_obj("host"))
```

## More examples

```python
from aam_py import AAML

doc = AAML.parse("""
root = /srv/app
active = root
env = production
""")

print(doc.find_deep("active"))      # /srv/app
print(doc.find_key("production"))   # env
print(doc.find_obj("production"))   # env (reverse lookup fallback)
```

```python
doc = AAML.parse("a = 1")
doc.merge_content("b = 2\na = 3")
print(doc.find_obj("a"))  # 3
print(doc.find_obj("b"))  # 2
```

## AAML syntax refresher

```aam
# comments
host = localhost
port = 8080

@import common.aam

base = /srv
current = base
```

## Local development checks

```bash
cargo test
```

If you work on packaging/bindings, also run the relevant CI matrix jobs from this repository.
