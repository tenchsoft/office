# Tench-Office 레포 추출 계획

## 원칙

- 완전히 독립적인 Cargo 워크스페이스
- 필요한 공유 크레이트는 물리적으로 복사
- 다른 레포와의 동기화 없음. 각 크레이트는 독립적으로 진화

---

## 앱

| 앱 | 패키지명 | 비고 |
|----|---------|------|
| docs | `tench-docs` | 문서 편집기 (.docx) |
| sheets | `tench-sheets` | 스프레드시트 |
| slides | `tench-slides` | 프레젠테이션 |
| kodocs | `tench-kodocs` | HWP 편집기 |

---

## 포함할 크레이트

| 크레이트 | 패키지명 | 직접 소비 앱 | 내부 의존성 |
|----------|---------|-------------|------------|
| tench-ui | `tench-ui` | docs, sheets, slides, kodocs | `ui-automation-core` |
| ui-automation-core | `tench-ui-automation-core` | 전체 (dev) | 없음 |
| tench-ui-test | `tench-ui-test` | 전체 (dev) | `tench-ui`, `ui-automation-core` |
| shared-types | `tench-shared-types` | docs, sheets, slides, kodocs | 없음 |
| storage-core | `tench-storage-core` | docs, sheets, slides, kodocs | 없음 |
| fs-core | `tench-fs-core` | docs, sheets, slides, kodocs | 없음 |
| document-core | `tench-document-core` | docs, sheets, slides, kodocs | 없음 |
| engine-core | `tench-engine-core` | docs, sheets, slides, kodocs | `engine-native`, `shared-types` |
| engine-native | `tench-engine-native` | (engine-core 경유) | 없음 |
| office-io | `tench-office-io` | docs, sheets, slides, kodocs | `document-core`, `fs-core`, `storage-core` |
| office-runtime | `tench-office-runtime` | docs, sheets, slides, kodocs | `document-core`, `hwp-io`, `office-io` |
| hwp-io | `tench-hwp-io` | kodocs, (office-runtime 경유) | `document-core` |
| sheets-core | `tench-sheets-core` | sheets | 없음 |

---

## 크레이트 의존성 그래프

```
tench-ui ──────── ui-automation-core
tench-ui-test ─── tench-ui, ui-automation-core

engine-core ───── engine-native, shared-types

office-io ─────── document-core, fs-core, storage-core
office-runtime ── document-core, hwp-io, office-io
hwp-io ────────── document-core

sheets-core (독립)
```

---

## 앱별 상세 의존성

### docs (`apps/docs/src-tauri`)

```
tench-document-core
tench-engine-core
tench-fs-core
tench-office-io
tench-shared-types
tench-storage-core
tench-office-runtime
tench-ui (features = ["tauri"])
tench-ui-test (dev)
tench-ui-automation-core (dev)
```

### sheets (`apps/sheets/src-tauri`)

```
tench-document-core
tench-engine-core
tench-fs-core
tench-office-io
tench-sheets-core
tench-shared-types
tench-storage-core
tench-office-runtime
tench-ui (features = ["tauri"])
tench-ui-automation-core (dev)
tench-ui-test (dev)
```

### slides (`apps/slides/src-tauri`)

```
tench-document-core
tench-engine-core
tench-fs-core
tench-office-io
tench-shared-types
tench-storage-core
tench-office-runtime
tench-ui (features = ["tauri"])
tench-ui-automation-core (dev)
tench-ui-test (dev)
```

### kodocs (`apps/kodocs/src-tauri`)

```
tench-document-core
tench-hwp-io
tench-engine-core
tench-fs-core
tench-office-io
tench-shared-types
tench-storage-core
tench-office-runtime
tench-ui (features = ["tauri"])
tench-ui-test (dev)
tench-ui-automation-core (dev)
```

---

## 디렉토리 구조

```
Tench-Office/
├── Cargo.toml              (워크스페이스 루트)
├── Cargo.lock
├── .gitea/
│   └── workflows/ci.yml
├── AGENTS.md
├── ARCHITECTURE.md
├── apps/
│   ├── docs/
│   │   └── src-tauri/
│   ├── sheets/
│   │   └── src-tauri/
│   ├── slides/
│   │   └── src-tauri/
│   └── kodocs/
│       └── src-tauri/
├── crates/
│   ├── tench-ui/
│   ├── ui-automation-core/
│   ├── tench-ui-test/
│   ├── shared-types/
│   ├── storage-core/
│   ├── fs-core/
│   ├── document-core/
│   ├── engine-core/
│   ├── engine-native/
│   ├── office-io/
│   ├── office-runtime/
│   ├── hwp-io/
│   └── sheets-core/
├── plans/
│   ├── spec/docs/
│   ├── spec/sheets/
│   ├── spec/slides/
│   ├── spec/kodocs/
│   ├── design/docs/
│   ├── design/sheets/
│   ├── design/slides/
│   ├── design/kodocs/
│   ├── background/docs/
│   ├── background/sheets/
│   ├── background/slides/
│   ├── background/kodocs/
│   ├── implement/docs/
│   ├── implement/sheets/
│   ├── implement/slides/
│   ├── implement/kodocs/
│   ├── test/docs/
│   ├── test/sheets/
│   ├── test/slides/
│   └── test/kodocs/
├── template/
└── tools/
    └── architecture-guard/
```

---

## 워크스페이스 설정

```toml
[workspace]
members = [
  "apps/docs/src-tauri",
  "apps/sheets/src-tauri",
  "apps/slides/src-tauri",
  "apps/kodocs/src-tauri",
  "crates/tench-ui",
  "crates/ui-automation-core",
  "crates/tench-ui-test",
  "crates/shared-types",
  "crates/storage-core",
  "crates/fs-core",
  "crates/document-core",
  "crates/engine-core",
  "crates/engine-native",
  "crates/office-io",
  "crates/office-runtime",
  "crates/hwp-io",
  "crates/sheets-core",
  "tools/architecture-guard",
]
resolver = "3"

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "UNLICENSED"
authors = ["Tench"]
```

---

## 이관 체크리스트

1. Gitea에 `Tench-Office` 빈 레포 생성
2. `apps/docs`, `apps/sheets`, `apps/slides`, `apps/kodocs` 복사
3. 13개 크레이트를 `crates/` 하위에 복사
4. `tools/architecture-guard` 복사, baseline을 이 레포 크레이트 13개로 재생성
5. 워크스페이스 루트 `Cargo.toml` 작성 (위 설정 기준)
6. `[workspace.dependencies]` 정리 — 이 레포에서 사용하는 외부 의존성만 남기기
7. 각 앱/크레이트의 `path` 참조 정리 — `path = "../../../crates/..."` → `path = "crates/..."` 로 통일
8. `cargo generate-lockfile` 실행
9. `.gitea/workflows/ci.yml` 작성
10. `AGENTS.md`, `ARCHITECTURE.md` 작성
11. `plans/` 하위에서 docs/sheets/slides/kodocs 관련 문서만 복사
12. `template/` 복사
13. `cargo check --workspace --locked` 통과 확인
14. `cargo test --workspace --locked` 통과 확인
15. Gitea CI 파이프라인 녹색 확인
