# Implement: license-persistence

> 작성 시점과 실행 시점 사이 코드 변경 가능성. 위치는 항상 grep으로 재확인 후 변경.

## 목표
- spec(`plans/spec/docs/license-persistence.md`): 앱 시작 시 저장된 자격 증명을
  읽어 license 상태를 복원하거나 미인증으로 진입.
- background(`plans/background/docs/license-persistence.md`): `LicenseState` 수명 주기,
  파일 atomic write, device_id 도출, 자동화 노드 4개 노출.

## 영향 받는 영역

| 영역 | 무엇이 바뀌나 | 찾기 전략 |
|------|----------------|-----------|
| `crates/tench-license-store/` (신규 crate) | crate 전체 생성. Cargo.toml + src/lib.rs + 4개 모듈 | `glob crates/tench-license-store/**` |
| `Cargo.toml` (workspace root) | `members`에 `crates/tench-license-store` 추가 + workspace dep 등록 | `grep -n 'tench-storage-core' Cargo.toml` (인접 행에 추가) |
| `crates/tench-license-store/src/lib.rs` | `LicenseStore`, `LicenseState`, `LicenseStatus` pub 타입 정의 + re-exports | `pub struct LicenseStore` |
| `crates/tench-license-store/src/device_id.rs` | 플랫폼별 device_id 도출 (`device_id()` 함수) | `pub fn device_id` |
| `crates/tench-license-store/src/store.rs` | AES-256-GCM 파일 read/write + atomic rename | `pub fn load`, `pub fn save` |
| `crates/tench-license-store/src/pc_code.rs` | PC 요청 코드 인코딩/디코딩 | `pub fn encode_pc_request_code`, `pub fn decode_pc_request_code` |
| `crates/tench-license-store/src/error.rs` | `LicenseStoreError` enum (thiserror 사용 ❌ — 단순 enum) | `pub enum LicenseStoreError` |
| `apps/docs/src-tauri/src/main.rs` (및 다른 3 앱 main.rs) | 앱 시작 시 `LicenseStore::load_or_init()` 호출 → `Arc<LicenseStore>` 보관 | `grep -n 'fn main'` 또는 `grep -n 'fn run'` |
| `apps/docs/src-tauri/src/ui/mod.rs` (자동화 노드) | 4개 license 노드 emit | `grep -n 'fn docs_automation_nodes'` (4 apps 각각) |

## 필요한 변경

### 1. 신규 crate 스켈레톤

- **crate 위치**: `crates/tench-license-store/`
- **Cargo.toml**:
  - `[package]`: name = `tench-license-store`, `version.workspace = true`, `edition.workspace = true`, `license.workspace = true`, `authors.workspace = true`
  - `[dependencies]`: `aes-gcm.workspace = true`, `sha2.workspace = true`, `base64.workspace = true`, `serde.workspace = true`, `serde_json.workspace = true`, `getrandom.workspace = true`, `dirs.workspace = true`
  - 플랫폼별 (target cfg):
    - Windows: `windows-sys` (workspace dep 아직 없음 — `windows-sys = { version = "0.59", features = ["Win32_System_Registry", "Win32_Foundation"] }`)
    - macOS: 외부 dep 없음 (`libc` 기본 FFI로 IOPlatformUUID 호출). 또는 `core-foundation` crate 검토.
    - Linux: 외부 dep 없음 (`/etc/machine-id` 파일 읽기)
- **src/lib.rs**: 모듈 트리 선언 + `pub use` re-exports.
  - `pub mod device_id; pub mod store; pub mod pc_code; pub mod error;`
  - `pub use store::{LicenseStore, LicenseState, LicenseStatus};`
  - `pub use error::LicenseStoreError;`
  - `pub use pc_code::{encode_pc_request_code, decode_pc_request_code, PcRequestCode};`

### 2. `device_id` 도출 — `device_id.rs`

- **입력**: 없음 (호출 시점에 OS 식별자 읽기)
- **처리**:
  - `#[cfg(target_os = "windows")]`: `HKLM\SOFTWARE\Microsoft\Cryptography\MachineGuid` 레지스트리 읽기 (`RegOpenKeyExW` + `RegQueryValueExW`). UTF-16 → UTF-8 변환.
  - `#[cfg(target_os = "macos")]`: `ioreg -d2 -c IOPlatformExpertDevice` 명령 실행 후 `IOPlatformUUID` 값 추출. 또는 IOKit FFI 직접 호출 (선호).
  - `#[cfg(target_os = "linux")`: `/etc/machine-id` 읽기 (없으면 `/var/lib/dbus/machine-id`).
  - 공통: 추출된 원본 문자열 → `Sha256::digest().hex_encode()` → 64자 hex.
  - 폴백: 모든 단계 실패 시 `getrandom`로 16바이트 난수 → hex → "ephemeral-" prefix 붙여 영구 저장 불가 표시.
- **출력**: `Result<String, LicenseStoreError>` (성공 시 64자 hex).
- **캐싱**: `OnceCell<String>`으로 프로세스 당 1회만 계산.

### 3. `LicenseState` 정의 및 영속화 — `store.rs`

- **타입**:
  ```rust
  pub enum LicenseStatus { Unactivated, Active, Expired }  // Display impl로 문자열 반환
  
  pub struct LicenseState {
      pub device_id: String,
      pub license_key: Option<String>,
      pub device_token: Option<String>,
      pub token_expires_at: Option<String>,  // RFC 3339
      pub last_refreshed_at: Option<String>,
  }
  ```
  (`DateTime<Utc>` 대신 `String` 사용 — chrono dep 회피, 직렬화 단순화)

- **`LicenseStore`**:
  ```rust
  pub struct LicenseStore {
      state: RwLock<LicenseState>,
      path: PathBuf,
  }
  ```
- **메서드**:
  - `load_or_init(product: &str) -> Result<Arc<Self>>`: 경로 계산 + 파일 읽기 시도 + 실패 시 초기 상태로 생성. `Arc`로 감싸 반환.
  - `state(&self) -> LicenseState`: read lock 복사 반환.
  - `set_activated(&self, license_key, device_token, expires_at) -> Result<()>`: state 갱신 + atomic write.
  - `set_token(&self, device_token, expires_at) -> Result<()>`: token만 갱신 (갱신 흐름용).
  - `clear(&self) -> Result<()>`: license_key/token/expires_at을 None으로 + atomic write (release/unbind용).
  - `current_status(&self) -> LicenseStatus`: state 평가 (token 유효 + 만료 전 = Active).

- **파일 경로 계산**:
  - `dirs::data_dir()` 사용 (플랫폼별 자동).
  - Windows: `dirs::data_dir()` → `%APPDATA%` (또는 `%ROAMING_APPDATA%`).
  - macOS: `dirs::data_dir()` → `~/Library/Application Support`.
  - Linux: `dirs::data_dir()` → `~/.local/share`.
  - 하위 경로: `Tench/<product>/license-store.bin`. `<product>`는 `docs`, `sheets`, `slides`, `kodocs`.

- **파일 포맷**:
  ```
  [12바이트 nonce] [AES-256-GCM ciphertext + GCM tag]
  ```
  평문은 `serde_json::to_vec(&LicenseState)`. 버전 필드는 JSON 안에 `"v": 1` 추가.
  
  키 도출: HKDF-SHA256(`device_id`, salt=`b"tench-license-store-v1"`, info=`b"aes-key"`, length=32).

- **Atomic write**:
  - `<path>.tmp` 파일에 쓰기 → `fsync` → `std::fs::rename(<path>.tmp, <path>)`.
  - rename은 같은 filesystem 내에서 atomic 보장.

### 4. PC 요청 코드 — `pc_code.rs`

- **`PcRequestCode`** struct (serde Serialize/Deserialize):
  ```rust
  pub struct PcRequestCode {
      pub v: u8,                    // 항상 1
      pub device_id: String,
      pub device_meta: serde_json::Value,
      pub nonce: String,            // 8바이트 hex (16자)
      pub exp: u64,                 // Unix 초
  }
  ```
- **`encode_pc_request_code(device_id, device_meta, nonce) -> String`**:
  - `exp` = `now + 600초`
  - JSON 직렬화 → base64url → `"TENCHPC-"` prefix 붙여 반환.
- **`decode_pc_request_code(code) -> Result<PcRequestCode>`**:
  - prefix 검증 → base64url decode → JSON 파싱 → 필드 타입 검증.
- **nonce 생성 헬퍼**: `pub fn generate_nonce() -> String` (16자 hex).

### 5. 앱 시작 통합 — `main.rs` (4 앱 각각)

- **입력**: 없음 (main 함수 진입)
- **처리**:
  - Tauri `Builder` setup 전 (또는 setup 안)에서 `LicenseStore::load_or_init("<product>")` 호출.
  - 결과 `Arc<LicenseStore>`를 app state (`tauri::Manager::manage`)에 등록.
  - 실패 시 `tracing::error!` 후 `Arc::new(LicenseStore::ephemeral())` 폴백 (메모리 전용, 디스크 저장 ❌).
- **출력**: app state에 등록된 `Arc<LicenseStore>` — 다른 모듈이 `app.state::<Arc<LicenseStore>>()`로 접근.

### 6. 자동화 노드 4개 — `ui/mod.rs`의 automation emit (4 앱 각각)

- **입력**: `Arc<LicenseStore>` 참조
- **처리**: 매 repaint 시 `store.state()` 읽어 4개 노드 emit
- **출력**: automation inventory에 다음 노드 추가:

| debug_id | role | value |
|----------|------|-------|
| `<product>.license.state` | `Label` | `"unactivated"` / `"active"` / `"expired"` |
| `<product>.license.device_id` | `Label` | `"<sha256-hex-64chars>"` |
| `<product>.license.license_key` | `Label` | `"TENCH-..."` 또는 `""` |
| `<product>.license.expires_at` | `Label` | `"2026-07-21..."` 또는 `""` |

- `role`은 `AccessRole::Label` (테스트에서 값 비교만, 클릭 불가).

## 새 자동화 노드

(background `Observability` 표와 1:1 — 이미 위에 명시됨)

| debug_id | role | value | 노출 조건 |
|----------|------|-------|-----------|
| `<product>.license.state` | `Label` | `"unactivated"` / `"active"` / `"expired"` | 항상 (모든 상태) |
| `<product>.license.device_id` | `Label` | 64자 hex | device_id 도출 성공 시. 실패 시 `"ephemeral-..."` |
| `<product>.license.license_key` | `Label` | `"TENCH-..."` 또는 `""` | 항상 |
| `<product>.license.expires_at` | `Label` | RFC 3339 또는 `""` | 항상 |

## 의존

- **라이브러리 보강**: 없음 (모든 dep가 이미 workspace에 있거나 thin layer).
  - `aes-gcm`, `sha2`, `base64`, `serde`, `serde_json`, `getrandom`, `dirs` — workspace dep 존재.
  - `windows-sys` — workspace dep 아님. `Cargo.toml` `[workspace.dependencies]`에 추가 필요.
- **선행 implement**: 없음. 본 crate은 foundation.
- **영향 받는 implement** (이후 작성):
  - `license-tab-ui` — LicenseStore 읽기/쓰기
  - `update-check-weekly` — LicenseStore에서 token 읽기 + 갱신 시 쓰기

## 작업 절차 (실행 에이전트가 매번 따른다)
1. 본 문서 + spec + background를 먼저 읽음.
2. `crates/tench-license-store/` 가 없으면 생성. 있으면 현재 상태 먼저 읽기.
3. `Cargo.toml` workspace `members` 와 `[workspace.dependencies]`에 `tench-license-store` 추가되어 있는지 확인.
4. `windows-sys` 가 workspace deps에 없으면 추가 (`features = ["Win32_System_Registry", "Win32_Foundation"]`).
5. 각 모듈 (device_id, store, pc_code, error, lib)을 작성. background의 `LicenseState` 스키마와 Observability 표 그대로 반영.
6. `cargo check -p tench-license-store --locked` 로 단일 crate 컴파일 확인.
7. 각 앱의 `main.rs` (docs/sheets/slides/kodocs)에 `LicenseStore::load_or_init` 호출 추가.
8. 각 앱의 `ui/mod.rs` automation emit 함수에 4개 license 노드 추가.
9. `cargo check --workspace --locked` 통과 확인.
10. `cargo test -p tench-license-store --locked` 단위 테스트 통과 확인.
11. `cargo test --workspace --locked ui_automation` 통과 확인 (노드 노출 확인).
