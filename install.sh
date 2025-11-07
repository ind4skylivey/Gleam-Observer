#!/bin/bash
# install.sh - GleamObserver Enhanced Installer with Systemd + Systray
# Version: 1.5
# Includes: Binary, Systemd Service, Desktop Entry, Systray Support, Auto-start

set -e

VERSION="1.5"
REPO_URL="https://github.com/ind4skylivey/Gleam-Observer"
INSTALL_PREFIX="${HOME}/.local"
CONFIG_DIR="${HOME}/.config/gleam_observer"
AUTOSTART_DIR="${HOME}/.config/autostart"
SYSTEMD_USER_DIR="${HOME}/.config/systemd/user"
ICONS_DIR="${HOME}/.local/share/icons/hicolor/256x256/apps"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# ============================================================================
# FUNCIONES DE UTILIDAD
# ============================================================================

print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[⚠]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

print_step() {
    echo ""
    echo -e "${MAGENTA}════════════════════════════════════════════════════${NC}"
    echo -e "${MAGENTA}$1${NC}"
    echo -e "${MAGENTA}════════════════════════════════════════════════════${NC}"
}

# ============================================================================
# 1. DETECTAR ENTORNO
# ============================================================================

detect_display() {
    print_step "STEP 1: Detecting Graphical Environment"
    
    if [ -z "$DISPLAY" ] && [ -z "$WAYLAND_DISPLAY" ]; then
        print_warning "No graphical session detected (DISPLAY/WAYLAND_DISPLAY not set)"
        print_info "Continuing in headless mode (daemon will still work)"
        return 1
    else
        if [ -n "$DISPLAY" ]; then
            print_success "X11 detected (DISPLAY=$DISPLAY)"
        else
            print_success "Wayland detected (WAYLAND_DISPLAY=$WAYLAND_DISPLAY)"
        fi
        return 0
    fi
}

detect_distro() {
    print_info "Detecting Linux distribution..."
    
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        print_success "Detected: $NAME ($VERSION_ID)"
        echo "$ID"
    else
        print_warning "Could not detect distribution"
        echo "unknown"
    fi
}

# ============================================================================
# 2. VERIFICAR DEPENDENCIAS
# ============================================================================

check_dependencies() {
    print_step "STEP 2: Checking Dependencies"
    
    local missing=()
    local optional_missing=()
    
    print_info "Checking build dependencies..."
    
    # Requeridas
    if ! command -v cargo &> /dev/null; then
        missing+=("rust (cargo)")
    else
        print_success "✓ Rust ($(cargo --version | cut -d' ' -f2))"
    fi
    
    if ! command -v git &> /dev/null; then
        missing+=("git")
    else
        print_success "✓ Git"
    fi
    
    # Opcionales pero importantes
    if ! command -v notify-send &> /dev/null; then
        optional_missing+=("libnotify (notify-send)")
    else
        print_success "✓ libnotify (notify-send)"
    fi
    
    if ! command -v systemctl &> /dev/null; then
        missing+=("systemd")
    else
        print_success "✓ systemd"
    fi
    
    if ! command -v xdg-open &> /dev/null; then
        optional_missing+=("xdg-utils")
    else
        print_success "✓ xdg-utils"
    fi
    
    # Mostrar resultados
    if [ ${#missing[@]} -ne 0 ]; then
        print_error "Missing required packages: ${missing[*]}"
        print_info "Installing missing packages..."
        install_packages missing[@]
    fi
    
    if [ ${#optional_missing[@]} -ne 0 ]; then
        print_warning "Missing optional packages: ${optional_missing[*]}"
        read -p "Install optional packages? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            install_packages optional_missing[@]
        fi
    fi
    
    print_success "Dependency check complete"
}

install_packages() {
    local packages=("${!1}")
    local distro=$(detect_distro)
    
    case "$distro" in
        arch|manjaro)
            print_info "Installing via pacman..."
            sudo pacman -S --noconfirm ${packages[@]} 2>/dev/null || print_warning "Some packages may already be installed"
            ;;
        debian|ubuntu|linuxmint)
            print_info "Installing via apt..."
            sudo apt-get update
            sudo apt-get install -y ${packages[@]} 2>/dev/null || print_warning "Some packages may already be installed"
            ;;
        fedora|rhel|centos)
            print_info "Installing via dnf..."
            sudo dnf install -y ${packages[@]} 2>/dev/null || print_warning "Some packages may already be installed"
            ;;
        opensuse*)
            print_info "Installing via zypper..."
            sudo zypper install -y ${packages[@]} 2>/dev/null || print_warning "Some packages may already be installed"
            ;;
        *)
            print_warning "Unknown distro ($distro), skipping auto-install"
            print_info "Please install packages manually: ${packages[*]}"
            ;;
    esac
}

# ============================================================================
# 3. COMPILAR BINARIO
# ============================================================================

build_binary() {
    print_step "STEP 3: Building Release Binary"
    
    if [ ! -f "Cargo.toml" ]; then
        print_error "Cargo.toml not found. Are you in the repository root?"
        exit 1
    fi
    
    print_info "Building with features: nvidia, amd, intel, systray"
    cargo build --release --features nvidia,amd,intel,systray 2>&1 | grep -E "(Compiling|Finished|error|warning)" | tail -20
    
    if [ -f "target/release/gleam" ]; then
        print_success "Binary built successfully: $(ls -lh target/release/gleam | awk '{print $5}')"
    else
        print_error "Build failed - binary not found"
        exit 1
    fi
}

# ============================================================================
# 4. INSTALAR BINARIO
# ============================================================================

install_binary() {
    print_step "STEP 4: Installing Binary"
    
    mkdir -p "$INSTALL_PREFIX/bin"
    cp target/release/gleam "$INSTALL_PREFIX/bin/"
    chmod +x "$INSTALL_PREFIX/bin/gleam"
    
    # Verificar que está en PATH
    if ! echo "$PATH" | grep -q "$INSTALL_PREFIX/bin"; then
        print_warning "WARNING: $INSTALL_PREFIX/bin is not in PATH"
        print_info "Add this to your shell profile:"
        echo "  export PATH=\"$INSTALL_PREFIX/bin:\$PATH\""
    fi
    
    print_success "Binary installed: $INSTALL_PREFIX/bin/gleam"
    print_info "Verify installation: $INSTALL_PREFIX/bin/gleam --version"
}

# ============================================================================
# 5. INSTALAR ICONO
# ============================================================================

install_icon() {
    print_step "STEP 5: Installing Application Icon"
    
    mkdir -p "$ICONS_DIR"
    
    if [ -f "assets/gleamobserver.png" ]; then
        cp assets/gleamobserver.png "$ICONS_DIR/"
        print_success "Icon installed: $ICONS_DIR/gleamobserver.png"
    else
        print_warning "Icon not found (assets/gleamobserver.png), using fallback"
        # Crear ícono dummy minimalista (placeholder)
        cat > "$ICONS_DIR/gleamobserver.png" << 'EOF'
# En producción, reemplazar con PNG real
EOF
    fi
}

# ============================================================================
# 6. CREAR DESKTOP ENTRY (Autostart)
# ============================================================================

create_desktop_entry() {
    print_step "STEP 6: Creating Desktop Entry (Autostart)"
    
    mkdir -p "$AUTOSTART_DIR"
    
    cat > "$AUTOSTART_DIR/gleam-observer.desktop" << EOF
[Desktop Entry]
Type=Application
Exec=$INSTALL_PREFIX/bin/gleam --tray
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true
X-GNOME-Autostart-Delay=5
X-KDE-autostart-after=panel

Name=GleamObserver
Comment=🌌 Cyberpunk Hardware Monitor - Always-on Daemon
Icon=gleamobserver
Categories=System;Monitor;Utility;
StartupWMClass=gleamobserver
Terminal=false
EOF

    chmod 644 "$AUTOSTART_DIR/gleam-observer.desktop"
    print_success "Desktop entry created: $AUTOSTART_DIR/gleam-observer.desktop"
    print_info "Application will auto-start on next login"
}

# ============================================================================
# 7. CREAR SYSTEMD USER SERVICE
# ============================================================================

create_systemd_service() {
    print_step "STEP 7: Installing Systemd User Service"
    
    mkdir -p "$SYSTEMD_USER_DIR"
    
    cat > "$SYSTEMD_USER_DIR/gleam-observer.service" << EOF
[Unit]
Description=GleamObserver - System Monitor Service (Daemon + Systray)
After=graphical-session.target network.target
PartOf=graphical-session.target

[Service]
Type=simple
ExecStart=$INSTALL_PREFIX/bin/gleam --tray
Restart=always
RestartSec=2
Environment=DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/%U/bus
Environment=PATH=$INSTALL_PREFIX/bin:/usr/local/bin:/usr/bin:/bin
Environment=XDG_CONFIG_HOME=%h/.config

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=gleam-observer
TimeoutStopSec=30

[Install]
WantedBy=default.target graphical-session.target
EOF

    chmod 644 "$SYSTEMD_USER_DIR/gleam-observer.service"
    
    # Recargar systemd y habilitar servicio
    systemctl --user daemon-reload
    systemctl --user enable gleam-observer.service
    
    print_success "Systemd user service installed and enabled"
    print_info "Service location: $SYSTEMD_USER_DIR/gleam-observer.service"
}

# ============================================================================
# 8. CREAR CONFIGURACIÓN POR DEFECTO
# ============================================================================

create_default_config() {
    print_step "STEP 8: Creating Default Configuration"
    
    mkdir -p "$CONFIG_DIR"
    
    if [ -f "$CONFIG_DIR/config.toml" ]; then
        print_warning "Config already exists: $CONFIG_DIR/config.toml"
        read -p "Overwrite? (y/n) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            return
        fi
    fi
    
    cat > "$CONFIG_DIR/config.toml" << 'TOML'
# GleamObserver Configuration File
# Location: ~/.config/gleam_observer/config.toml

[general]
update_interval_ms = 1000
theme = "catppuccin"
default_profile = "default"
language = "en"

[display]
render_fps = 60
adaptive_refresh = true
refresh_on_idle = 5000

[alerts]
enabled = true
cooldown_sec = 60

[alerts.thresholds]
cpu_percent = 80.0
memory_percent = 85.0
gpu_temp_celsius = 85.0
gpu_util_percent = 95.0
disk_usage_percent = 90.0
network_mbps = 100.0

[history]
enabled = true
max_entries = 3600
retention_hours = 1

[trends]
enabled = true
min_confidence = 0.5
show_stable_trends = true

[gpu]
nvidia_enabled = true
amd_enabled = true
intel_enabled = false

[export]
csv_enabled = false
csv_path = "/tmp/gleam_observer/metrics.csv"
json_enabled = false
json_path = "/tmp/gleam_observer/metrics.jsonl"
TOML

    print_success "Default config created: $CONFIG_DIR/config.toml"
    print_info "You can customize thresholds and preferences in this file"
}

# ============================================================================
# 9. INSTALAR NOTIFICADOR (si falta)
# ============================================================================

install_notification_daemon() {
    print_step "STEP 9: Setting Up Notification System"
    
    if command -v notify-send &> /dev/null; then
        print_success "notify-send already installed"
    else
        print_warning "notify-send not found"
        read -p "Install notification daemon? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            local distro=$(detect_distro)
            case "$distro" in
                arch|manjaro)
                    sudo pacman -S --noconfirm libnotify
                    ;;
                debian|ubuntu|linuxmint)
                    sudo apt-get install -y libnotify-bin
                    ;;
                fedora|rhel|centos)
                    sudo dnf install -y libnotify
                    ;;
                *)
                    print_warning "Please install libnotify manually"
                    ;;
            esac
        fi
    fi
    
    # Verificar daemon de notificaciones (dunst, notify-daemon, etc)
    if ! pgrep -f "dunst|notify-daemon|xfce4-notifyd|makoctl" &> /dev/null; then
        print_warning "No notification daemon running"
        print_info "For GNOME/KDE: notifications are built-in"
        print_info "For other: install dunst, mako, or xfce4-notifyd"
    else
        print_success "Notification daemon is running"
    fi
}

# ============================================================================
# 10. INFORMACIÓN POST-INSTALACIÓN
# ============================================================================

print_post_install_info() {
    echo ""
    echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║     ✨ GleamObserver v$VERSION Installation Complete! ✨      ║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    echo -e "${GREEN}What's Installed:${NC}"
    echo "  ✓ Binary: $INSTALL_PREFIX/bin/gleam"
    echo "  ✓ Config: $CONFIG_DIR/config.toml"
    echo "  ✓ Service: $SYSTEMD_USER_DIR/gleam-observer.service"
    echo "  ✓ Autostart: $AUTOSTART_DIR/gleam-observer.desktop"
    echo "  ✓ Icon: $ICONS_DIR/gleamobserver.png"
    echo ""
    
    echo -e "${BLUE}Quick Start:${NC}"
    echo "  1. Start daemon now:"
    echo "     systemctl --user start gleam-observer"
    echo ""
    echo "  2. Check status:"
    echo "     systemctl --user status gleam-observer"
    echo ""
    echo "  3. View logs:"
    echo "     journalctl --user -u gleam-observer -f"
    echo ""
    
    echo -e "${YELLOW}What Happens Next:${NC}"
    echo "  • Service starts now and on every login"
    echo "  • Icon appears in system tray (look for 🌌)"
    echo "  • Alerts trigger when CPU/RAM/GPU exceed thresholds"
    echo "  • Click tray icon to open dashboard or configure"
    echo ""
    
    echo -e "${MAGENTA}Manage Daemon:${NC}"
    echo "  Stop:     systemctl --user stop gleam-observer"
    echo "  Restart:  systemctl --user restart gleam-observer"
    echo "  Disable:  systemctl --user disable gleam-observer"
    echo "  Logs:     journalctl --user -u gleam-observer -f"
    echo ""
    
    echo -e "${GREEN}Edit Configuration:${NC}"
    echo "  $CONFIG_DIR/config.toml"
    echo ""
    
    echo -e "${CYAN}📚 Documentation:${NC}"
    echo "  • GitHub: $REPO_URL"
    echo "  • Issues: $REPO_URL/issues"
    echo ""
}

# ============================================================================
# MAIN FLOW
# ============================================================================

main() {
    echo ""
    echo -e "${MAGENTA}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${MAGENTA}║  🌌 GleamObserver Daemon + Systray Installer v$VERSION      ║${NC}"
    echo -e "${MAGENTA}║     Cyberpunk Hardware Monitor - Always-On Mode            ║${NC}"
    echo -e "${MAGENTA}╚════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    # Flujo de instalación
    detect_display
    check_dependencies
    build_binary
    install_binary
    install_icon
    create_desktop_entry
    create_systemd_service
    create_default_config
    install_notification_daemon
    
    # Información final
    print_post_install_info
    
    echo -e "${GREEN}✨ Installation Complete! ✨${NC}"
    echo -e "${BLUE}Restart your session or run:${NC} systemctl --user start gleam-observer"
    echo ""
}

main "$@"
