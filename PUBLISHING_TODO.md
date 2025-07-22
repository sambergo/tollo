# Publishing Todo List - Tollo IPTV Player

## GitHub Release Ready Tasks (Current Focus)

### ✅ Add MIT LICENSE file to project
**Status**: COMPLETED  
**Priority**: HIGH  
MIT License file has been added to the project root.

### ✅ Change all Töllö mentions to Tollo throughout project  
**Status**: COMPLETED  
**Priority**: HIGH  
All references updated across configuration files, documentation, and source code.

### 🔄 Improve README for GitHub users (installation, build instructions, features)
**Status**: PENDING  
**Priority**: HIGH  
Expand README.md with:
- Clear project description and features
- Prerequisites and system requirements
- Step-by-step build instructions
- Usage guide with screenshots/demo
- Development setup instructions
- Contributing guidelines

### 🔄 Update basic metadata in Cargo.toml and package.json (author, description)
**Status**: PENDING  
**Priority**: HIGH  
Update basic fields:
- Fix `authors = ["you"]` → actual author info
- Add `license = "MIT"` to Cargo.toml
- Improve descriptions
- Add basic repository info when available

### 🔄 Add CONTRIBUTING.md with development setup and guidelines
**Status**: PENDING  
**Priority**: MEDIUM  
Create contributor guidelines covering:
- Development environment setup
- Code style and conventions
- Pull request process
- Issue reporting

### 🔄 Add detailed build instructions to README
**Status**: PENDING  
**Priority**: MEDIUM  
Include:
- Prerequisites (Node.js, Rust, system dependencies)
- Platform-specific build steps
- Troubleshooting common issues

### 🔄 Add screenshots or demo to README
**Status**: PENDING  
**Priority**: MEDIUM  
Visual documentation to help users understand the app.

### 🔄 Document system dependencies and requirements
**Status**: PENDING  
**Priority**: MEDIUM  
List required system packages, external players (MPV), etc.

### 🔄 Clean up any development-only files not needed for users
**Status**: PENDING  
**Priority**: LOW  
Review for unnecessary dev files, configs, or data.

---

## Future Distribution Channels (After GitHub Ready)

### For Automated Releases & CI/CD
- [ ] Set up GitHub Actions workflow for cross-platform automated releases
- [ ] Create release automation with proper versioning
- [ ] Configure code signing for Windows and macOS releases
- [ ] Set up asset uploading (AppImage, .exe, .dmg, .deb)

### For AUR (Arch User Repository)
- [ ] Create PKGBUILD file for Arch Linux packaging
- [ ] Create .SRCINFO file
- [ ] Add desktop entry file for Linux integration
- [ ] Ensure tagged releases on GitHub
- [ ] Complete metadata (repository URLs, keywords)

### For Cargo/crates.io Publishing
- [ ] Complete Cargo.toml metadata (repository, homepage, keywords)
- [ ] Consider if GUI app should be published to crates.io
- [ ] Add comprehensive documentation

### For Binary Releases
- [ ] Platform-specific installers (AppImage, MSI, DMG)
- [ ] Installation documentation per platform
- [ ] Version bump to stable release (0.2.0 or 1.0.0)

---

## GitHub Ready Checklist
- [x] MIT License added
- [x] App name updated from Töllö to Tollo
- [ ] README enhanced for new users
- [ ] Build instructions clear and complete
- [ ] Basic metadata updated
- [ ] Contributing guidelines added
- [ ] System requirements documented
- [ ] Screenshots/demo included

## Notes
- Current version: 0.1.0 (marked as development/unstable)
- Tauri app with React/TypeScript frontend  
- SQLite database, external player integration
- Well-structured codebase, ready for GitHub users