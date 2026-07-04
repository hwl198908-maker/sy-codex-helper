# Inline Update Download Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace browser-based update downloads with an in-app update download flow that shows progress, reuses a cached installer for the same version, verifies SHA-256 when provided, and launches the installer.

**Architecture:** Keep the existing `latest.json` version check. Add one Tauri command that downloads the installer into the app data update cache, emits progress events, verifies the optional checksum, and starts the downloaded installer. Update the final step UI to call this command instead of opening the URL in a browser.

**Tech Stack:** Tauri 2, Rust `reqwest` blocking client, React, Mantine, Vitest, Cargo tests.

---

### Task 1: Update Manifest Shape

**Files:**
- Modify: `src-tauri/src/updater.rs`
- Modify: `src/lib/updates.ts`
- Modify: `src/lib/updates.test.ts`

- [ ] Add optional `sha256` to both Rust and TypeScript manifest types.
- [ ] Add tests that a manifest with `sha256` parses correctly.

### Task 2: Rust Download Command

**Files:**
- Modify: `src-tauri/src/updater.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] Add `download_and_install_update(app, manifest)` command.
- [ ] Validate HTTP(S) download URLs.
- [ ] Store downloads in `app_local_data_dir()/updates`.
- [ ] Reuse a cached installer when the version and optional checksum match.
- [ ] Emit `update-download-progress` events with percent, downloaded bytes, and total bytes.
- [ ] Launch the installer with `std::process::Command`.
- [ ] Add Rust tests for cache file naming and SHA-256 verification.

### Task 3: Final Step UI

**Files:**
- Modify: `src/components/CompleteStep.tsx`

- [ ] Remove browser download behavior.
- [ ] Listen for `update-download-progress`.
- [ ] Show progress while downloading.
- [ ] Rename the update action to "立即更新".
- [ ] Show cached installer status when the backend reuses an existing file.

### Task 4: Version and Manifest

**Files:**
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src/lib/defaults.ts`
- Server: `/var/www/html/codex-manager/latest.json`

- [ ] Bump app version to `0.1.9`.
- [ ] Build installer.
- [ ] Upload versioned installer and update `latest.json` with `sha256`.

### Task 5: Verification

- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml -- --nocapture`.
- [ ] Run `npm test`.
- [ ] Run `npm run build`.
- [ ] Run `npm run tauri build`.
- [ ] Verify the HTTPS manifest points to `0.1.9`.
- [ ] Download the online installer and confirm SHA-256 matches.
