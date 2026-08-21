# access-cell

A cell that allows **re-entrant mutable access** without deadlock. Use it when you need something like a single-threaded “Mutex” where callbacks might need to access the same value again.

## Why?

With a normal `Mutex<T>`, if you lock it and then try to lock it again from the same thread (e.g. in a callback), you deadlock. `AccessCell<T>` avoids that: the first `access` runs immediately; any further `access` calls from inside that closure are **queued** and run in order after the current closure finishes.

So you can safely do “lock → do work → call something that also wants to lock” without re-entrancy causing a deadlock.

## Example

```rust
use access_cell::AccessCell;
use std::sync::Arc;

let cell = Arc::new(AccessCell::new(0));

cell.access({
    let cell = cell.clone();
    move |_| {
        // Nested access would deadlock with a Mutex, here it’s queued and runs after.
        cell.access(|v| {
            *v = 10;
        });
    }
});

assert_eq!(*cell.access_ref(), 10);
```

## API

| Method | Description |
|--------|-------------|
| `AccessCell::new(value)` | Create a new cell wrapping `value`. |
| `.access(\|v\| { ... })` | Run a closure with exclusive mutable access. Re-entrant calls are queued. |
| `.access_ref()` | Borrow the value immutably (safe to call anytime). |
| `.access_mut()` | **Unsafe** outside of an `access` closure. Use only inside a closure passed to `access`. |

## When to use it

- Single-threaded or “one logical owner” code where re-entrant access is possible (e.g. callbacks, event handlers).
- When you want “exclusive access” semantics without a real lock, and you’re okay with nested work being deferred to a queue.

Not a replacement for `Mutex` in multi-threaded code; it does not provide thread-safe locking.

## Installation
```bash
cargo add access-cell
```

Or manually

```toml
[dependencies]
access-cell = "0.1"
```
