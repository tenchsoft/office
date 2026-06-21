# Test: license-persistence

## 검증 대상
spec(`plans/spec/docs/license-persistence.md`)의 acceptance criteria → 테스트 함수 매핑.

| Acceptance Criteria | 시나리오 (테스트 함수명) |
|---------------------|---------------------------|
| AC1: 앱 시작 후 1초 이내 상태 결정 | `license_persistence_loads_within_budget` |
| AC2: 활성화 이력 있으면 License 탭에 자동 표시 | `license_persistence_restores_activated_state` (UI는 `license-tab-ui`에서 검증, 본 테스트는 state 노드만) |
| AC3: 활성화 이력 없으면 빈 입력 상태 | `license_persistence_fresh_start_unactivated` |
| AC4: 활성 상태 - 알림 라벨 숨김 | `license_persistence_active_hides_label` (`update-notification-label`에서 통합 검증; 본 테스트는 state 노드만) |
| AC5: 미인증/만료 - 알림 라벨 표시 | `license_persistence_unactivated_shows_label_marker` |
| AC6: 손상된 파일 → 미인증 폴백 | `license_persistence_corrupted_file_falls_back` |
| AC7: 삭제된 파일 → 미인증 진입 (정상) | `license_persistence_missing_file_is_unactivated` |
| AC8: 같은 기기 매번 동일 device_id | `license_persistence_device_id_stable_across_runs` |

## 테스트 파일 위치

- 단위 테스트 (crate 단독): `crates/tench-license-store/tests/store_unit.rs`
- 통합 테스트 (automation 노드): `apps/docs/src-tauri/tests/license_persistence_ui_e2e.rs`

## Required Test Shape

### Success scenarios

- **신규 설치 → 미인증 진입**: 파일 없는 상태에서 `LicenseStore::load_or_init` →
  `state.license_key`가 `None`이고 `state.device_id`는 채워져 있음.
  → `license_persistence_fresh_start_unactivated`
- **활성화 저장 → 재시작 시 복원**: `set_activated(...)` 호출 후 새 인스턴스 로드 →
  동일 license_key/token/expires_at 복원됨.
  → `license_persistence_restores_activated_state`
- **device_id 안정성**: 같은 임시 디렉토리에서 두 번 `load_or_init` → 동일 device_id.
  (레지스트리/UUID 소스가 같다는 가정; CI에서는 CI 머신의 device_id가 안정적임을 전제)
  → `license_persistence_device_id_stable_across_runs`

### Failure/negative scenarios

- **파일 없음**: `load_or_init` 호출 시 파일이 존재하지 않음 → Ok 반환,
  state = unactivated. (정상 케이스)
  → `license_persistence_missing_file_is_unactivated`
- **파일 손상**: 임의의 100바이트 난수를 `license-store.bin`에 기록 →
  `load_or_init` → Ok 반환 (폴백), state = unactivated. 손상 파일은 그대로 남음.
  → `license_persistence_corrupted_file_falls_back`
- **잘못된 device_id 경로**: 의도적으로 OS 식별자 읽기를 fail시킬 수 없으므로
  (레지스트리/ioreg는 CI에서 항상 성공), 에러 경로는 mock 기반 단위 테스트에서만 검증.

### Edge cases

- **라운드트립 (set → save → reload → 동일 state)**: 활성화 후 save →
  drop → load_or_init → 모든 필드 동일. hex 비교까지 포함.
  → `license_persistence_roundtrip_preserves_state`
- **Atomic write 중단 시뮬레이션**: `.tmp`만 존재하고 본 파일 없음 →
  `load_or_init` 시도 → 미인증 폴백 (이전 파일 없음). `.tmp`는 무시됨.
  → `license_persistence_orphaned_tmp_ignored`
- **Clear 후 reload**: `set_activated(...)` → `clear()` → drop → load →
  `state.license_key`는 None이고 `device_id`는 유지.
  → `license_persistence_clear_keeps_device_id`
- **성능予산**: 4 KB state 기준 `load_or_init` < 50ms.
  → `license_persistence_loads_within_budget`

### UI integration scenarios (apps/docs 측)

- **앱 시작 시 노드 노출**: docs 앱을 headless harness로 부팅 → automation inventory에
  4개 노드 모두 존재 (`docs.license.state`, `.device_id`, `.license_key`, `.expires_at`).
  → `license_persistence_ui_nodes_exposed`
- **노드 값 일관성**: state=`unactivated` → `docs.license.state == "unactivated"`,
  `docs.license.license_key == ""`, `docs.license.expires_at == ""`.
  → `license_persistence_ui_unactivated_node_values`

## 사용할 자동화 노드

implement(`plans/implement/docs/license-persistence.md`)의 자동화 노드 표와 일치.

| debug_id | 검증 시점 | 기대 value/state |
|----------|------------|-------------------|
| `docs.license.state` | fresh start 후 | `"unactivated"` |
| `docs.license.state` | 활성화 후 | `"active"` |
| `docs.license.state` | 만료 token 갱신 실패 후 | `"expired"` |
| `docs.license.device_id` | 항상 | 64자 hex string |
| `docs.license.license_key` | 미활성화 시 | `""` |
| `docs.license.license_key` | 활성화 후 | `"TENCH-..."` (55자) |
| `docs.license.expires_at` | 미활성화 시 | `""` |
| `docs.license.expires_at` | 활성화 후 | RFC 3339 문자열 |

## 의존

- 선행 implement: `plans/implement/docs/license-persistence.md`
- 픽스처: 단위 테스트는 `tempfile::TempDir`로 격리된 디렉토리 사용 (workspace dep
  에 `tempfile` 추가 필요 — `[dev-dependencies]`).
- 다이얼로그 주입: 불필요.
- 환경 변수: device_id 도출 테스트를 위해 `TENCH_TEST_DEVICE_ID` 환경 변수가
  설정되어 있으면 OS 식별자 대신 그 값을 사용하도록 `device_id.rs`에 테스트 전용
  분기 추가 (`#[cfg(test)]`).

## Verification

```bash
# 단위 테스트 (crate 독립)
cargo test -p tench-license-store --locked

# 통합 UI 테스트 (docs 앱 기준)
cargo test -p tench-docs --locked license_persistence_ui_e2e

# 전체 workspace 체크
cargo check --workspace --locked
cargo test --workspace --locked ui_automation
```

## 작업 절차 (실행 에이전트가 매번 따른다)

1. spec과 implement를 먼저 읽음.
2. `crates/tench-license-store/tests/` 가 없으면 생성.
3. 단위 테스트 (`store_unit.rs`):
   - `tempfile::TempDir`로 격리 디렉토리 생성 후 `LicenseStore::load_or_init_with_dir` 호출.
     (프로덕션 코드는 `load_or_init(product)`이고, 테스트 전용 `load_or_init_with_dir(product, dir)`을 노출 — `#[cfg(test)]` 또는 `pub(crate)` visibility.)
   - 4가지 패턴 (A: 값 변이, B: 출현/소멸, C: 내용 반영, D: 라운드트립) 각각 1개 이상.
4. `apps/docs/src-tauri/tests/license_persistence_ui_e2e.rs`:
   - `TestHarness`로 docs 앱 부팅. automation_report 쿼리.
   - debug_id 4개 존재 및 값 검증.
   - `TENCH_UI_AUTOMATION=1` 환경 필수 (release build에서는 이 테스트 skip).
5. `TENCH_TEST_DEVICE_ID` 환경 변수로 device_id 도출 deterministic하게 만들기.
6. `cargo test -p tench-license-store --locked` 통과.
7. `cargo test -p tench-docs --locked license_persistence_ui_e2e` 통과.
8. `cargo check --workspace --locked` 경고 0.
9. ui_automation 게이트 통과: `cargo test --workspace --locked ui_automation`.
