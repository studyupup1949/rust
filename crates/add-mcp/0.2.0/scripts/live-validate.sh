#!/usr/bin/env bash
#
# live-validate.sh — Validate add-mcp against real AI client configurations.
#
# Installs a dummy MCP server into each client's config, verifies the file,
# then cleans up. Non-destructive: backs up existing configs and restores them.
#
# Usage: bash scripts/live-validate.sh
#
set -euo pipefail

# ─── Config ──────────────────────────────────────────────────────────────────

TEST_SERVER_NAME="__add_mcp_live_test__"
BINARY="/usr/bin/true"  # dummy command, just needs to exist
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
ADD_MCP="$PROJECT_DIR/target/release/add-mcp"

# Track results for summary
declare -a RESULTS=()
PASS=0
FAIL=0
SKIP=0

# Track backups for cleanup on interrupt
declare -a BACKUP_PAIRS=()

# ─── Colors ──────────────────────────────────────────────────────────────────

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

# ─── Helpers ─────────────────────────────────────────────────────────────────

log()      { echo -e "${BLUE}[info]${NC} $*"; }
log_pass() { echo -e "${GREEN}[PASS]${NC} $*"; }
log_fail() { echo -e "${RED}[FAIL]${NC} $*"; }
log_skip() { echo -e "${YELLOW}[SKIP]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[warn]${NC} $*"; }

record_pass() {
    RESULTS+=("${GREEN}PASS${NC}  $1")
    ((PASS++))
}

record_fail() {
    RESULTS+=("${RED}FAIL${NC}  $1 — $2")
    ((FAIL++))
}

record_skip() {
    RESULTS+=("${YELLOW}SKIP${NC}  $1 — $2")
    ((SKIP++))
}

check_installed() {
    command -v "$1" &>/dev/null
}

# Prompt user to install an npm package. Returns 0 if installed, 1 if skipped.
offer_install() {
    local cmd="$1"
    local package="$2"

    if check_installed "$cmd"; then
        return 0
    fi

    echo ""
    echo -en "  ${BOLD}$cmd${NC} not found. Install ${BOLD}$package${NC}? [y/N] "
    read -r answer
    if [[ "$answer" =~ ^[Yy]$ ]]; then
        log "Installing $package..."
        npm install -g "$package" || {
            log_warn "npm install failed for $package"
            return 1
        }
        # Verify it's now available
        if check_installed "$cmd"; then
            log "Installed $cmd successfully."
            return 0
        else
            log_warn "$cmd still not found after install."
            return 1
        fi
    fi
    return 1
}

# Back up a config file. Records the pair for cleanup.
backup_config() {
    local path="$1"
    local backup="${path}.add-mcp-backup"

    if [[ -f "$path" ]]; then
        cp "$path" "$backup"
        BACKUP_PAIRS+=("$path|$backup|existed")
    else
        BACKUP_PAIRS+=("$path||absent")
    fi
}

# Restore a config file from backup.
restore_config() {
    local path="$1"
    local backup="${path}.add-mcp-backup"

    if [[ -f "$backup" ]]; then
        mv "$backup" "$path"
    elif [[ -f "$path" ]]; then
        # File didn't exist before — remove it
        rm "$path"
        # Also remove parent dirs we may have created (only if empty)
        local dir
        dir="$(dirname "$path")"
        rmdir "$dir" 2>/dev/null || true
    fi
}

# Restore all backed-up configs (called on exit/interrupt)
cleanup_all() {
    if [[ ${#BACKUP_PAIRS[@]} -gt 0 ]]; then
        echo ""
        log "Cleaning up backups..."
        for entry in "${BACKUP_PAIRS[@]}"; do
            local path backup state
            IFS='|' read -r path backup state <<< "$entry"
            if [[ "$state" == "existed" && -f "${path}.add-mcp-backup" ]]; then
                mv "${path}.add-mcp-backup" "$path"
                log "  Restored: $path"
            elif [[ "$state" == "absent" && -f "$path" ]]; then
                rm "$path"
                local dir
                dir="$(dirname "$path")"
                rmdir "$dir" 2>/dev/null || true
                log "  Removed (was absent before): $path"
            fi
        done
    fi
}

trap cleanup_all EXIT INT TERM

# Verify a JSON config file contains the test server under a given section key.
# Usage: verify_json_config <path> <section_key>
verify_json_config() {
    local path="$1"
    local section_key="$2"

    if [[ ! -f "$path" ]]; then
        echo "config file not created at $path"
        return 1
    fi

    if ! jq -e ".[\"$section_key\"][\"$TEST_SERVER_NAME\"]" "$path" &>/dev/null; then
        echo "server not found under .$section_key.$TEST_SERVER_NAME"
        return 1
    fi

    return 0
}

# Verify a YAML config file (Goose) contains the test server.
# Usage: verify_yaml_config <path> <section_key>
verify_yaml_config() {
    local path="$1"
    local section_key="$2"

    if [[ ! -f "$path" ]]; then
        echo "config file not created at $path"
        return 1
    fi

    if ! python3 -c "
import yaml, json, sys
with open('$path') as f:
    data = yaml.safe_load(f)
section = data.get('$section_key', {})
if '$TEST_SERVER_NAME' not in section:
    sys.exit(1)
" 2>/dev/null; then
        echo "server not found under .$section_key.$TEST_SERVER_NAME"
        return 1
    fi

    return 0
}

# Verify a TOML config file (Codex) contains the test server.
# Usage: verify_toml_config <path> <section_key>
verify_toml_config() {
    local path="$1"
    local section_key="$2"

    if [[ ! -f "$path" ]]; then
        echo "config file not created at $path"
        return 1
    fi

    if ! grep -q "\[$section_key\\.\"$TEST_SERVER_NAME\"\]" "$path" &&
       ! grep -q "\[$section_key\\.$TEST_SERVER_NAME\]" "$path"; then
        # Also try as inline table
        if ! grep -q "$TEST_SERVER_NAME" "$path"; then
            echo "server not found in TOML under $section_key"
            return 1
        fi
    fi

    return 0
}

# ─── Per-agent test functions ────────────────────────────────────────────────

test_claude_code() {
    local agent="claude-code"
    local config_path="$HOME/.claude.json"
    local section_key="mcpServers"

    log "Testing Claude Code..."
    backup_config "$config_path"

    if ! "$ADD_MCP" install "$BINARY" -a "$agent" -g -n "$TEST_SERVER_NAME" -y 2>&1; then
        record_fail "Claude Code" "install command failed"
        restore_config "$config_path"
        return
    fi

    local err
    if err=$(verify_json_config "$config_path" "$section_key"); then
        # Also try CLI verification if claude is installed
        if check_installed "claude"; then
            if claude mcp list 2>/dev/null | grep -q "$TEST_SERVER_NAME"; then
                log_pass "Claude Code (file + CLI verified)"
                record_pass "Claude Code (file + CLI)"
            else
                log_pass "Claude Code (file verified, CLI did not list server)"
                record_pass "Claude Code (file only — CLI mismatch)"
            fi
        else
            log_pass "Claude Code (file verified)"
            record_pass "Claude Code"
        fi
    else
        log_fail "Claude Code: $err"
        record_fail "Claude Code" "$err"
    fi

    restore_config "$config_path"
}

test_claude_desktop() {
    local agent="claude-desktop"
    local config_path="$HOME/.config/Claude/claude_desktop_config.json"
    local section_key="mcpServers"

    # macOS path differs
    if [[ "$(uname)" == "Darwin" ]]; then
        config_path="$HOME/Library/Application Support/Claude/claude_desktop_config.json"
    fi

    log "Testing Claude Desktop (file read-back only)..."
    backup_config "$config_path"

    if ! "$ADD_MCP" install "$BINARY" -a "$agent" -g -n "$TEST_SERVER_NAME" -y 2>&1; then
        record_fail "Claude Desktop" "install command failed"
        restore_config "$config_path"
        return
    fi

    local err
    if err=$(verify_json_config "$config_path" "$section_key"); then
        log_pass "Claude Desktop"
        record_pass "Claude Desktop"
    else
        log_fail "Claude Desktop: $err"
        record_fail "Claude Desktop" "$err"
    fi

    restore_config "$config_path"
}

test_codex() {
    local agent="codex"
    local config_path="$HOME/.codex/config.toml"
    local section_key="mcp_servers"

    log "Testing Codex..."
    backup_config "$config_path"

    if ! "$ADD_MCP" install "$BINARY" -a "$agent" -g -n "$TEST_SERVER_NAME" -y 2>&1; then
        record_fail "Codex" "install command failed"
        restore_config "$config_path"
        return
    fi

    local err
    if err=$(verify_toml_config "$config_path" "$section_key"); then
        if check_installed "codex"; then
            log_pass "Codex (file verified, CLI available)"
            record_pass "Codex"
        else
            log_pass "Codex (file verified)"
            record_pass "Codex"
        fi
    else
        log_fail "Codex: $err"
        record_fail "Codex" "$err"
    fi

    restore_config "$config_path"
}

test_cursor() {
    local agent="cursor"
    local config_path="$HOME/.cursor/mcp.json"
    local section_key="mcpServers"

    log "Testing Cursor (file read-back only)..."

    # Only test if cursor dir exists or cursor is installed
    if [[ ! -d "$HOME/.cursor" ]] && ! check_installed "cursor"; then
        log_skip "Cursor (not installed)"
        record_skip "Cursor" "not installed"
        return
    fi

    backup_config "$config_path"

    if ! "$ADD_MCP" install "$BINARY" -a "$agent" -g -n "$TEST_SERVER_NAME" -y 2>&1; then
        record_fail "Cursor" "install command failed"
        restore_config "$config_path"
        return
    fi

    local err
    if err=$(verify_json_config "$config_path" "$section_key"); then
        log_pass "Cursor"
        record_pass "Cursor"
    else
        log_fail "Cursor: $err"
        record_fail "Cursor" "$err"
    fi

    restore_config "$config_path"
}

test_gemini_cli() {
    local agent="gemini-cli"
    local config_path="$HOME/.gemini/settings.json"
    local section_key="mcpServers"

    log "Testing Gemini CLI..."
    backup_config "$config_path"

    if ! "$ADD_MCP" install "$BINARY" -a "$agent" -g -n "$TEST_SERVER_NAME" -y 2>&1; then
        record_fail "Gemini CLI" "install command failed"
        restore_config "$config_path"
        return
    fi

    local err
    if err=$(verify_json_config "$config_path" "$section_key"); then
        log_pass "Gemini CLI"
        record_pass "Gemini CLI"
    else
        log_fail "Gemini CLI: $err"
        record_fail "Gemini CLI" "$err"
    fi

    restore_config "$config_path"
}

test_goose() {
    local agent="goose"
    local config_path="$HOME/.config/goose/config.yaml"
    local section_key="extensions"

    log "Testing Goose..."

    # Check for python3 + yaml module
    if ! python3 -c "import yaml" 2>/dev/null; then
        log_skip "Goose (python3 pyyaml not available for verification)"
        record_skip "Goose" "python3 pyyaml not available"
        return
    fi

    backup_config "$config_path"

    if ! "$ADD_MCP" install "$BINARY" -a "$agent" -g -n "$TEST_SERVER_NAME" -y 2>&1; then
        record_fail "Goose" "install command failed"
        restore_config "$config_path"
        return
    fi

    local err
    if err=$(verify_yaml_config "$config_path" "$section_key"); then
        log_pass "Goose"
        record_pass "Goose"
    else
        log_fail "Goose: $err"
        record_fail "Goose" "$err"
    fi

    restore_config "$config_path"
}

test_github_copilot() {
    local agent="github-copilot"
    local config_path="$HOME/.copilot/mcp-config.json"
    local section_key="mcpServers"

    log "Testing GitHub Copilot..."
    backup_config "$config_path"

    if ! "$ADD_MCP" install "$BINARY" -a "$agent" -g -n "$TEST_SERVER_NAME" -y 2>&1; then
        record_fail "GitHub Copilot" "install command failed"
        restore_config "$config_path"
        return
    fi

    local err
    if err=$(verify_json_config "$config_path" "$section_key"); then
        log_pass "GitHub Copilot"
        record_pass "GitHub Copilot"
    else
        log_fail "GitHub Copilot: $err"
        record_fail "GitHub Copilot" "$err"
    fi

    restore_config "$config_path"
}

test_opencode() {
    local agent="opencode"
    local config_path="$HOME/.config/opencode/opencode.json"
    local section_key="mcp"

    log "Testing OpenCode..."
    backup_config "$config_path"

    if ! "$ADD_MCP" install "$BINARY" -a "$agent" -g -n "$TEST_SERVER_NAME" -y 2>&1; then
        record_fail "OpenCode" "install command failed"
        restore_config "$config_path"
        return
    fi

    local err
    if err=$(verify_json_config "$config_path" "$section_key"); then
        log_pass "OpenCode"
        record_pass "OpenCode"
    else
        log_fail "OpenCode: $err"
        record_fail "OpenCode" "$err"
    fi

    restore_config "$config_path"
}

test_vscode() {
    local agent="vscode"
    local config_path="$HOME/.config/Code/User/mcp.json"
    local section_key="servers"

    # macOS path differs
    if [[ "$(uname)" == "Darwin" ]]; then
        config_path="$HOME/Library/Application Support/Code/User/mcp.json"
    fi

    log "Testing VS Code (file read-back only)..."

    # Only test if code is installed or config dir exists
    if ! check_installed "code" && [[ ! -d "$(dirname "$config_path")" ]]; then
        log_skip "VS Code (not installed)"
        record_skip "VS Code" "not installed"
        return
    fi

    backup_config "$config_path"

    if ! "$ADD_MCP" install "$BINARY" -a "$agent" -g -n "$TEST_SERVER_NAME" -y 2>&1; then
        record_fail "VS Code" "install command failed"
        restore_config "$config_path"
        return
    fi

    local err
    if err=$(verify_json_config "$config_path" "$section_key"); then
        log_pass "VS Code"
        record_pass "VS Code"
    else
        log_fail "VS Code: $err"
        record_fail "VS Code" "$err"
    fi

    restore_config "$config_path"
}

test_zed() {
    local agent="zed"
    local config_path="$HOME/.config/zed/settings.json"
    local section_key="context_servers"

    # macOS path differs
    if [[ "$(uname)" == "Darwin" ]]; then
        config_path="$HOME/Library/Application Support/Zed/settings.json"
    fi

    log "Testing Zed (file read-back only)..."

    # Only test if zed is installed or config dir exists
    if ! check_installed "zed" && ! check_installed "zeditor" && [[ ! -d "$(dirname "$config_path")" ]]; then
        log_skip "Zed (not installed)"
        record_skip "Zed" "not installed"
        return
    fi

    backup_config "$config_path"

    if ! "$ADD_MCP" install "$BINARY" -a "$agent" -g -n "$TEST_SERVER_NAME" -y 2>&1; then
        record_fail "Zed" "install command failed"
        restore_config "$config_path"
        return
    fi

    local err
    if err=$(verify_json_config "$config_path" "$section_key"); then
        log_pass "Zed"
        record_pass "Zed"
    else
        log_fail "Zed: $err"
        record_fail "Zed" "$err"
    fi

    restore_config "$config_path"
}

# ─── Main ────────────────────────────────────────────────────────────────────

echo ""
echo -e "${BOLD}add-mcp Live Validation${NC}"
echo -e "Tests add-mcp install against real AI client config files."
echo -e "Backs up existing configs and restores them after each test."
echo ""

# Prerequisites
if ! check_installed "jq"; then
    echo -en "${YELLOW}jq${NC} is required for JSON verification. Install? [y/N] "
    read -r answer
    if [[ "$answer" =~ ^[Yy]$ ]]; then
        if check_installed "apt-get"; then
            sudo apt-get install -y jq
        elif check_installed "brew"; then
            brew install jq
        else
            echo "Please install jq manually."
            exit 1
        fi
    else
        echo "jq is required. Exiting."
        exit 1
    fi
fi

if [[ ! -f "$BINARY" ]]; then
    log_warn "Test binary $BINARY not found. Using 'echo' as fallback."
    BINARY="$(command -v echo)"
fi

# Build
log "Building add-mcp (release)..."
(cd "$PROJECT_DIR" && cargo build --release) || {
    echo -e "${RED}Build failed. Aborting.${NC}"
    exit 1
}

if [[ ! -x "$ADD_MCP" ]]; then
    echo -e "${RED}Binary not found at $ADD_MCP. Aborting.${NC}"
    exit 1
fi

echo ""
echo -e "${BOLD}Running tests...${NC}"
echo ""

# Optional: offer to install CLI tools for deeper verification
echo -e "${BOLD}CLI tools enable deeper verification (mcp list commands).${NC}"
echo -e "You can skip any installs — file read-back is always tested."
echo ""

offer_install "claude" "@anthropic-ai/claude-code" || true
offer_install "codex" "@openai/codex" || true
offer_install "opencode" "opencode-ai" || true
offer_install "gemini" "@google/gemini-cli" || true

echo ""
echo -e "${BOLD}─── Testing all 10 agents ───${NC}"
echo ""

test_claude_code
test_claude_desktop
test_codex
test_cursor
test_gemini_cli
test_goose
test_github_copilot
test_opencode
test_vscode
test_zed

# ─── Summary ─────────────────────────────────────────────────────────────────

echo ""
echo -e "${BOLD}═══════════════════════════════════════${NC}"
echo -e "${BOLD}  Summary${NC}"
echo -e "${BOLD}═══════════════════════════════════════${NC}"
echo ""
for result in "${RESULTS[@]}"; do
    echo -e "  $result"
done
echo ""
echo -e "  ${GREEN}Passed: $PASS${NC}  ${RED}Failed: $FAIL${NC}  ${YELLOW}Skipped: $SKIP${NC}"
echo ""

if [[ $FAIL -gt 0 ]]; then
    echo -e "${RED}Some tests failed. Review the output above for details.${NC}"
    exit 1
else
    echo -e "${GREEN}All tested agents passed.${NC}"
fi
