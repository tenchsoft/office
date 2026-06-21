# Tench-Office 레포 추출 계획

## Objective

`~/tench/Tench-One` 모노레포에서 오피스 제품군(docs, sheets, slides, kodocs)과 관련 13개 크레이트, 도구, 계획 문서를 완전히 독립적인 `Tench-Office` 워크스페이스로 추출한다. 추출된 레포는 다른 레포와의 동기화 없이 독립적으로 진화한다.

---

## 소스 분석 요약

### 현재 Tench-One 구조

- **51개 워크스페이스 멤버**: 15개 앱 + 34개 크레이트 + 2개 도구
- 추출 대상: 4개 앱 + 13개 크레이트 + 1개 도구 = **18개 워크스페이스 멤버**

### 추출 대상 13개 크레이트 확인 (전부 존재)

| 크레이트 | 내부 의존성 | 외부 전용 의존성 |
|----------|------------|-----------------|
| `shared-types` | 없음 | `serde`, `serde_json` |
| `storage-core` | 없음 | `getrandom`, `serde`, `aes-gcm`, `sha2` |
| `fs-core` | 없음 | `serde` |
| `document-core` | 없음 | `regex`, `serde`, `serde_json` |
| `engine-native` | 없음 | `serde`, `serde_json` |
| `ui-automation-core` | 없음 | `serde` |
| `sheets-core` | 없음 | 없음 |
| `shared-types` | 없음 | `serde`, `serde_json` |
| `engine-core` | `engine-native`, `shared-types` | `dirs`, `regex`, `toml`, `ureq`, `aes-gcm`, `sha2`, `base64`, `getrandom` |
| `tench-ui` | `ui-automation-core` | `vello`, `parley`, `accesskit`, `kurbo`, `peniko`, `smallvec`, `log`, `image` + 선택적(`tauri`, `pollster`, `winit`, `tray-icon`, `reqwest`, `tokio`, `serde`, `serde_json`) |
| `tench-ui-test` | `tench-ui`(path), `ui-automation-core` | `accesskit`, `image`, `kurbo`, `pollster` |
| `office-io` | `document-core`, `fs-core`, `storage-core` | `zip` |
| `office-runtime` | `document-core`, `hwp-io`, `office-io` | 없음 |
| `hwp-io` | `document-core`(path) | `cfb`, `flate2`, `byteorder`, `serde`, `serde_json`, `zip`, `quick-xml`, `thiserror` |

### 경로 참조 이슈 (수정 필요)

소스에서 일부 크레이트가 workspace 참조 대신 **상대 경로**를 사용:

1. **`tench-ui-test`** (`crates/tench-ui-test/Cargo.toml:10`): `tench-ui = { path = "../tench-ui" }` → 워크스페이스 참조로 변경 필요
2. **`hwp-io`** (`crates/hwp-io/Cargo.toml:10`): `tench-document-core = { path = "../document-core" }` → 워크스페이스 참조로 변경 필요
3. **`docs` 앱** (`apps/docs/src-tauri/Cargo.toml:28`): `tench-ui = { path = "../../../crates/tench-ui", ... }` → 워크스페이스 참조로 변경 필요
4. **`docs` 앱** (`apps/docs/src-tauri/Cargo.toml:31-32`): dev-dep도 path 참조 → 워크스페이스 참조로 변경 필요
5. **`sheets` 앱** (`apps/sheets/src-tauri/Cargo.toml:29`): `tench-ui` path 참조 → 변경 필요
6. **`slides` 앱** (`apps/slides/src-tauri/Cargo.toml:28`): `tench-ui` path 참조 → 변경 필요
7. **`kodocs` 앱** (`apps/kodocs/src-tauri/Cargo.toml:22`): `tench-hwp-io` path 참조 → 변경 필요
8. **`kodocs` 앱** (`apps/kodocs/src-tauri/Cargo.toml:29`): `tench-ui` path 참조 → 변경 필요
9. **`kodocs` 앱** (`apps/kodocs/src-tauri/Cargo.toml:32-33`): dev-dep path 참조 → 변경 필요

### CI에서 제외해야 할 도구

소스 CI는 `tench-ci-core` (금지 명령 스캐너)와 `tench-workspace-guard`를 사용하지만, 이 크레이트들은 추출 대상이 아님. 새 레포의 CI에서는 해당 단계를 제거하거나 대체해야 함.

### Architecture-Guard Baseline 필터링

현재 `line_budget_baseline.txt`는 347개 항목이 있으며, 이중 docs/sheets/slides/kodocs 및 13개 크레이트에 해당하는 항목만 추출해야 함.

---

## Implementation Plan

### Phase 1: 디렉토리 복사

- [x] **1.1** `~/tench/Tench-Office/` 디렉토리 생성 (없는 경우)
- [x] **1.2** 4개 앱 복사
- [x] **1.3** 13개 크레이트 복사
- [x] **1.4** `tools/architecture-guard/` 복사
- [x] **1.5** `template/` 전체 복사
- [x] **1.6** `plans/` 하위에서 docs/sheets/slides/kodocs 관련 문서만 복사
- [x] **1.7** `.gitignore` 확인 및 복사

### Phase 2: 워크스페이스 루트 Cargo.toml 작성

- [x] **2.1** `~/tench/Tench-Office/Cargo.toml` 작성

- [x] **2.2** `[workspace.dependencies]`에 포함할 외부 의존성 목록

- [x] **2.3** `[workspace.dependencies]`에 포함할 내부 크레이트 path 참조 13개

### Phase 3: Cargo.toml 경로 참조 정리

- [x] **3.1** `crates/tench-ui-test/Cargo.toml` 수정
- [x] **3.2** `crates/hwp-io/Cargo.toml` 수정
- [x] **3.3** `apps/docs/src-tauri/Cargo.toml` 수정
- [x] **3.4** `apps/sheets/src-tauri/Cargo.toml` 수정
- [x] **3.5** `apps/slides/src-tauri/Cargo.toml` 수정
- [x] **3.6** `apps/kodocs/src-tauri/Cargo.toml` 수정
- [x] **3.7** `crates/tench-ui/Cargo.toml` 확인: OK
- [x] **3.8** 모든 크레이트의 workspace 참조 확인 완료

### Phase 4: Architecture-Guard Baseline 재생성

- [x] **4.1** `tools/architecture-guard/line_budget_baseline.txt`에서 추출 대상이 아닌 앱/크레이트 항목 제거 (347→163 lines)
- [x] **4.2** `tools/architecture-guard/main.rs`의 `COMMAND_BUDGET_BASELINE` 상수 확인 — OK (docs/kodocs만 참조)
- [x] **4.3** baseline 라인 카운트 일치 확인 (파일 변경 없음으로 동일)

### Phase 5: CI 설정

- [x] **5.1** `.gitea/workflows/ci.yml` 작성
- [x] **5.2** Gitea remote URL을 `Tench/Tench-Office`로 설정

### Phase 6: 문서 파일 작성

- [x] **6.1** `AGENTS.md` 작성
- [x] **6.2** `ARCHITECTURE.md` 작성

### Phase 7: 빌드 검증

- [x] **7.1** `cargo generate-lockfile` 실행 — Cargo.lock 생성
- [x] **7.2** `cargo check --workspace --locked` 실행 — 통과
- [x] **7.3** `cargo test --workspace --locked --lib` 실행 — 58+38+5+4=105 유닛 테스트 통과
- [x] **7.4** `cargo test --workspace --locked --tests` 실행 — 1603 테스트 통과, 0 실패
- [x] **7.5** `cargo fmt --all -- --check` 실행 — 통과
- [x] **7.6** `cargo clippy --workspace --locked --all-targets -- -D warnings` 실행 — 통과

### Phase 8: Git 초기화 및 푸시

- [x] **8.1** `~/tench/Tench-Office/`에서 `git init` + 초기 커밋 (3949 files, 262367 insertions)
- [ ] **8.2** Gitea에 `Tench-Office` 빈 레포 생성 (API 토큰에 write:organization 스코프 없음 — 수동 생성 필요)
- [x] **8.3** remote origin을 `git@tench_gitea:Tench/Tench-Office.git`으로 설정 완료
- [ ] **8.4** 초기 커밋 push (8.2 레포 생성 후 가능)
- [ ] **8.5** Gitea CI 파이프라인 녹색 확인 (8.4 push 후 가능)

---

## Verification Criteria

- [x] `cargo check --workspace --locked` 통과
- [x] `cargo test --workspace --locked` 통과 (1603 tests, 0 failures)
- [x] `cargo fmt --all -- --check` 통과
- [x] `cargo clippy --workspace --locked --all-targets -- -D warnings` 통과
- [x] `cargo run --locked -p tench-architecture-guard -- --strict` 통과 (527 files, 174 Tauri commands)
- [x] 워크스페이스 멤버가 정확히 18개 (4 앱 + 13 크레이트 + 1 도구)
- [x] 모든 path 참조가 workspace 참조로 통일됨 (상대 경로 잔여 없음)
- [x] `Cargo.lock`이 존재하고 `cargo check --locked`가 통과
- [ ] Gitea CI 파이프라인 녹색 (레포 생성 후 push 필요)
- [x] 다른 레포(Tench-One 등)에 대한 의존성이 전혀 없음

---

## Potential Risks and Mitigations

1. **숨겨진 내부 크레이트 의존성**
   - 13개 크레이트가 추출 대상이 아닌 크레이트(예: `app-core`, `search-core`)를 암시적으로 의존할 가능성
   - 완화: `cargo check --workspace`로 미사용 크레이트 의존성 여부 확인. 현재 Cargo.toml 분석 결과 모든 내부 의존성이 13개 크레이트 내에서 해결됨을 확인

2. **외부 의존성 누락**
   - `hwp-io`가 소스에서 `cfb`, `flate2`, `byteorder`, `quick-xml`, `thiserror`를 직접 참조 (workspace 미경유)
   - 완화: 각 크레이트의 Cargo.toml을 직접 확인하여 workspace.dependencies에 누락 없이 포함

3. **tench-ui의 선택적 의존성 충돌**
   - `tench-ui`는 `vello`, `parley`, `accesskit` 등을 직접 버전 지정 (workspace 미경유)
   - 완화: workspace.dependencies에 포함하되, tench-ui의 직접 버전 지정을 workspace 참조로 통일하거나 현행 유지

4. **CI runner에 Tauri 빌드 의존성 누락**
   - `cargo check`는 Tauri 시스템 라이브러리(webkit2gtk 등)를 필요로 함
   - 완화: Gitea runner에 이미 설치되어 있을 가능성 높음 (기존 Tench-One CI가 동일 runner 사용)

5. **Baseline 라인 카운트 불일치**
   - 복사 후 파일이 변경되지 않았더라도 line ending 차이로 카운트가 달라질 수 있음
   - 완화: 복사 후 `architecture-guard`를 먼저 실행하여 baseline 검증

6. **plans/ 디렉토리 내 비오피스 문서 참조**
   - 일부 plans 문서가 비오피스 크레이트를 참조할 수 있음
   - 완화: plans 문서는 참조용이므로 빌드에 영향 없음. 필요시 문서 내 링크 수정

---

## Alternative Approaches

1. **git subtree/filter 사용**: `git filter-branch` 또는 `git subtree`로 히스토리 유지하며 추출
   - 장점: Git 히스토리 보존
   - 단점: 모노레포 구조에서 특정 앱/크레이트만 필터링하기 복잡, Cargo 설정 충돌 가능성
   - 현재 계획(물리적 복사)이 단순하고 확실함

2. **Cargo workspace 가상 매니페스트만 교체**: 기존 디렉토리 구조를 유지하며 Cargo.toml만 수정
   - 장점: 파일 복사 불필요
   - 단점: Tench-One과 물리적으로 공유되어 독립 진화 원칙 위배

3. **git clone 후 불필요 항목 삭제**: 전체 클론 후 필요 없는 앱/크레이트 삭제
   - 장점: .git 히스토리 보존, 누락风险 감소
   - 단점: 히스토리에 비오피스 제품 정보 포함, `.git` 크기 과대
