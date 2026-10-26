# Structured Notes

A tree-structured note keeper for the desktop: nodes form a tree, topics (rich text with tags, images and file attachments) live in nodes. Topics can be copied, moved, linked into several nodes, favorited and searched.

This is the desktop edition of the Laravel app: the same interface, running offline on your computer with all data stored locally.

**Tauri 2** (Rust), **Vue 3**, **shadcn-vue** and **Tailwind CSS 4**.

## Stack

- Rust backend with SQLite ([rusqlite](https://github.com/rusqlite/rusqlite), bundled, WAL mode) and versioned migrations
- Rich text content is sanitized in Rust with [ammonia](https://github.com/rust-ammonia/ammonia) before it is stored
- Topic images and attachments are stored as files next to the database and served to the webview through a custom `media://` protocol
- Vue 3 with `vue-router`, TipTap editor, shadcn-vue (reka-ui) components and Tailwind CSS 4
- Strict CSP, minimal capabilities, window size/position restored between runs

## Data

Everything lives in the per-user app data folder (`com.xoder.structurednotes`), e.g. on Windows `%APPDATA%\com.xoder.structurednotes`:

- `notes.db` – the SQLite database
- `files/` – images and attachments

Back up or move the whole folder to migrate your notes.

## Differences from the web edition

The desktop app is single-user, so accounts, two-factor/passkeys, user administration, public/private visibility and public topic pages are gone. Everything else is kept: tree, topics, tags, favorites, search (exact phrase / all words / titles only / by tag), cut/copy/link/paste, images and file attachments, light/dark/system appearance.

## Development

Prerequisites: [Rust](https://rustup.rs) (stable), Node.js 24+ and the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your OS.

```bash
npm install
npm run tauri dev       # run the app with hot reload
npm run tauri build     # produce an installer in src-tauri/target/release/bundle
```

## Checks

```bash
npm run build                                   # vue-tsc + vite build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets
cargo test --manifest-path src-tauri/Cargo.toml
```
