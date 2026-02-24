#!/bin/bash
#
# MERKLITH Blockchain One-Line Installer
# Supports: Linux, macOS, Windows (WSL)
#
# Usage:
#   curl -fsSL https://get.merklith.com | bash
#   wget -qO- https://get.merklith.com | bash
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
MERKLITH_VERSION="${MERKLITH_VERSION:-latest}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.merklith}"
BIN_DIR="${INSTALL_DIR}/bin"
DATA_DIR="${INSTALL_DIR}/data"
CONFIG_DIR="${INSTALL_DIR}/config"

# Logging
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)     OS=linux;;
        Darwin*)    OS=macos;;
        CYGWIN*|MINGW*|MSYS*) OS=windows;;
        *)          OS=unknown;;
    esac
    
    case "$(uname -m)" in
        x86_64)     ARCH=x86_64;;
        arm64|aarch64) ARCH=arm64;;
        *)          ARCH=unknown;;
    esac
    
    log_info "Detected OS: $OS, Architecture: $ARCH"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check if running as root (warn but don't exit)
    if [ "$EUID" -eq 0 ]; then
        log_warn "Running as root is not recommended for security reasons"
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
    
    # Check for required tools
    local missing_tools=()
    
    if ! command -v curl &> /dev/null && ! command -v wget &> /dev/null; then
        missing_tools+=("curl or wget")
    fi
    
    if ! command -v tar &> /dev/null; then
        missing_tools+=("tar")
    fi
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        log_error "Missing required tools: ${missing_tools[*]}"
        exit 1
    fi
    
    # Check disk space (need at least 500MB)
    local available_space=$(df "$HOME" | awk 'NR==2 {print $4}')
    if [ "$available_space" -lt 512000 ]; then  # 500MB in KB
        log_error "Insufficient disk space. Need at least 500MB free"
        exit 1
    fi
    
    log_success "Prerequisites check passed"
}

# Download and install
download_install() {
    log_info "Downloading MERKLITH..."
    
    local download_url
    if [ "$MERKLITH_VERSION" = "latest" ]; then
        download_url="https://github.com/merklith/merklith/releases/latest/download/merklith-${OS}-${ARCH}.tar.gz"
    else
        download_url="https://github.com/merklith/merklith/releases/download/${MERKLITH_VERSION}/merklith-${OS}-${ARCH}.tar.gz"
    fi
    
    # Create directories
    mkdir -p "$BIN_DIR" "$DATA_DIR" "$CONFIG_DIR"
    
    # Download
    local temp_file=$(mktemp)
    log_info "Downloading from: $download_url"
    
    if command -v curl &> /dev/null; then
        curl -fsSL "$download_url" -o "$temp_file" --progress-bar
    else
        wget -q --show-progress "$download_url" -O "$temp_file"
    fi
    
    # Extract
    log_info "Extracting..."
    tar -xzf "$temp_file" -C "$BIN_DIR"
    rm "$temp_file"
    
    # Make executable
    chmod +x "$BIN_DIR"/*
    
    log_success "MERKLITH installed to $BIN_DIR"
}

# Setup environment
setup_environment() {
    log_info "Setting up environment..."
    
    # Detect shell
    local shell_rc
    case "$SHELL" in
        */bash) shell_rc="$HOME/.bashrc" ;;
        */zsh)  shell_rc="$HOME/.zshrc" ;;
        */fish) shell_rc="$HOME/.config/fish/config.fish" ;;
        *)      shell_rc="$HOME/.profile" ;;
    esac
    
    # Add to PATH if not already there
    if ! grep -q "$BIN_DIR" "$shell_rc" 2>/dev/null; then
        log_info "Adding $BIN_DIR to PATH in $shell_rc"
        echo "" >> "$shell_rc"
        echo "# MERKLITH Blockchain" >> "$shell_rc"
        echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$shell_rc"
        echo "export MERKLITH_HOME=\"$INSTALL_DIR\"" >> "$shell_rc"
    fi
    
    # Create default config
    if [ ! -f "$CONFIG_DIR/default.toml" ]; then
        cat > "$CONFIG_DIR/default.toml" << 'EOF'
# MERKLITH Default Configuration

[chain]
id = 1337
name = "merklith-dev"
block_time = 2

[rpc]
enabled = true
http_address = "0.0.0.0:8545"
ws_address = "0.0.0.0:8546"
cors = true

[p2p]
enabled = true
listen_address = "0.0.0.0:30303"
bootstrap_nodes = []

[consensus]
enabled = false  # Set to true for validator mode
validator_address = ""

[wallet]
keystore_dir = "~/.merklith/keystore"
default_account = ""
EOF
    fi
    
    log_success "Environment configured"
}

# Install shell completions
install_completions() {
    log_info "Installing shell completions..."
    
    # Bash
    if [ -d "$HOME/.bash_completion.d" ] || [ -d "/etc/bash_completion.d" ]; then
        "$BIN_DIR/merklith" completions bash > "$HOME/.merklith/completions.bash" 2>/dev/null || true
        if [ -f "$HOME/.merklith/completions.bash" ]; then
            echo "source $HOME/.merklith/completions.bash" >> "$HOME/.bashrc"
        fi
    fi
    
    # Zsh
    if [ -d "$HOME/.zsh/completions" ]; then
        "$BIN_DIR/merklith" completions zsh > "$HOME/.zsh/completions/_merklith" 2>/dev/null || true
    fi
    
    # Fish
    if [ -d "$HOME/.config/fish/completions" ]; then
        "$BIN_DIR/merklith" completions fish > "$HOME/.config/fish/completions/merklith.fish" 2>/dev/null || true
    fi
}

# Verify installation
verify_installation() {
    log_info "Verifying installation..."
    
    if [ ! -f "$BIN_DIR/merklith" ]; then
        log_error "Installation failed: merklith binary not found"
        exit 1
    fi
    
    if [ ! -f "$BIN_DIR/merklith-node" ]; then
        log_error "Installation failed: merklith-node binary not found"
        exit 1
    fi
    
    # Test binaries
    local version
    version=$("$BIN_DIR/merklith" --version 2>/dev/null || echo "unknown")
    log_success "MERKLITH version: $version"
    
    log_success "Installation verified"
}

# Print next steps
print_next_steps() {
    echo
    echo -e "${GREEN}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║        MERKLITH Blockchain Installation Complete!             ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════════════════╝${NC}"
    echo
    echo "Installation directory: $INSTALL_DIR"
    echo
    echo -e "${YELLOW}To get started:${NC}"
    echo
    echo "1. Reload your shell configuration:"
    echo -e "   ${BLUE}source ~/.bashrc${NC} (or restart your terminal)"
    echo
    echo "2. Create your first wallet:"
    echo -e "   ${BLUE}merklith wallet create${NC}"
    echo
    echo "3. Start a local node:"
    echo -e "   ${BLUE}merklith-node --dev${NC}"
    echo
    echo "4. Explore the blockchain:"
    echo -e "   ${BLUE}merklith explorer${NC}"
    echo
    echo -e "${YELLOW}Documentation:${NC}"
    echo "  - Website: https://merklith.com"
    echo "  - GitHub: https://github.com/merklith/merklith"
    echo
    echo -e "${GREEN}Where Trust is Forged${NC}"
    echo
}

# Uninstall function
uninstall() {
    log_warn "This will remove MERKLITH from your system"
    read -p "Are you sure? (y/N) " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf "$INSTALL_DIR"
        log_success "MERKLITH has been uninstalled"
        log_info "Don't forget to remove PATH entries from your shell config"
    else
        log_info "Uninstall cancelled"
    fi
    exit 0
}

# Main installation
main() {
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --version)
                MERKLITH_VERSION="$2"
                shift 2
                ;;
            --install-dir)
                INSTALL_DIR="$2"
                BIN_DIR="${INSTALL_DIR}/bin"
                DATA_DIR="${INSTALL_DIR}/data"
                CONFIG_DIR="${INSTALL_DIR}/config"
                shift 2
                ;;
            --uninstall)
                uninstall
                ;;
            --help)
                echo "MERKLITH Blockchain Installer"
                echo
                echo "Usage: $0 [OPTIONS]"
                echo
                echo "Options:"
                echo "  --version VERSION    Install specific version (default: latest)"
                echo "  --install-dir DIR    Installation directory (default: ~/.merklith)"
                echo "  --uninstall          Remove MERKLITH"
                echo "  --help              Show this help"
                echo
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Print banner
    echo -e "${BLUE}"
    echo "    _                _ _           _   _           _"
    echo "   / \   _ __  _ __ | (_) ___ __ _| |_(_)_ __   __| |"
    echo "  / _ \ | '_ \| '_ \| | |/ __/ _\` | __| | '_ \ / _\` |"
    echo " / ___ \| | | | | | | | | (_| (_| | |_| | | | | (_| |"
    echo "/_/   \_\_| |_|_| |_|_|_|\___\__,_|\__|_|_| |_|\__,_|"
    echo
    echo -e "${NC}"
    echo "Installing MERKLITH Blockchain..."
    echo
    
    # Run installation steps
    detect_os
    check_prerequisites
    download_install
    setup_environment
    install_completions
    verify_installation
    print_next_steps
}

# Run main function
main "$@"