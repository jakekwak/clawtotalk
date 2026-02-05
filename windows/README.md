# Windows Platform Files

This directory contains Windows-specific resources for the Dioxus Voice Assistant application.

## Files

### app.manifest
Windows application manifest that defines:
- Windows 10/11 compatibility
- DPI awareness for high-resolution displays
- Execution level (runs as normal user, no admin required)

### resource.rc
Windows resource file that includes:
- Application icon
- Version information
- Company and product metadata
- Manifest embedding

### icon.ico
Application icon file (256x256 recommended, with multiple sizes embedded)

## Creating an Icon

If `icon.ico` doesn't exist, you need to create one:

### Option 1: Online Tools
1. Visit https://www.icoconverter.com/ or https://convertio.co/png-ico/
2. Upload a PNG image (256x256 or larger)
3. Convert to ICO format with multiple sizes (16x16, 32x32, 48x48, 256x256)
4. Download and save as `windows/icon.ico`

### Option 2: Using ImageMagick
```bash
# Install ImageMagick first
# On Windows: choco install imagemagick
# On macOS: brew install imagemagick

# Convert PNG to ICO with multiple sizes
magick convert icon.png -define icon:auto-resize=256,128,64,48,32,16 icon.ico
```

### Option 3: Using GIMP
1. Open your image in GIMP
2. Scale to 256x256 (Image → Scale Image)
3. Export as ICO (File → Export As → icon.ico)
4. In the export dialog, select multiple sizes

## Building on Windows

The build script (`build.rs`) will automatically:
1. Link required Windows libraries (ole32, winmm, avrt)
2. Embed the resource file using `embed-resource` crate
3. Include the manifest and icon in the final executable

### Build Requirements
- Rust toolchain (MSVC or GNU)
- Windows SDK (for MSVC toolchain)
- Resource compiler (rc.exe for MSVC, windres for GNU)

### Build Commands
```bash
# Debug build
cargo build

# Release build with optimizations
cargo build --release

# The executable will be in target/release/dioxus-voice-assistant.exe
```

## Windows-Specific Features

### WASAPI Optimizations
- Exclusive mode for lower latency
- Event-driven audio processing
- MMCSS (Multimedia Class Scheduler Service) integration
- Configurable thread priorities

### Permission Handling
- Automatic microphone permission detection
- Registry-based privacy settings check
- Direct link to Windows Settings for microphone access

### Performance
- Low-latency audio (5-10ms buffer)
- High-priority audio threads
- Optimized for Windows 10/11

## Troubleshooting

### Icon Not Showing
- Ensure `icon.ico` exists in the `windows/` directory
- Rebuild the project completely: `cargo clean && cargo build --release`
- Windows may cache icons; try restarting Explorer or rebooting

### Build Errors
- If resource compilation fails, ensure Windows SDK is installed
- For GNU toolchain, install `mingw-w64` which includes `windres`
- Check that `embed-resource` crate is in `Cargo.toml`

### Permission Issues
- If microphone access is denied, run: `ms-settings:privacy-microphone`
- Ensure the app is allowed in Windows Privacy Settings
- Check antivirus software isn't blocking microphone access
