# Implement: window-chrome

> 작성 시점과 실행 시점 사이 코드 변경 가능성. 위치는 항상 grep으로 재확인 후 변경.

## 목표
- spec(`plans/spec/docs/window-chrome.md`)의 캡션 버튼 + 헤더 드래그 동작 구현.
- design(`plans/design/docs/window-chrome.md`)의 Component breakdown / Visual properties / States를 코드로 옮김.
- 공유 크레이트 `tench-ui`에 기반을 두고 4개 앱에 동일 패턴 적용.

## 영향 받는 영역

| 영역 | 무엇이 바뀌나 | 찾기 전략 |
|------|----------------|-----------|
| `crates/tench-ui/src/widgets/window_controls.rs` (신규) | `paint_window_controls` / `window_control_at` / `control_rect` + `WINDOW_CONTROLS_W` 상수; `window_resize_edge_at` / `WindowResizeEdge` (크기 조절 힛테스트) | `pub fn paint_window_controls`, `pub fn window_resize_edge_at` |
| `crates/tench-ui/src/core/events.rs` | `WindowAction` enum 추가 (`Minimize`/`ToggleMaximize`/`Close`/`StartDrag`) | `pub enum WindowAction` |
| `crates/tench-ui/src/core/widget.rs` | `GlobalState.window_maximized` 필드 + `EventCtx::submit_window_action` | `pub struct GlobalState`, `impl EventCtx` |
| `crates/tench-ui/src/widgets/window_controls.rs` (신규) | `paint_window_controls` / `window_control_at` / `control_rect` + `WINDOW_CONTROLS_W` 상수 | `pub fn paint_window_controls` |
| `crates/tench-ui/src/platform/native.rs` | `NativeBackend`가 `Weak<Window>` 보유, `drain_window_actions()`에서 실행, `request_close` 플래그로 `NativeApp`이 `event_loop.exit()`; 테두리 resize는 `handle_winit_window_event`에서 `CursorMoved`(커서 전환)/`MouseInput`(drag_resize_window, 위젯 우회) 처리 | `fn drain_window_actions`, `fn resize_direction_for`, `fn resize_cursor_for` |
| `crates/tench-ui/src/lib.rs` | `WindowAction`, `WindowControl`, `WINDOW_CONTROLS_W`, `paint_window_controls`, `window_control_at` re-export + prelude | `pub use widgets::` |
| 각 앱 `<app>/src-tauri/src/ui/state` | `window_maximized: bool`, `window_control_hovered: Option<WindowControl>` 필드 + 초기화 | `pub struct <App>State`, `<App>State::new` |
| 각 앱 메뉴 바/툴바 페인트 | 우측 콘텐츠 `WINDOW_CONTROLS_W`만큼 좌측 이동 + `paint_window_controls` 호출 | `fn paint_menu_bar` / `fn paint_toolbar` |
| 각 앱 `<app>/src-tauri/src/ui/widget.rs::paint` | `self.state.window_maximized = ctx.global.window_maximized` 동기화 | `fn paint` |
| 각 앱 포인터 Down 핸들러 | 캡션 버튼 우선 힛테스트 → 액션 제출; 메뉴/버튼 외 빈 공간 → `StartDrag` | `fn handle_pointer_down` / `PointerEvent::Down` |
| 각 앱 포인터 Move 핸들러 | `window_control_hovered` 갱신 (hover 피드백) | `fn handle_pointer_move` / `PointerEvent::Move` |
| 각 앱 `<app>/src-tauri/src/ui/automation.rs` | `window_control_at`/`control_rect` 기반 캡션 노드 3개 push, 기존 우측 콘텐츠 rect 이동 | `fn <app>_automation_nodes` |
| 각 앱 `tauri.conf.json` | `windows[0].decorations = false` | `"decorations"` |
| 각 앱 `<app>/src-tauri/src/main.rs` | `NativeConfig { decorations: false, .. }` | `NativeConfig` |

## 필요한 변경 (의도 단위)

### 1. WindowAction + 전역 최대화 상태
- **입력**: 위젯이 캡션 클릭/빈 헤더 드래그를 감지.
- **처리**: `EventCtx::submit_window_action(WindowAction)` → `GlobalState.pending_actions`에 push.
- **출력**: 네이티브 백엔드가 drain하여 실행; 헤드리스 테스트는 큐에 남겨 검증.

### 2. 캡션 버튼 페인트/히트테스트 헬퍼 (공유)
- `control_rect(window_width, menu_bar_h, control)`: 우측 끝 기준 버튼 rect.
- `window_control_at(x, y, window_width, menu_bar_h)`: 힛테스트 → `Option<WindowControl>`.
- `paint_window_controls(...)`: 3버튼 페인트 (hover/maximize 상태 반영).

### 3. 네이티브 백엔드 액션 실행
- **입력**: 매 이벤트 펌프 후 `GlobalState.pending_actions`.
- **처리**: `WindowAction`만 추출해 실행. `Minimize`→`set_minimized(true)`, `ToggleMaximize`→`is_maximized` 토글, `StartDrag`→`drag_window()`, `Close`→`request_close=true`. 비-WindowAction은 큐 보존.
- **출력**: `global.window_maximized` 매 render마다 `window.is_maximized()`로 동기화.
- **Close**: winit 0.30에 `Window::close()` 없음 → `NativeApp.window_event`가 `backend.take_close_request()` 시 `event_loop.exit()`. 외부 `CloseRequested`도 동일.

### 4. 각 앱 헤더 통합
- 페인트: 메뉴 바/툴바 끝에 `paint_window_controls` 호출; 기존 우측 콘텐츠는 `width - WINDOW_CONTROLS_W` 기준으로 좌측 이동.
- 포인터 Down: 헤더 y 범위 내에서 (a) 캡션 힛테스트 우선, (b) 메뉴/툴바 항목, (c) 나머지 빈 공간 → `StartDrag`.
- 포인터 Move: `window_control_at`으로 hover 갱신, 변경 시 `request_paint`.

### 5. 자동화 노드
- 각 앱 `<app>_automation_nodes`에 3개 버튼 노드 push. maximize 노드는 `value: "restored"/"maximized"`. rect는 `control_rect`로 단일 진실.

### 6. 테두리 크기 조절 (네이티브 백엔드)
- **입력**: winit `CursorMoved` / `MouseInput` (물리 좌표).
- **처리**: `window_resize_edge_at(x, y, w, h)`로 가장자리 방향 계산. `CursorMoved` 시 edge↔내부 전환에서만 커서 set (`resize_cursor_for` → `NsResize`/`EwResize`/`NwseResize`/`NeswResize`/`Default`). `MouseInput` Pressed 시 edge면 `window.drag_resize_window(resize_direction_for(edge))` 후 위젯 전달 생략(`return false`).
- **출력**: OS가 resize loop 수행 → `Resized` 이벤트 → 재페인트. 헤드리스 하니스는 `handle_winit_window_event`를 안 거치므로 간섭 없음.
- **캡션 제외**: `window_resize_edge_at` 내부에서 `x > width - WINDOW_CONTROLS_W && y < 80` 영역은 `None` (캡션 버튼 우선).

## 새 자동화 노드

design의 "Automatio 노드" 표와 동일:

| debug_id | role | value | 노출 조건 |
|----------|------|-------|-----------|
| `<product>.window.minimize` | `Button` | — | 항상 |
| `<product>.window.maximize` | `Button` | `"restored"`/`"maximized"` | 항상 |
| `<product>.window.close` | `Button` | — | 항상 |

`<product>` = `docs` / `kodocs` / `sheets` / `slides`.

## 의존
- 선행: 없음 (신규 기능).
- 영향: 각 앱 헤더 레이아웃/자동화에 직접 결합. 헤더 구조 변경 시 본 implement 갱신.
- 라인 예산: 공유 크레이트 신규 파일 `window_controls.rs`는 300줄 이내. 기존 파일(`automation.rs`/`pointer.rs`/`widget.rs`/`native.rs` 등)은 baseline 갱신으로 성장 기록.

## 작업 절차 (실행 에이전트가 매번 따른다)
1. spec / design 읽기.
2. "영향 받는 영역" 표의 각 항목을 grep으로 위치 확정.
3. "필요한 변경" 의도대로 코드 변경.
4. `cargo fmt --all && cargo clippy --workspace --locked --all-targets -- -D warnings` 통과.
5. `cargo run -p tench-architecture-guard -- --strict` 통과 (baseline 갱신 포함).
6. `plans/test/docs/window-chrome.md`의 테스트 실행 (Ubuntu CI에서).
