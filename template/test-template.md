# Test Template

`plans/test/<product>/<id>.md`에 들어갈 문서의 골격 + 테스트 설계 정전 레퍼런스.

**원칙: rot-resistance.** 작성 시점의 코드 스냅샷·라인 번호 박지 마라. **stable한
셀렉터**(`debug_id`)와 **행위 검증** 패턴으로 작성. 작성과 실행 사이 코드가 바뀌어도
테스트 의도는 그대로 유효해야 한다.

전제: hardening plan(`plans/2026-05-06-tench-ui-test-e2e-library-hardening-v1.md`)
적용. 미적용 구간은 free function helper(`click`, `type_text`, `key`)로 치환.

---

## 1. Skeleton (`plans/test/<product>/<id>.md` 골격)

```markdown
# Test: <id>

## 검증 대상
spec(`plans/spec/<product>/<id>.md`)의 acceptance criteria → 테스트 함수 매핑.

| Acceptance Criteria | 시나리오 (테스트 함수명) |
|---------------------|---------------------------|
| AC1: <조건> | `<id>_<scenario>` |
| AC2: <조건> | `<id>_<scenario>` |

## 테스트 파일 위치
`apps/<product>/src-tauri/tests/<id_snake>_ui_e2e.rs`

## Required Test Shape
- **Success**: <한 줄> → 함수: `<id>_<success_name>`
- **Failure/negative** (1–2개): <한 줄씩> → 함수: ...
- **Edge cases** (1–3개): 라운드트립/경계/상태보존 → 함수: ...

## 사용할 자동화 노드
implement(`plans/implement/<product>/<id>.md`)의 자동화 노드 표와 일치.

| debug_id | 검증 시점 | 기대 value/state |
|----------|------------|-------------------|
| `<product>.<area>.<element>` | 트리거 전/후 | `"off"` → `"on"` 등 |

## 의존
- 선행 implement: `plans/implement/<product>/<id>.md`
- 픽스처: <필요 / 불필요>
- 다이얼로그 주입: <필요 / 불필요> (필요 시 §6.2)

## Verification
\`\`\`bash
cargo test -p <crate> <id>
cargo check --workspace --locked
\`\`\`

## 작업 절차 (실행 에이전트가 매번 따른다)
1. spec과 implement를 먼저 읽음.
2. 자동화 노드 셀렉터를 현재 코드에 grep해 노출 확인. 없으면 implement로 회귀.
3. 각 시나리오 함수 작성 — §2 행위 검증 패턴 사용. selector 존재만 검증 금지.
4. `cargo test -p <crate> <id>` 통과.
5. `cargo check --workspace --locked` 통과.
```

---

## 2. 행위 검증 (Behavioral Verification)

> **셀렉터 존재만 검증하는 테스트는 거부.**

### 4가지 패턴

| 패턴 | 검증 대상 | 잡는 회귀 |
|------|-----------|-----------|
| **A. Value 변이** | `value`/`label` before→after | 핸들러 누락, 상태 갱신 누락 |
| **B. 출현/소멸** | selector present/absent | 모달 라이프사이클, 라우팅 |
| **C. 컨텐츠 반영** | 입력 → 다른 영역 `value` 변화 | 입력→상태→렌더 단절 |
| **D. 라운드트립** | 열기→닫기→다시 열기 후 `value` | 상태 누수, 잘못된 초기화 |

### 패턴 A 정전 예시

```rust
let opened = open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.find");
assert_eq!(node_value(&opened, "docs.modal.find_replace.case_sensitive"), "off");

let typed = type_text(&mut harness, "alpha");
assert_eq!(node_value(&typed, "docs.modal.find_replace.match_count"), "3");

let enabled = click(&mut harness, "docs.modal.find_replace.case_sensitive");
assert_eq!(node_value(&enabled, "docs.modal.find_replace.case_sensitive"), "on");
assert_eq!(node_value(&enabled, "docs.modal.find_replace.match_count"), "1");
```

5단계: **셋업 → 진입 → 입력 → 행위 검증 → (보조 검증)**. 보조(`assert_capture_changed`/`png_valid`)는 단독 사용 금지.

### 안티패턴

| 패턴 | 왜 |
|------|----|
| `assert_selector` 만 나열 | 트리에 ID 노출만 검증 |
| `assert_png_valid` 단독 | "패닉 안 났음" 정도 |
| `assert_capture_changed` 단독 | 어떤 변화든 통과 |
| `click_at(x, y)` 좌표 클릭 | 레이아웃 변경에 침묵 실패 |
| 한 함수에 5+ 시나리오 | 실패 위치 추적 불가 |

---

## 3. 자동화 노드 (셀렉터 + role/value)

### ID 명명

```
<product>.<area>.<element>[.<sub>][.<index>]
```

점 구분, 영문 소문자 + 언더스코어, 인덱스 0-base. 영역명 한정 하이픈 (`docs.text-context.copy`).

### role — AccessRole 매핑

`crates/tench-ui/src/core/widget.rs::AccessRole` 변형 이름만 사용. 자유 문자열 금지.

| UI | role |
|----|------|
| 일반 버튼 | `"Button"` |
| 토글 (on/off) | `"Switch"` |
| 체크박스 / 라디오 | `"CheckBox"` / `"RadioButton"` |
| 단일줄 / 멀티줄 입력 | `"TextInput"` / `"MultilineTextInput"` |
| 메뉴 컨테이너 / 항목 | `"Menu"` / `"MenuItem"` |
| 모달 | `"Dialog"` |
| 탭 컨테이너 / 단일 탭 | `"TabList"` / `"Tab"` |
| 슬라이더 / 드래그 핸들 | `"Slider"` |
| 진행 표시기 | `"ProgressBar"` |
| 리스트 / 항목 | `"List"` / `"ListItem"` |
| 정적 텍스트 / 상태 표시 | `"Label"` |
| 이미지 / 썸네일 | `"Image"` |
| 링크 | `"Link"` |
| 테이블 / 행 / 셀 | `"Table"` / `"TableRow"` / `"TableCell"` |
| 의미 없는 컨테이너 | `"GenericContainer"` |

새 의미는 enum 변형 추가 + 표 갱신이 선행. 자유 문자열 금지.

### 필드 사용

| 필드 | 의미 | 규칙 |
|------|------|------|
| `role` | 위젯 종류 — 정적 | AccessRole 변형 이름만 |
| `label` | 사람이 읽는 이름 | locale 가변. sanity check 1회만 |
| `value` | 동적 상태 — 검증 핵심 | **locale-stable**: `"on"`/`"off"`/숫자/enum key. 번역어 금지 |
| `enabled` | 활성화 여부 | disabled 검증의 근거 |

**클릭/입력으로 바뀌는 상태는 반드시 `value`에 노출.** value 비면 행위 검증 불가.

### bounds + 트리

- bounds는 paint/event/automation 단일 진실 함수(hardening 3)에서. 손으로 복제 금지.
- 자식 노드는 부모 안에 (모달 → 컨트롤). 평탄 트리는 오버레이 차단 검출(hardening 4.1)을 깨뜨림.

---

## 4. UI 종류별 시나리오

각 영역에 최소 3개(success/negative/edge). 4–6개 일반.

| UI | role | 시나리오 셋 |
|----|------|-------------|
| Toggle | `Switch` | value off→on; 빈 입력에서 부수효과 없음; 두 번 클릭 라운드트립 |
| Modal | `Dialog` | open/internal/OK/cancel/round-trip; **키보드 격리** negative 필수 |
| 메뉴 항목 | `MenuItem` | 클릭 → 결과; disabled 무동작; 단축키 동치 |
| 입력 필드 | `TextInput` | 입력→value+부수효과; 빈/잘못된 입력; 라운드트립 |
| 키보드 단축키 | (대상 role) | modifier 정확; modifier 누락; 모달 라우팅 차단 |
| Automatic (시간 기반) | `Label` 등 | `AnimFrame` → 상태 변화; 억제 조건; 입력 리셋 |
| 컨텍스트 메뉴 | `MenuItem` | 우클릭→항목; disabled; ESC dismiss |
| 드래그 핸들 | `Slider` | from→to value 변화; 영역 외 클램프; 라운드트립 |
| 드롭 영역 | `GenericContainer` | 유효 경로 (§6.2); 잘못된 확장자; 다중 파일 |
| Disabled state | (대상 role) | `enabled == false` 노출; 클릭 무동작; 조건 충족 시 enabled |

### 모달 키보드 격리 (거의 모든 모달의 negative)

```rust
click(&mut harness, "docs.document");
let before = type_text(&mut harness, "Body");
let body_before = node_value(&before, "docs.document.text");

open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.find");
type_text(&mut harness, "Query");

let body_after = node_value(&capture(&mut harness), "docs.document.text");
assert_eq!(body_before, body_after);
```

---

## 5. 코드 헬퍼 (canonical)

```rust
use tench_<product>_lib::ui::<App>;
use tench_ui_automation_core::{
    find_node, UiAutomationCapture, UiAutomationKey, UiAutomationModifiers, UiAutomationSelector,
};
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::{assert_capture_changed, TestHarness};

fn make_harness() -> TestHarness {
    TestHarness::with_config(<App>::new(), HarnessConfig::with_viewport(1280.0, 820.0))
}

fn selector(id: &str) -> UiAutomationSelector { UiAutomationSelector::debug_id(id) }

fn node_value(cap: &UiAutomationCapture, id: &str) -> String {
    find_node(cap.ui_tree.as_ref().expect("ui tree"), &selector(id))
        .and_then(|n| n.value.as_deref().or(n.label.as_deref()))
        .expect("node value").to_string()
}

fn node_enabled(cap: &UiAutomationCapture, id: &str) -> bool {
    find_node(cap.ui_tree.as_ref().expect("ui tree"), &selector(id))
        .expect("node").enabled
}
```

데스크톱: `1280×820`. 모바일: `HarnessConfig::mobile()` (390×844).

### Component API (hardening 적용 후)

```rust
let cs = harness.component("docs.modal.find_replace.case_sensitive");
cs.assert_value("off");
cs.click().assert_value("on");

harness.component("docs.toolbar.zoom_in")
    .click_with(UiAutomationModifiers { control: true, ..Default::default() });
```

`Component::click()`은 오버레이 차단 시 `ClickBlocked` 패닉.

---

## 6. 특수 케이스

### 6.1 i18n / 라벨

i18n 미구현 시기 정책:
- `value`로 검증, label 어설션은 sanity check 1회만.
- 라벨에 토글 상태 박지 마라 (`"Case sensitive (on)"`). 상태는 항상 `value`로.
- `value`는 locale-stable: `"on"`/`"off"`/숫자/enum key. 번역어 금지.
- i18n 도입 후: `HarnessConfig::with_locale(Locale::EN)` 고정 + label 어설션을 i18n 키로 마이그레이션.

### 6.2 Native dialog

`apps/<product>/src-tauri/src/lib.rs::DialogResult` + `set_dialog_receiver`로 결과 주입.

```rust
fn make_harness_with_dialog() -> (TestHarness, mpsc::Sender<DialogResult>) {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut app = DocsApp::new();
    app.set_dialog_receiver(rx);
    (TestHarness::with_config(app, HarnessConfig::with_viewport(1280.0, 820.0)), tx)
}

let (mut harness, dialog_tx) = make_harness_with_dialog();
open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.open");
dialog_tx.send(DialogResult::OpenFile("/tmp/x.docx".into())).unwrap();
harness.dispatch_window(WindowEvent::AnimFrame(0));     // 폴링 주기 진행
```

`app_handle: None`이라 native dialog 자체는 안 뜨지만, 결과 처리 경로는 검증 가능. cancel/잘못된 경로 negative 필수.

### 6.3 다중 탭 / 세션

자동화 노드 컨벤션:
```
docs.tabs (TabList)
├── docs.tab.<idx> (Tab, value="<title>")
│   ├── docs.tab.<idx>.dirty (Label, value="true"/"false")
│   └── docs.tab.<idx>.close (Button)
└── docs.tab.new (Button)
docs.active_tab (Label, value="<idx>")
docs.document.* (활성 탭 반영)
docs.tab.<idx>.text (비활성 탭 본문)
```

검증 핵심: 비활성 탭 dirty 격리, 탭 전환 시 `docs.document.text` 스왑, dirty 닫기 확인 다이얼로그.

### 6.4 모바일 / 반응형

`HarnessConfig::mobile()` (390×844). 모바일 시나리오는 `mobile_` prefix 함수. 같은 의미 컨트롤은 같은 ID, 모바일 전용만 `docs.hamburger.*` / `docs.mobile.*`.

### 6.5 픽스처

- 짧은 텍스트는 `type_text` 충분.
- 긴/복잡: 엔진 API 직접 호출 또는 `app.load_from_bytes(include_bytes!("fixtures/x.docx"))`.
- 위치: `apps/<product>/src-tauri/tests/fixtures/`. 공유는 `crates/tench-ui-test/fixtures/`.

### 6.6 스냅샷 baseline (드물게)

행위 검증으로 안 잡히는 시각 회귀에만: 색상 팔레트, 차트, 복잡한 레이아웃, 테마 전환.

```rust
capture.assert_matches_baseline("docs/find_modal_default");      // hardening 5.5
```

갱신: `TENCH_UI_UPDATE_SNAPSHOTS=1 cargo test ...`. 일반 동작에는 사용 금지 (maintenance burden).

---

## 7. 흔한 함정

| 증상 | 원인 / 처방 |
|------|-------------|
| 셀렉터 못 찾음 | `automation_children` 미구현 → `mod.rs`에 노드 push |
| 클릭이 다른 곳에 감 | bounds 드리프트 → layout 단일 진실 (hardening 3) |
| modifier 안 붙음 | `PointerButtonEvent`엔 modifier 없음 → `Click { modifiers }` (hardening 1) |
| 통과인데 화면 변화 없음 | `ctx.request_paint()` 누락 |
| `value`가 None | 동적 상태에 `value: Some(...)` 누락 |
| 같은 ID 중복 | row index를 ID에 포함 (`docs.comments.row.0`) |
| 폰트 다름 | 시스템 폰트 사용 → tench-ui 임베디드만 |
| dialog 응답 없음 | `set_dialog_receiver`로 결과 주입 (§6.2) |
| 라벨 어설션 i18n에 깨짐 | `value`로 검증 (§6.1) |
| modal 자식 클릭 안 됨 | 자동화 트리 평탄 → 부모/자식으로 push |
| 테스트 간 충돌 | 글로벌 자원 → 인스턴스 채널 / `tempfile::tempdir()` |
| 비활성 탭 dirty | `events.rs`의 `active_session_idx` 라우팅 (§6.3) |

---

## 8. 병렬 / CI 환경

| 자원 | 격리 | 처방 |
|------|------|------|
| 픽셀 캡처 / 엔진 인스턴스 | ✓ | 별도 wgpu, 독립 `App::new()` |
| `DIALOG_TX: OnceLock` | ✗ | `set_dialog_receiver(rx)` 인스턴스 채널 |
| 파일 시스템 | ✗ | `tempfile::tempdir()` |
| 환경변수 / 글로벌 플래그 | ✗ | 테스트에서 `set_var` 금지 |
| 폰트 (CI 차이) | — | tench-ui 임베디드만, 시스템 fallback 금지 |
| 시간 (CI 차이) | — | `dispatch_window(AnimFrame(ms))`만, wall clock 금지 |

플랫폼 의존 있는 테스트는 `plans/test/<product>/<id>.md`에 `### CI Notes` 절 추가.

---

## 9. 테스트 함수 명명

`<id>_<scenario>` 형식. snake_case, 50자 이내, 동사형.

예:
- `case_sensitive_toggle_recomputes_mixed_case_matches`
- `find_modal_typing_is_isolated_from_document_until_escape`
- `mobile_hamburger_menu_opens_navigation_drawer`

`_test`/`test_`/`it_` 접두/접미 금지. 같은 `<id>` prefix 공유 → `cargo test -p <crate> <id>`로 한 번에 실행.

---

## 10. 작업 체크리스트

### 작성 시 (`plans/test/<product>/<id>.md`)
- [ ] spec(`plans/spec/<product>/<id>.md`)의 모든 acceptance criteria가 시나리오와 매핑됨
- [ ] success / negative / edge 셋 모두 포함
- [ ] 사용할 자동화 노드가 implement(`plans/implement/<product>/<id>.md`) 표와 일치
- [ ] 모달 spec이면 키보드 격리 negative 포함
- [ ] disabled 의미 있으면 `enabled` 검증 포함
- [ ] 함수명 `<id>_<scenario>` 패턴, 50자 이내

### 실행 시 (테스트 코드 작성)
- [ ] 행위 검증 패턴 A–D 중 하나 이상 사용
- [ ] selector 존재만 검증 안 함
- [ ] `debug_id` 셀렉터 기반, 좌표 클릭 없음
- [ ] `value` 비교 (locale-stable 값으로)
- [ ] `cargo test -p <crate> <id>` 통과
- [ ] `cargo check --workspace --locked` 통과

---

## 11. 참고

- E2E 라이브러리 보강: `plans/2026-05-06-tench-ui-test-e2e-library-hardening-v1.md`
- AccessRole 정의: `crates/tench-ui/src/core/widget.rs::AccessRole`
- 자동화 액션: `crates/ui-automation-core/src/lib.rs::UiAutomationAction`
- 헤드리스 하니스: `crates/tench-ui-test/src/harness.rs`
- 헬퍼: `crates/tench-ui-test/src/test_helpers.rs`
- 워크플로우: `AGENTS.md` "Testing", "UI Automation & Headless Visual Capture"
