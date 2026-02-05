# Build Instructions

## Prerequisites

- Rust 1.75 or later
- Dioxus CLI: `cargo install dioxus-cli`

## Desktop Builds

### Windows
```bash
cargo build --release --target x86_64-pc-windows-msvc
```

### macOS (Intel)
```bash
cargo build --release --target x86_64-apple-darwin
```

### macOS (Apple Silicon)
```bash
cargo build --release --target aarch64-apple-darwin
```

### Linux
```bash
cargo build --release
```

## Mobile Builds

### Android

Prerequisites:
- Android NDK
- Android SDK

```bash
# Add Android targets
rustup target add aarch64-linux-android armv7-linux-androideabi

# Build
dx build --platform android --release
```

### iOS

Prerequisites:
- Xcode
- iOS SDK

```bash
# Add iOS targets
rustup target add aarch64-apple-ios x86_64-apple-ios

# Build
dx build --platform ios --release
```

## Web Build

```bash
dx build --platform web --release
```

## Development

Run in development mode:
```bash
dx serve
```

Run tests:
```bash
cargo test
```

Run property-based tests:
```bash
cargo test --test proptest
```
