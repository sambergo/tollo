# Windows Setup Instructions

This document provides instructions for building and running Töllö on Windows.

## Prerequisites

### Required Software
1. **Node.js** (v18 or later)
   - Download from https://nodejs.org/
   - Verify installation: `node --version`

2. **pnpm** (Package Manager)
   - Install globally: `npm install -g pnpm`
   - Verify installation: `pnpm --version`

3. **Rust** (latest stable)
   - Download from https://rustup.rs/
   - This will install both Rust and Cargo
   - Verify installation: `rustc --version` and `cargo --version`

4. **Visual Studio Build Tools** (for Rust compilation)
   - Download "Build Tools for Visual Studio 2022" from Microsoft
   - Or install Visual Studio Community with C++ build tools
   - Required for compiling native dependencies

### Windows-Specific Dependencies
- **WebView2**: Usually pre-installed on Windows 11, may need manual installation on Windows 10
  - Download from https://developer.microsoft.com/en-us/microsoft-edge/webview2/

## Building the Application

### Development Setup
1. Clone the repository and navigate to the project directory
2. Install Node.js dependencies:
   ```cmd
   pnpm install
   ```

3. Install Rust dependencies (automatically handled by Tauri):
   ```cmd
   cd src-tauri
   cargo fetch
   cd ..
   ```

### Development Mode
Start the development server:
```cmd
pnpm dev:tauri
```
This will:
- Start the Vite development server for the frontend
- Compile and run the Rust backend
- Open the application window

### Production Build
Build the complete application:
```cmd
pnpm build:tauri
```
This creates:
- An MSI installer in `src-tauri/target/release/bundle/msi/`
- An executable in `src-tauri/target/release/`

## Windows-Specific Configurations

### Binary Target Configuration
The `Cargo.toml` file includes a Windows-specific binary target configuration:
```toml
[[bin]]
name = "tollo"
path = "src/main.rs"
```
This is required for proper Windows compilation and prevents "entrypoint not found" errors.

### Windows Subsystem
The `main.rs` file includes Windows-specific attributes:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```
This prevents a console window from appearing in release builds.

### Bundle Configuration
The `tauri.conf.json` includes Windows-specific bundle settings for MSI generation.

## Troubleshooting

### "Entrypoint not found" Error (STATUS_ENTRYPOINT_NOT_FOUND)
This error typically occurs due to missing runtime dependencies or DLL issues. Try these solutions in order:

1. **Install Visual C++ Redistributable**:
   - Download and install "Microsoft Visual C++ Redistributable for Visual Studio 2015-2022"
   - Available from Microsoft's official website
   - This provides essential runtime libraries

2. **Clean and rebuild completely**:
   ```cmd
   pnpm clean
   cd src-tauri
   cargo clean
   cd ..
   pnpm install
   pnpm build:tauri
   ```

3. **Check binary target configuration**:
   - Ensure the `[[bin]]` section exists in `src-tauri/Cargo.toml`
   - Verify `main.rs` file exists and calls `tollo_lib::run()`

4. **Use static linking for reqwest** (already configured):
   - The project uses `rustls-tls` instead of system OpenSSL to avoid DLL dependencies

5. **Alternative: Use cargo run directly**:
   ```cmd
   cd src-tauri
   cargo run
   ```

### WebView2 Issues
If the application fails to start due to WebView2:
1. Download and install WebView2 Runtime manually
2. Restart the system after installation

### Build Tool Issues
If you encounter compilation errors:
1. Ensure Visual Studio Build Tools are installed
2. Restart your terminal/PowerShell after installation
3. Try building with verbose output: `cargo build --verbose`

### Permission Issues
If MSI installation fails:
1. Run the installer as Administrator
2. Check Windows Defender/antivirus settings
3. Ensure the certificate is trusted (for signed builds)

## Performance Notes

### Database Location
On Windows, the SQLite database is stored in:
```
%APPDATA%\com.tollo.app\data\database.sqlite
```

### Image Cache Location
Cached images are stored in:
```
%APPDATA%\com.tollo.app\image_cache\
```

### External Player Integration
The application can launch external media players like VLC or MPV. Ensure the player executable is in your system PATH or configure the full path in settings.

## Development Tips

### Hot Reload
During development, both frontend and backend support hot reload:
- Frontend changes reload automatically via Vite
- Rust changes require a rebuild but Tauri handles this automatically

### Debugging
For debugging on Windows:
1. Use `pnpm dev:tauri` for development mode with console output
2. Check the browser developer tools for frontend issues
3. Use `cargo run` in `src-tauri` directory for backend-only debugging

### Cross-Compilation
To build for other platforms from Windows:
```cmd
# For Linux (requires additional setup)
rustup target add x86_64-unknown-linux-gnu

# For macOS (requires additional setup)
rustup target add x86_64-apple-darwin
```
Note: Cross-compilation may require additional dependencies and configuration.