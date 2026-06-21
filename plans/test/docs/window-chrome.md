# Test: window-chrome

## 검증 대상
spec(`plans/spec/docs/window-chrome.md`)의 acceptance criteria → 테스트 함수 매핑.

| Acceptance Criteria | 시나리오 (테스트 함수명) |
|---------------------|---------------------------|
| 우측 상단에 3개 버튼 보인다 | `window_controls_caption_buttons_present` |
| 최소화 클릭 → 최소화 동작 | `window_controls_minimize_submits_minimize_action` |
| 최대화 클릭 → 토글 동작 + value 반영 | `window_controls_maximize_submits_toggle_maximize_action` |
| 닫기 클릭 → 닫기 동작 | `window_controls_close_submits_close_action` |
| 헤더 빈 영역 드래그 → 이동 | `window_controls_empty_menu_bar_submits_start_drag` |
| 메뉴 항목 클릭은 드래그 아님 | `window_controls_menu_item_click_does_not_drag` |
| 캡션 클릭이 메뉴로 오해되지 않음 | `window_controls_caption_click_does_not_open_menu` |
| 테두리 힛존이 올바른 방향 반환 | `window_resize_edge_at` 단위 테스트 (엣지/모서리/캡션 제외) |

## 테스트 파일 위치
`apps/docs/src-tauri/tests/window_controls_ui_e2e.rs` (캡션 버튼 + 드래그)
`crates/tench-ui/src/widgets/window_controls.rs` `#[cfg(test)] mod tests` (resize 힛존 단위 테스트)

> sheets/slides/kodocs는 동일 패턴이므로 docs 테스트로 대표 검증. 각 앱의 헤더
> 높이 차이(`MENU_BAR_H` / `DOC_TAB_H+MENU_H` / `TOOLBAR_H`)는 implement에
> 명시되어 있고, 공유 `window_control_at`/`control_rect`가 단일 진실이므로
> 앱별 중복 테스트는 유지보수 부담만 키움 (필요시 각 앱에 동일 파일 추가 가능).

## Required Test Shape
- **Success**: 캡션 버튼 3개 클릭 → 올바른 `WindowAction` 제출 (drain_actions로 검증).
- **Edge**: 빈 헤더 영역 클릭 → `StartDrag`; 메뉴 항목 클릭 → 드래그 아님 (negative).
- **State**: maximize 노드 `value`가 초기 `"restored"` (헤드리스에선 창이 없으므로).

## 사용할 자동화 노드
implement(`plans/implement/docs/window-chrome.md`)의 자동화 노드 표와 일치.

| debug_id | 검증 시점 | 기대 value/state |
|----------|------------|-------------------|
| `docs.window.minimize` | 클릭 전 | 존재 |
| `docs.window.maximize` | 초기 | `value == "restored"` |
| `docs.window.close` | 클릭 전 | 존재 |
| `docs.menu.active` | 캡션 클릭 후 | `"none"` (메뉴 안 열림) |
| `docs.menu.active` | File 클릭 후 | `"File"` (드래그 대신 메뉴 동작) |

## 행위 검증 패턴
- **패턴 A (Value 변이)**: maximize 노드의 `value`로 복원/최대화 상태 검증.
- **패턴 B (출현/소멸)**: 캡션 클릭 후 `docs.menu.active == "none"`으로 메뉴 미오픈 검증.
- **행위 검증 핵심**: 클릭 후 `harness.drain_actions()`에 올바른 `WindowAction`이
  남았는지 확인. 이것이 "버튼이 의도대로 동작한다"의 증거. (헤드리스 하니스는 실제
  창 조작을 수행하지 않으므로 액션 큐가 의도의 단일 진실.)

## 드래그 영역 테스트 (좌표 사용 불가피)
빈 헤더 공간은 debug_id가 없으므로 좌표 클릭을 사용한다. 좌표는 레이아웃 상수에서
유도: 메뉴 항목 끝(~x=342)과 캡션 시작(`width - WINDOW_CONTROLS_W`) 사이 중간점.
레이아웃이 크게 바뀌지 않는 한 안정. 변경 시 본 테스트 좌표도 갱신.

## 의존
- 선행 implement: `plans/implement/docs/window-chrome.md`.
- 픽스처: 불필요 (`DocsApp::new()` 기본 상태).
- 다이얼로그 주입: 불필요.

## Verification
```bash
cargo test -p tench-docs --test window_controls_ui_e2e
cargo test -p tench-ui --lib window_controls   # resize 힛존 단위 테스트 (lib 바이너리, Windows에서도 동작)
cargo clippy --workspace --locked --all-targets -- -D warnings
cargo run -p tench-architecture-guard -- --strict
```

### CI Notes
- 본 테스트는 Ubuntu CI(`ubuntu-latest`)에서 실행. 헤드리스 wgpu 렌더링 필요.
- **Windows 로컬**: UI e2e 테스트 바이너리가 `STATUS_ENTRYPOINT_NOT_FOUND`(0xC0000139)로
  런타임 실패하는 사전 환경 이슈가 있음 (본 기능과 무관, 기존 테스트 바이너리도 동일).
  따라서 로컬 Windows에서는 테스트 실행 불가, Ubuntu CI로 검증.

## 작업 절차 (실행 에이전트가 매번 따른다)
1. spec / implement 읽기.
2. 자동화 노드 셀렉터(`docs.window.*`)가 `harness.automation_report()`에 노출되는지 확인.
3. 각 시나리오 함수 작성 — `drain_actions()` 기반 행위 검증 사용.
4. `cargo test -p tench-docs --test window_controls_ui_e2e` 통과 (Ubuntu).
5. `cargo clippy --workspace --locked --all-targets -- -D warnings` 통과.
