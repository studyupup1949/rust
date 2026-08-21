#!/usr/bin/env bash
set -euo pipefail

BOLD="\033[1m"
GREEN="\033[0;32m"
YELLOW="\033[0;33m"
RED="\033[0;31m"
CYAN="\033[0;36m"
RESET="\033[0m"

echo -e "${CYAN}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  AUTONOMOUS AGENT SYSTEM - INSTALL"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${RESET}"

# Check prerequisites
echo -e "${YELLOW}Checking prerequisites...${RESET}"

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Rust/Cargo not found.${RESET}"
    echo "  Install via: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi
echo -e "  ${GREEN}✓${RESET} Rust $(cargo --version | cut -d' ' -f2)"

# Check for Git (optional but useful)
if command -v git &> /dev/null; then
    echo -e "  ${GREEN}✓${RESET} Git $(git --version | cut -d' ' -f3)"
else
    echo -e "  ${YELLOW}○${RESET} Git not found (optional for repo agent)"
fi

# Check for SQLite
if command -v sqlite3 &> /dev/null; then
    echo -e "  ${GREEN}✓${RESET} SQLite3"
else
    echo -e "  ${YELLOW}○${RESET} SQLite3 not found (will use bundled)"
fi

echo ""

# Build
echo -e "${YELLOW}Building AAS...${RESET}"
cargo build --release 2>&1 | tail -5
echo -e "  ${GREEN}✓${RESET} Build complete"

# Install binary
echo ""
echo -e "${YELLOW}Installing binary...${RESET}"
INSTALL_DIR="${HOME}/.cargo/bin"
mkdir -p "${INSTALL_DIR}"
cp target/release/aas "${INSTALL_DIR}/aas"
echo -e "  ${GREEN}✓${RESET} Installed to ${INSTALL_DIR}/aas"

# Create config directory
CONFIG_DIR="${HOME}/.aas"
mkdir -p "${CONFIG_DIR}"
echo -e "  ${GREEN}✓${RESET} Config directory: ${CONFIG_DIR}"

# Print version
echo ""
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo -e "${GREEN}${BOLD}  AAS v$(./target/release/aas version 2>/dev/null || echo "0.1.0") installed!${RESET}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo ""
echo "  Quick start:"
echo "    aas dashboard     → Open setup wizard in browser"
echo "    aas run           → Start the agent swarm"
echo "    aas status        → View swarm status"
echo "    aas help          → Show all commands"
echo ""
echo "  Configuration:"
echo "    ~/.aas/config.json"
echo "    ~/.aas/aas.db"
echo ""

# Ask if user wants to start dashboard
read -r -p "Open setup dashboard now? [Y/n] " RESP
if [[ -z "${RESP}" || "${RESP}" =~ ^[Yy] ]]; then
    echo "Starting dashboard at http://localhost:3000 ..."
    cargo run --release -- dashboard 2>/dev/null || "${INSTALL_DIR}/aas" dashboard
fi
