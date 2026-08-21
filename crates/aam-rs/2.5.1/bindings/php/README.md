# aam-php

PHP bindings for `aam-rs` using `FFI` on top of the stable C API.

## What you get

- Parse AAM from strings.
- Forward lookup (`key -> value`) and reverse lookup fallback (`value -> key`).
- Simple zero-dependency wrapper class for scripting and prototyping.

## Build native library

```bash
cargo build --release --features ffi
```

By default `AamDocument` reads the library from:

- `AAM_PHP_LIB` env var, if present.
- `target/release/libaam_rs.so` as fallback on Linux.

## Basic usage

```php
<?php

require_once 'php/src/AamPhp.php';

$aam = new RustGames\Aam\AamDocument("host = localhost\nport = 8080", '/absolute/path/to/libaam_rs.so');
echo $aam->get('host');
// localhost
```

## More examples

```php
<?php

require_once 'php/src/AamPhp.php';

$aam = new RustGames\Aam\AamDocument("env = production\nrole = api");

// direct lookup
var_dump($aam->get('env')); // "production"

// key/value lookup with fallback
var_dump($aam->find('production')); // ["env" => "production"]

// missing key
var_dump($aam->get('missing')); // null
```

## Tests

```bash
AAM_RS_LIB=target/release/libaam_rs.so php php/tests/smoke.php
```

The smoke script includes multiple assertions for valid parsing, reverse lookup, missing keys, and invalid input errors.

