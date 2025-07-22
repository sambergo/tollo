# Publishing Todo List - Tollo IPTV Player

This checklist covers all tasks needed before publishing Tollo to AUR, Cargo, and releasing binaries.

## High Priority Tasks

### ✅ Add MIT LICENSE file to project
**Status**: COMPLETED  
**Priority**: HIGH  
MIT License file has been added to the project root.

### 🔄 Update Cargo.toml metadata (author, license, repository, homepage, keywords)
**Status**: PENDING  
**Priority**: HIGH  
Need to update:
- `authors = ["you"]` → actual author info
- Add `license = "MIT"`
- Add `repository = "https://github.com/username/tollo"`
- Add `homepage = "https://github.com/username/tollo"`
- Add `keywords = ["iptv", "media-player", "tauri", "streaming"]`
- Add `readme = "README.md"`

### 🔄 Update package.json metadata and description
**Status**: PENDING  
**Priority**: HIGH  
Need to update:
- Add proper description
- Add repository field
- Add homepage field
- Add keywords array

### 🔄 Set up GitHub Actions workflow for cross-platform automated releases
**Status**: PENDING  
**Priority**: HIGH  
Create `.github/workflows/release.yml` for:
- Cross-platform builds (Windows, macOS, Linux)
- Automated tagging and releases
- Asset uploading (AppImage, .exe, .dmg, .deb)

## Medium Priority Tasks

### 🔄 Create PKGBUILD file for AUR submission
**Status**: PENDING  
**Priority**: MEDIUM  
Create PKGBUILD for Arch Linux packaging with proper dependencies and build instructions.

### 🔄 Improve README with installation guides, usage instructions, and features
**Status**: PENDING  
**Priority**: MEDIUM  
Expand README.md with:
- Installation instructions for different platforms
- Usage guide and screenshots
- Feature list and capabilities
- Build instructions
- Contributing guidelines

### 🔄 Create .desktop file for Linux desktop integration
**Status**: PENDING  
**Priority**: MEDIUM  
Add `tollo.desktop` file for proper Linux desktop integration.

## Low Priority Tasks

### 🔄 Configure code signing for Windows and macOS releases
**Status**: PENDING  
**Priority**: LOW  
Set up code signing certificates for trusted distribution on Windows and macOS.

### 🔄 Consider version bump to 0.2.0 or 1.0.0 when ready for stable release
**Status**: PENDING  
**Priority**: LOW  
Bump version from 0.1.0 to indicate stability and readiness for public release.

---

## Distribution Channels Checklist

### For Cargo/crates.io Publishing
- [ ] License field in Cargo.toml
- [ ] Proper metadata (author, description, keywords)
- [ ] Repository URL
- [ ] Consider if GUI app should be published to crates.io

### For AUR (Arch User Repository)
- [ ] PKGBUILD file
- [ ] .SRCINFO file
- [ ] Desktop entry file
- [ ] Tagged release on GitHub

### For Binary Releases
- [ ] GitHub Actions for cross-platform builds
- [ ] Release automation
- [ ] Platform-specific installers (AppImage, MSI, DMG)
- [ ] Installation documentation

## Notes
- Current version: 0.1.0 (marked as development/unstable)
- Tauri app with React/TypeScript frontend
- SQLite database, external player integration
- Well-structured codebase, ready for packaging