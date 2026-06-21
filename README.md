<div align="center">

# Tench Office

**A 100% Rust office suite. Local-first, AI-assisted, no cloud.**

Docs · Sheets · Slides · Kodocs — four editors that respect your machine, your files, and your time.

[![Language: Rust](https://img.shields.io/badge/Language-Rust-dea584.svg)](https://www.rust-lang.org/)
[![Framework: Tauri 2](https://img.shields.io/badge/Framework-Tauri_2-FFC140.svg)](https://v2.tauri.app/)
[![License: UNLICENSED](https://img.shields.io/badge/License-UNLICENSED-red.svg)](#license)
[![Status: Preview](https://img.shields.io/badge/Status-Preview-orange.svg)](#roadmap)
[![Pricing: $1/mo](https://img.shields.io/badge/Pricing-%241%2Fmo-1ca096.svg)](https://tenchsoft.com/pricing)

[English](README.md) · [한국어](README.ko.md)

</div>

---

## Overview

Tench Office is a cross-platform office suite built entirely in Rust with Tauri 2. It bundles four editors that run locally — no telemetry, no accounts, no server round-trips for editing. AI assistance flows through the local Tench Engine, so your documents never leave your machine.

The suite targets Windows, macOS, and Linux. A single `$1/month` subscription per device unlocks every update across all four editors — cancel anytime, keep the version you have.

## Products

| | Product | Description | Default format |
|---|---|---|---|
| 📝 | **Docs** | Word processor with track changes, comments, templates | `.docx` |
| 📊 | **Sheets** | Spreadsheet with 200+ formulas, charts, pivot tables | `.xlsx` |
| 🎤 | **Slides** | Presentations with AI layout, presenter mode | `.pptx` |
| 🇰🇷 | **Kodocs** | Korean document editor with full HWP/HWPX fidelity | `.hwp` / `.hwpx` |

Kodocs is the suite's Korean-market differentiator — a native HWP editor that respects 한글 and 한컴 포맷 정밀하게.

## Features

- **100% Rust** — engine, UI, formats, AI plumbing. No JavaScript anywhere.
- **Local-first AI** — inference runs through Tench Engine via IPC, never HTTP.
- **Format roundtrip** — `.docx`, `.xlsx`, `.pptx`, `.odt`, `.ods`, `.odp`, `.hwp`, `.hwpx`, Markdown, PDF, HTML, plain text.
- **Privacy by construction** — files stay on your disk; no analytics, no auto-update of "the cloud".
- **Cross-platform** — Windows, macOS, Linux, Android, iOS from a single Rust codebase.
- **Self-built UI** — `tench-ui` is an in-house retained-mode widget framework rendered via Vello + wgpu. No Electron, no web view.

## Architecture

```
apps/<product>/src-tauri/        Thin product shells (Tauri 2)
crates/document-core/            Shared document model + engine
crates/office-io/                Format readers/writers (docx, xlsx, pptx, odt, ...)
crates/office-runtime/           Cross-product dispatch (open, save, export)
crates/hwp-io/                   HWP / HWPX precision I/O
crates/sheets-core/              Formula engine, grid model
crates/engine-core/              Tench Engine client (routing, providers)
crates/engine-native/            Local model discovery (GGUF, SafeTensors)
crates/storage-core/             Local-first persistence + encryption (AES-GCM)
crates/fs-core/                  File-system access policy
crates/tench-ui/                 Self-built widget framework
crates/tench-ui-test/            Headless visual capture / E2E harness
tools/architecture-guard/        Repo structure enforcement
```

See [`ARCHITECTURE.md`](ARCHITECTURE.md) for the full layer map and plan-folder convention.

## Build

```bash
# Prerequisites: Rust stable, platform build tools (MSVC / Xcode / gcc).
cargo check --workspace --locked
cargo build --workspace --locked
cargo test --workspace --locked
```

Run a single product:

```bash
cargo run --locked -p docs    # or: sheets, slides, kodocs
```

## Roadmap

- [x] Initial extraction from Tench-One monorepo
- [x] Shared office-io + office-runtime layers
- [x] HWP/HWPX precision I/O (`hwp-io`)
- [ ] Docs/Sheets/Slides first stable release
- [ ] Kodocs 1.0 (한글 템플릿 라이브러리 + 맞춤법)
- [ ] Mobile companion apps

## Pricing

- **$1 / month per device** — every update, every patch, every new feature.
- **Bulk** (5+ devices) — 30% off, $0.70/device/month.
- Cancel anytime, keep the version you have. No lock-out.

→ https://tenchsoft.com/pricing

## License

UNLICENSED — source available for review, binary distribution requires a subscription. See `LICENSE` (or contact hello@tenchsoft.com).

## Sister Projects

- **[Tench Engine](https://github.com/tenchsoft/engine)** — local AI inference runtime used by every Tench app.
- **[Tench Media](https://github.com/tenchsoft/media)** — image / video suite (View, Pixel Design, Player, Composer).
- **[Tench Authoring](https://github.com/tenchsoft/authoring)** — long-form writing tools (Story, Universe).
- **[Tench Knowledge](https://github.com/tenchsoft/knowledge)** — research & study (Research, Study).
- **[Tench Code](https://github.com/tenchsoft/code)** — AI-augmented code editor.
- **[tenchsoft.com](https://tenchsoft.com)** — account, license, downloads.
- **[Dolphin Labs](https://dolphinelabs.com)** — open-source sister label (mymy, OpenNodia).
