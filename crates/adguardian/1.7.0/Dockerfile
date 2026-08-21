# syntax=docker/dockerfile:1

# Build: compile a static musl binary for the TARGET arch
FROM rust:1.90-alpine AS builder
RUN apk add --no-cache build-base cmake perl ca-certificates
WORKDIR /app
COPY . .
ARG TARGETPLATFORM
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo-reg-${TARGETPLATFORM} \
    cargo build --release && cp target/release/adguardian /adguardian

# Runtime: just the static binary on an empty image ---
FROM scratch
LABEL org.opencontainers.image.title="AdGuardian-Term" \
      org.opencontainers.image.description="Real-time traffic monitoring for AdGuard Home, in your terminal" \
      org.opencontainers.image.source="https://github.com/Lissy93/AdGuardian-Term" \
      org.opencontainers.image.licenses="MIT"
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
COPY --from=builder /adguardian /adguardian
USER 65534:65534
ENTRYPOINT ["/adguardian"]
