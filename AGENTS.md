# Repository Guidelines

## Project Structure & Module Organization

Tollo is a Tauri 2 desktop application. The React/TypeScript frontend lives in `src/`: reusable UI is in `components/`, Zustand state in `stores/`, custom hooks in `hooks/`, shared types in `types/`, and CSS in `styles/`. Static assets belong in `public/` or `src/assets/`.

The Rust backend is under `src-tauri/`. Application setup and Tauri command registration are in `src-tauri/src/lib.rs`; feature modules such as playlists, search, caching, and database access are nearby. Benchmarks live in `src-tauri/benches/`, Tauri permissions in `src-tauri/capabilities/`, and configuration in `src-tauri/tauri.conf.json`. Do not commit generated `dist/` or `src-tauri/target/` content.

## Build, Test, and Development Commands

- `pnpm install`: install frontend and Tauri CLI dependencies.
- `pnpm dev`: run the Vite frontend only.
- `pnpm dev:tauri`: launch the complete desktop app in development mode.
- `pnpm build`: type-check and build the frontend.
- `pnpm build:tauri`: create platform application bundles.
- `pnpm type-check`: validate TypeScript without emitting files.
- `pnpm format:check`: check TypeScript, TSX, and CSS with Prettier.
- `cd src-tauri && cargo test`: run all Rust unit and integration tests.
- `cd src-tauri && cargo check`: quickly validate backend compilation.
- `cd src-tauri && cargo clippy --all-targets`: catch common Rust issues.

## Coding Style & Naming Conventions

Use Prettier (`pnpm format`) for frontend files and `cargo fmt` for Rust. Follow existing two-space indentation in TypeScript/CSS and standard rustfmt output. Name React components and files in `PascalCase`, hooks as `useSomething`, Zustand stores as `somethingStore`, and TypeScript helpers in `camelCase`. Rust modules and functions use `snake_case`; types and traits use `PascalCase`. Keep Tauri commands grouped with their feature module and registered in `lib.rs`.

## Testing Guidelines

Rust uses the built-in test harness, with focused unit tests beside implementation code and broader cases in `src-tauri/src/integration_tests.rs`. Name tests after observable behavior, use `tempfile` for isolated filesystem/database state, and add regression coverage with every backend fix. Run `cargo test` before submitting. No frontend test framework is currently configured; at minimum run `pnpm type-check` and manually exercise affected UI flows through `pnpm dev:tauri`.

## Commit & Pull Request Guidelines

Recent history mixes short imperative messages with Conventional Commit prefixes such as `feat:`. Prefer concise, present-tense subjects (`fix: preserve selection after refresh`) and keep each commit focused. Pull requests should explain the user-visible change, list verification commands, link relevant issues, and include screenshots or recordings for UI changes. Call out schema, configuration, or platform-specific impacts explicitly.
