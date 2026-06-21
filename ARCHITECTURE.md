# Repository Architecture

This repo contains the Tench Office suite — docs, sheets, slides, and kodocs.
The repo is organized around shared foundations first, product shells second.

## Layers

```text
apps/*              Thin product app shells (Tauri 2)
crates/*-core       Shared Rust domain and platform contracts
crates/tench-ui     Self-built retained-mode UI framework
crates/tench-ui-test  Headless UI test harness
tools/*             Repo automation and CI entrypoints
```

## Shared Feature Ownership

| Shared area | Rust crate | Reused by |
| --- | --- | --- |
| UI framework | `tench-ui` | every app |
| UI test harness | `tench-ui-test` | every app (dev) |
| UI automation protocol | `ui-automation-core` | every app (dev) |
| Documents/annotations/office content | `document-core` | docs, sheets, slides, kodocs |
| Local files/permissions | `fs-core` | docs, sheets, slides, kodocs |
| Local storage policy | `storage-core` | docs, sheets, slides, kodocs |
| Engine routing | `engine-core` | docs, sheets, slides, kodocs |
| Native engine discovery | `engine-native` | engine-core |
| Shared types | `shared-types` | engine-core |
| Office file I/O | `office-io` | docs, sheets, slides, kodocs |
| Office runtime dispatch | `office-runtime` | docs, sheets, slides, kodocs |
| HWP/HWPX I/O | `hwp-io` | kodocs, office-runtime |
| Spreadsheet domain | `sheets-core` | sheets |

## Product Shell Rule

Product apps should only own product-specific composition and domain glue. If a
feature appears in multiple plan directories, it starts in a shared crate.

## Plan Mapping

| Plans | App slot | Primary shared crates |
| --- | --- | --- |
| `docs` | `apps/docs` | `document-core`, `fs-core`, `storage-core`, `office-io`, `office-runtime`, `engine-core`, `tench-ui` |
| `sheets` | `apps/sheets` | `document-core`, `fs-core`, `storage-core`, `office-io`, `office-runtime`, `engine-core`, `sheets-core`, `tench-ui` |
| `slides` | `apps/slides` | `document-core`, `fs-core`, `storage-core`, `office-io`, `office-runtime`, `engine-core`, `tench-ui` |
| `kodocs` | `apps/kodocs` | `document-core`, `fs-core`, `storage-core`, `office-io`, `office-runtime`, `hwp-io`, `engine-core`, `tench-ui` |
