set shell := ["bash", "-euo", "pipefail", "-c"]

version := "0.4.0"
pkg_name := "oxide-explorer"
arch := "amd64"

# Default: build debug
default: build

# Build debug
build:
    cargo build

# Build release
release:
    cargo build --release

# Run debug
run:
    cargo run

# Run release
run-release:
    cargo run --release

# Build without terminal feature
build-no-terminal:
    cargo build --no-default-features

# Check compilation
check:
    cargo check

# Run clippy lints
lint:
    cargo clippy -- -W clippy::all

# Format code
fmt:
    cargo fmt

# Clean build artifacts
clean:
    cargo clean
    rm -rf {{pkg_name}}_*_{{arch}}.deb

# Build .deb package
deb: release
    #!/usr/bin/env bash
    set -euo pipefail
    PKG_DIR="{{pkg_name}}_{{version}}_{{arch}}"

    echo "Creating .deb package..."
    rm -rf "$PKG_DIR"
    mkdir -p "$PKG_DIR/DEBIAN"
    mkdir -p "$PKG_DIR/usr/bin"
    mkdir -p "$PKG_DIR/usr/share/oxide-explorer"
    mkdir -p "$PKG_DIR/usr/share/applications"
    mkdir -p "$PKG_DIR/usr/share/doc/oxide-explorer"

    cat > "$PKG_DIR/DEBIAN/control" << 'EOF'
    Package: oxide-explorer
    Version: {{version}}
    Section: utils
    Priority: optional
    Architecture: {{arch}}
    Depends: libgtk-4-1, libadwaita-1-0, libvte-2.91-gtk4-0
    Maintainer: Monzer Omer <monzer.a.omer@gmail.com>
    Description: Blazing-fast power-user file manager
     Oxide Explorer is a GTK4/libadwaita file manager built with Rust,
     designed for power users and developers. Features include tabbed
     browsing, dual-pane mode, embedded terminal, Miller columns,
     Lua plugin system, vim keybindings, advanced search, batch rename,
     archive management, file tagging, and network drive support.
    Homepage: https://github.com/oxide-explorer/oxide-explorer
    EOF
    # Remove leading whitespace from heredoc
    sed -i 's/^    //' "$PKG_DIR/DEBIAN/control"

    cp target/release/oxide-explorer "$PKG_DIR/usr/bin/"
    cp data/style.css "$PKG_DIR/usr/share/oxide-explorer/"
    cp data/com.oxide.explorer.desktop "$PKG_DIR/usr/share/applications/"
    cp LICENSE "$PKG_DIR/usr/share/doc/oxide-explorer/copyright"
    cp README.md "$PKG_DIR/usr/share/doc/oxide-explorer/"

    chmod 755 "$PKG_DIR/usr/bin/oxide-explorer"

    dpkg-deb --build "$PKG_DIR"
    rm -rf "$PKG_DIR"

    echo "Built: ${PKG_DIR}.deb"

# Install the .deb package
install: deb
    sudo dpkg -i {{pkg_name}}_{{version}}_{{arch}}.deb

# Uninstall
uninstall:
    sudo dpkg -r {{pkg_name}}

# Install build dependencies (Ubuntu/Debian)
deps:
    sudo apt install -y libgtk-4-dev libadwaita-1-dev libvte-2.91-gtk4-dev

# Show project stats
stats:
    @echo "Files:"
    @find src -name '*.rs' | wc -l
    @echo "Lines of Rust:"
    @find src -name '*.rs' -exec cat {} + | wc -l
    @echo "Binary size (release):"
    @ls -lh target/release/oxide-explorer 2>/dev/null || echo "  (not built yet)"
