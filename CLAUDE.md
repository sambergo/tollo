# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tollo is a modern IPTV player built with Tauri (Rust backend) and React/TypeScript frontend. It allows users to manage IPTV playlists, browse channels, search content, and play media through external players.

## Development Commands

### Frontend (React/TypeScript)
- `pnpm dev` - Start Vite development server
- `pnpm build` - Build frontend (runs TypeScript compiler + Vite build)
- `pnpm type-check` - Run TypeScript type checking without emitting
- `pnpm lint` - Run linting (currently just TypeScript type checking)
- `pnpm format` - Format code with Prettier
- `pnpm format:check` - Check code formatting

### Tauri (Full Application)
- `pnpm start` or `pnpm dev:tauri` - Start Tauri development mode
- `pnpm build:tauri` - Build complete Tauri application
- `pnpm check:tauri` - Run Tauri configuration checks

### Rust Backend
- `cd src-tauri && cargo build` - Build Rust backend
- `cd src-tauri && cargo check` - Quick check Rust code
- `cd src-tauri && cargo test` - Run Rust tests

### Cleanup
- `pnpm clean` - Remove dist and target directories
- `pnpm clean:full` - Full cleanup including node_modules and lock files
- `pnpm install:all` - Install all dependencies (Node.js + Rust)

## Architecture Overview

### Frontend Structure
- **State Management**: Zustand stores in `src/stores/` for channels, UI, search, filters, settings
- **Components**: React components in `src/components/` with CSS modules in `src/styles/`
- **Hooks**: Custom hooks in `src/hooks/` for keyboard navigation, search, image caching
- **Types**: TypeScript definitions in `src/types/`

### Backend Structure (Rust)
- **Main Entry**: `src-tauri/src/lib.rs` - Application setup and command registration
- **Database**: SQLite database management in `database.rs`
- **Core Modules**:
  - `channels.rs` - Channel data and operations
  - `playlists.rs` - M3U playlist management
  - `m3u_parser.rs` - M3U file parsing
  - `image_cache.rs` - Image caching system
  - `search.rs` - Search functionality with caching
  - `favorites.rs` - Favorite channels management
  - `history.rs` - Channel viewing history
  - `settings.rs` - Application settings
  - `filters.rs` - Saved search filters

### Key Features
- **Async Operations**: Both sync and async versions of most commands for performance
- **Caching**: Smart caching for channels, search results, and images
- **External Player**: Integration with MPV and other media players
- **Keyboard Navigation**: Vim-like navigation (see KEYBINDINGS.md)
- **Group Management**: Channel organization by groups
- **Search**: Fast fuzzy search with result caching

## Database Schema
Uses SQLite with tables for:
- `channels` - Channel information
- `favorites` - User favorites
- `history` - Viewing history
- `settings` - Application settings
- `channel_lists` - IPTV playlist sources
- `groups` - Channel groups
- `saved_filters` - User-saved search filters

## Development Notes

### Tauri Commands
Commands are registered in `lib.rs` and organized by functionality. Most have both sync and async versions for different use cases.

### State Management
- Frontend uses Zustand for React state
- Backend uses Tauri's managed state with Arc<Mutex<T>> for thread safety
- Database state is shared across all backend operations

### Image Caching
Smart image caching system with:
- Async image downloads
- Cache size management
- Preloading for performance

### Search System
- Fuzzy search with smart caching
- Separate search cache for performance
- Cache warming for common searches

## File Locations
- Frontend source: `src/`
- Rust backend: `src-tauri/src/`
- Database: `src-tauri/data/database.sqlite`
- Tauri configuration: `src-tauri/tauri.conf.json`