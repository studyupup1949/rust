# Build stage
FROM rust:1.75-slim as builder

WORKDIR /build

RUN apt-get update && apt-get install -y \
    pkg-config \
    libsqlite3-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libsqlite3-0 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /root

COPY --from=builder /build/target/release/aas /usr/local/bin/aas

EXPOSE 3000

HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

ENTRYPOINT ["aas"]
CMD ["run"]
