# TermIDE

A fast, efficient terminal-based text editor built with Rust.

## Build Prerequisites

### Linux

On Linux systems, you'll need X11 development libraries for clipboard support:

**Ubuntu/Debian:**
```bash
sudo apt-get install libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

**Fedora/RHEL:**
```bash
sudo dnf install libxcb-devel
```

**Arch Linux:**
```bash
sudo pacman -S libxcb
```

**Note:** TermIDE uses the system clipboard via X11 on Linux. If you're running in a headless environment or without X11, the editor will automatically fall back to an internal clipboard that works within the editor session.

### macOS

No additional dependencies required. Clipboard support uses native Cocoa frameworks.

### Windows

No additional dependencies required. Clipboard support uses native Windows API.

## Testing

The project includes a comprehensive test suite with >80% code coverage.

### Running Tests

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --lib

# Run integration tests
cargo test --test integration_tests

# Run specific module tests
cargo test buffer::tests
```
