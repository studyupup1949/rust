# abstract_semaphore

A portable abstraction over native operating system semaphores.

`abstract_semaphore` provides a minimal, portable interface over the native semaphore implementation of each supported operating system. Rather than implementing a userspace semaphore, the crate delegates all synchronization to the operating system.

The objective is simple:

> **Write portable code without learning a different semaphore API for every operating system.**

---

## Features

- Native operating system semaphores.
- Portable API.
- Minimal interface.
- No asynchronous runtime required.
- No custom synchronization algorithm.
- Zero allocations performed by the crate itself.
- Small dependency footprint.

---

## Design philosophy

This crate intentionally exposes only the fundamental semaphore operations.

It is **not** intended to become another synchronization framework.

Instead, it provides a thin abstraction over the semaphore primitive supplied by each operating system while preserving its behaviour whenever possible.

The crate does **not** attempt to:

- emulate semaphores;
- hide operating-system behaviour;
- replace existing concurrency frameworks;
- introduce scheduling policies.

The operating system remains responsible for synchronization.

---

## Supported platforms

| Platform | Status | Native implementation |
|----------|:------:|-----------------------|
| Linux (glibc) | ✅ Supported | POSIX semaphores (`sem_t`) |
| Windows | ✅ Supported | Win32 Semaphores |
| macOS | ❌ Unsupported | - |
| FreeBSD | ❌ Unsupported | - |
| OpenBSD | ❌ Unsupported | - |
| NetBSD | ❌ Unsupported | - |
| RTOS | ❌ Unsupported | - |

> **Note**
>
> Only Linux and Windows are currently supported.
> Additional platforms may be added in the future, but there is currently no public roadmap or commitment regarding support for specific operating systems.

---

## Installation

```toml
[dependencies]
abstract_semaphore = "0.1"
```

---

## Basic usage

```rust
use abstract_semaphore::Semaphore;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let semaphore = Semaphore::new(1)?;

    semaphore.wait()?;

    println!("Critical section");

    semaphore.post()?;

    semaphore.destroy()?;

    Ok(())
}
```

---

## API

Currently the public API intentionally consists of only four operations.

```rust
Semaphore::new(initial)
Semaphore::wait()
Semaphore::post()
Semaphore::destroy()
```

### `new(initial)`

Creates a semaphore with the given initial value.

---

### `wait()`

Acquires one resource from the semaphore.

If no resources are available, the current thread blocks until another thread releases one.

---

### `post()`

Releases one resource.

If another thread is blocked waiting on the semaphore, the operating system may wake one of them.

---

### `destroy()`

Destroys the native semaphore.

The method consumes the `Semaphore`, preventing further use after destruction.

---

# Correct usage

## Resource pool

```rust
let pool = Semaphore::new(4)?;

// Acquire one resource
pool.wait()?;

// Use shared resource...

// Release it
pool.post()?;
```

---

## Producer / Consumer

```text
Producer

post()

Consumer

wait()
```

---

## Ping-Pong synchronization

```text
Semaphore A = 1
Semaphore B = 0

Thread A

wait(A)
...
post(B)

Thread B

wait(B)
...
post(A)
```

---

# Incorrect usage

## Destroying a semaphore while another thread is using it

```rust
let semaphore = Semaphore::new(1)?;

std::thread::spawn(|| {

    semaphore.wait().unwrap();

});

semaphore.destroy()?;
```

**Incorrect.**

Destroying a semaphore while another thread is blocked or still using it may result in undefined behaviour depending on the operating system.

---

## Forgetting to release the semaphore

```rust
semaphore.wait()?;

// ...

// Missing semaphore.post()
```

This permanently consumes one resource from the semaphore.

---

## Using a destroyed semaphore

```rust
let semaphore = Semaphore::new(1)?;

semaphore.destroy()?;

// Impossible because destroy consumes self.
```

The API intentionally prevents this mistake.

---

# Platform specific notes

## Linux

Implementation based on POSIX semaphores.

Native functions used:

- `sem_init()`
- `sem_wait()`
- `sem_post()`
- `sem_destroy()`

The implementation relies on the operating system's semaphore implementation.

Signal handling follows POSIX semantics.

---

## Windows

Implementation based on Win32 semaphores.

Native functions used:

- `CreateSemaphoreW()`
- `WaitForSingleObject()`
- `ReleaseSemaphore()`
- `CloseHandle()`

The implementation relies on the Windows kernel synchronization primitives.

---

# Known limitations

Current limitations include:

- unnamed semaphores only;
- no timeout support;
- no named semaphores;
- no semaphore arrays;
- no multi-resource acquisition;
- no fairness guarantees.

These features may be added in future versions if they can be implemented while preserving the design philosophy of the crate.

---

# Thread safety

The crate is intended for multi-threaded synchronization.

The operating system guarantees synchronization of its native semaphore implementation.

The programmer is responsible for ensuring that a semaphore is not destroyed while still being used by another thread.

---

# Why not `std::sync`?

The Rust standard library intentionally does not expose native operating-system semaphores.

`abstract_semaphore` exists to provide a portable abstraction over those native primitives.

---

# Why not Tokio?

Tokio provides asynchronous synchronization primitives.

`abstract_semaphore` targets native operating-system semaphores and synchronous system programming.

---

# License

MIT License.