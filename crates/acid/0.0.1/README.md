# acidarchive.com api

acidarchive.com backend

## Pre-requisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Docker](https://docs.docker.com/get-docker/)

```bash
# archlinux 
sudo pacman -S lld clang postgresql
```

```bash
# macos
brew install michaeleisel/zld/zld
```
```bash
cp .env.example .env
```
```bash
./scripts/init_db.sh
```

```bash
./scripts/init_redis.sh
```

## Build



```bash
cargo build
```

## Run
```bash
cargo run
```
API: http://localhost:8000

Docs: http://localhost:8000/docs/api

## Test
```bash
cargo test
```

## Development

```bash
./scripts/setup_hooks.sh
```


```bash
rustup component add clippy rustfmt
```

### sqlx-cli
```bash
cargo install --version="~0.8.2" sqlx-cli --no-default-features --features rustls,postgres
```
```bash
cargo sqlx prepare -- --lib
```
```bash
cargo sqlx prepare -- --tests
```