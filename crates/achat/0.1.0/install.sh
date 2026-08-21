#!/bin/sh
# install.sh — curl|sh installer for achat
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/quadra-a/achat/main/install.sh | sh
#   curl -fsSL https://raw.githubusercontent.com/quadra-a/achat/main/install.sh | sh -s -- --to ~/.local/bin
#   curl -fsSL https://raw.githubusercontent.com/quadra-a/achat/main/install.sh | sh -s -- --tag v0.2.0

set -eu

# ── Formatting helpers (degrade gracefully if tput missing) ──────────────────

BOLD="$(tput bold 2>/dev/null || printf '')"
RED="$(tput setaf 1 2>/dev/null || printf '')"
GREEN="$(tput setaf 2 2>/dev/null || printf '')"
YELLOW="$(tput setaf 3 2>/dev/null || printf '')"
BLUE="$(tput setaf 4 2>/dev/null || printf '')"
RESET="$(tput sgr0 2>/dev/null || printf '')"

info()  { printf '%s\n' "${BOLD}${BLUE}info${RESET}: $*"; }
warn()  { printf '%s\n' "${BOLD}${YELLOW}warn${RESET}: $*" >&2; }
error() { printf '%s\n' "${BOLD}${RED}error${RESET}: $*" >&2; }

# ── Utility ──────────────────────────────────────────────────────────────────

has() { command -v "$1" >/dev/null 2>&1; }

need() {
    if ! has "$1"; then
        error "required command not found: $1"
        exit 1
    fi
}

# ── Clean-up on exit ────────────────────────────────────────────────────────

cleanup() {
    if [ -n "${TMPDIR_CREATED:-}" ] && [ -d "$TMPDIR_CREATED" ]; then
        rm -rf "$TMPDIR_CREATED"
    fi
}
trap cleanup EXIT INT TERM

# ── Download helper (curl preferred, wget fallback) ─────────────────────────

download() {
    url="$1"
    output="$2"  # use "-" for stdout

    if has curl; then
        curl --proto '=https' --tlsv1.2 -sSfL "$url" -o "$output"
    elif has wget; then
        wget --https-only --secure-protocol=TLSv1_2 -qO "$output" "$url"
    else
        error "either curl or wget is required"
        exit 1
    fi
}

# ── Platform detection ──────────────────────────────────────────────────────

detect_target() {
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)  os_part="unknown-linux-gnu" ;;
        Darwin) os_part="apple-darwin" ;;
        *)
            error "unsupported operating system: $os"
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64|amd64)   arch_part="x86_64" ;;
        aarch64|arm64)   arch_part="aarch64" ;;
        *)
            error "unsupported architecture: $arch"
            exit 1
            ;;
    esac

    TARGET="${arch_part}-${os_part}"
}

# ── Resolve latest tag via GitHub API redirect ──────────────────────────────

resolve_latest_tag() {
    # We ask for the latest-release redirect URL which contains the tag.
    # This avoids needing jq to parse JSON.
    if has curl; then
        url="$(curl -sSfI "https://github.com/quadra-a/achat/releases/latest" 2>/dev/null \
               | grep -i '^location:' | head -1 | tr -d '\r')"
    elif has wget; then
        url="$(wget --server-response --spider "https://github.com/quadra-a/achat/releases/latest" 2>&1 \
               | grep -i '^\s*location:' | tail -1 | tr -d '\r')"
    fi

    # Fallback: try the JSON API
    if [ -z "${url:-}" ]; then
        tag="$(download "https://api.github.com/repos/quadra-a/achat/releases/latest" - \
               | grep '"tag_name"' | head -1 | cut -d'"' -f4)"
        if [ -n "$tag" ]; then
            printf '%s' "$tag"
            return
        fi
        error "could not determine latest release tag"
        exit 1
    fi

    # Extract tag from redirect URL (last path segment)
    tag="${url##*/}"
    printf '%s' "$tag"
}

# ── Permission elevation ────────────────────────────────────────────────────

ensure_writable() {
    dir="$1"
    if [ -w "$dir" ]; then
        SUDO=""
    elif has sudo; then
        info "elevated permissions required to install to $dir"
        SUDO="sudo"
    else
        error "$dir is not writable and sudo is not available"
        exit 1
    fi
}

# ── Argument parsing ────────────────────────────────────────────────────────

INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
TAG=""
FORCE="false"

usage() {
    cat <<EOF
install.sh — install achat prebuilt binary

USAGE
    install.sh [options]

OPTIONS
    -h, --help          Show this help
    --to DIR            Install directory [default: /usr/local/bin]
    --tag TAG           Specific release tag (e.g. v0.2.0) [default: latest]
    -f, --force         Overwrite existing binary without prompting
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --to)
            INSTALL_DIR="$2"
            shift 2
            ;;
        --tag)
            TAG="$2"
            shift 2
            ;;
        -f|--force)
            FORCE="true"
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            error "unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# ── Main ────────────────────────────────────────────────────────────────────

need tar
need uname

detect_target

if [ -z "$TAG" ]; then
    info "resolving latest release..."
    TAG="$(resolve_latest_tag)"
fi

ARCHIVE="achat-${TARGET}.tar.gz"
URL="https://github.com/quadra-a/achat/releases/download/${TAG}/${ARCHIVE}"

info "repository:  https://github.com/quadra-a/achat"
info "tag:         ${TAG}"
info "target:      ${TARGET}"
info "archive:     ${ARCHIVE}"
info "install dir: ${INSTALL_DIR}"

# Check for existing install
if [ -e "${INSTALL_DIR}/achat" ] && [ "$FORCE" = "false" ]; then
    warn "achat already exists at ${INSTALL_DIR}/achat"
    warn "pass --force to overwrite, or remove it first"
    exit 1
fi

# Create a temp directory
TMPDIR_CREATED="$(mktemp -d 2>/dev/null || mktemp -d -t achat-install)"

info "downloading ${BLUE}${URL}${RESET}"
download "$URL" "${TMPDIR_CREATED}/${ARCHIVE}"

# Verify the download produced a non-empty file
if [ ! -s "${TMPDIR_CREATED}/${ARCHIVE}" ]; then
    error "download failed — archive is empty"
    exit 1
fi

# Extract
info "extracting archive..."
tar -xzf "${TMPDIR_CREATED}/${ARCHIVE}" -C "${TMPDIR_CREATED}"

if [ ! -f "${TMPDIR_CREATED}/achat" ]; then
    error "expected binary 'achat' not found in archive"
    exit 1
fi

# Install
mkdir -p "$INSTALL_DIR"
ensure_writable "$INSTALL_DIR"

${SUDO:-} cp "${TMPDIR_CREATED}/achat" "${INSTALL_DIR}/achat"
${SUDO:-} chmod 755 "${INSTALL_DIR}/achat"

printf '\n'
info "${GREEN}achat ${TAG} installed successfully${RESET} to ${INSTALL_DIR}/achat"

# PATH check
case ":${PATH}:" in
    *":${INSTALL_DIR}:"*) ;;
    *)
        warn "${INSTALL_DIR} is not in your \$PATH"
        warn "add it with:  export PATH=\"${INSTALL_DIR}:\$PATH\""
        ;;
esac

printf '\n'
info "run ${BOLD}achat --help${RESET} to get started"
