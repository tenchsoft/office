# Implement Template

`plans/implement/<product>/<id>.md`에 들어갈 문서의 골격.

**원칙: 작성 시점의 코드 스냅샷을 박지 마라.** 작성과 실행 사이에 코드가 바뀔 수 있다.
대신 **stable한 식별자**(모듈/함수/타입 이름, `debug_id`, AccessRole 변형)와
**찾기 전략**(grep 패턴, 키워드)으로 위치를 가리킨다. 실행 에이전트는 매번 현재 코드를
직접 읽어 위치를 확정한 후 변경한다.

implement 문서를 보면 다른 에이전트가 "어디를 어떻게 바꿔야 하는가"를 무리 없이
판단할 수 있어야 한다.

---

## 골격

```markdown
# Implement: <id>

> 작성 시점과 실행 시점 사이 코드 변경 가능성. 위치는 항상 grep으로 재확인 후 변경.

## 목표
- spec(`plans/spec/<product>/<id>.md`)의 핵심 동작 한 줄.
- design(`plans/design/<product>/<id>.md`)이 있으면 Component breakdown / Visual properties / States를 코드로 옮긴다.
- background(`plans/background/<product>/<id>.md`)가 있으면 Trigger / Lifecycle / Observability 표를 코드로 옮긴다.
- 하나는 반드시 있어야 함 (UI / background / 둘 다).

## 영향 받는 영역
모듈/함수 단위. **라인 번호 금지**, **현재 코드 스니펫 인용 금지**.

| 영역 | 무엇이 바뀌나 | 찾기 전략 |
|------|----------------|-----------|
| `apps/<product>/src-tauri/src/ui/<module>.rs::<함수명 또는 책임>` | <한 줄 변경 의도> | `grep -n '<keyword>'` 또는 `fn <name>(` |
| `apps/<product>/src-tauri/src/ui/mod.rs` (자동화 노드 emit) | 새 노드 N개 추가 | `fn <product>_automation_nodes` |
| `crates/<crate>/src/<module>.rs` (엔진 변경) | <한 줄 변경 의도> | `pub fn <name>` |

## 필요한 변경 (의도 단위)
각 변경은 **무엇을 어떻게 동작시켜야 하는가**의 서술. 실행 코드를 박지 않는다.

### 1. <영역 이름>
- **입력**: 어떤 이벤트/상태가 들어오는가
- **처리**: 어떤 분기/계산을 수행하는가
- **출력/사이드 이펙트**: state 변화, repaint 요청, 다른 모듈 호출 등
- **순서/우선순위**: 다른 분기보다 먼저/나중인지 (예: "context menu 분기보다 먼저")

### 2. <영역 이름>
...

## 새 자동화 노드
**design의 "Component breakdown" 표 + background의 "Observability" 표와 1:1 일치해야
한다.** 두 문서가 `role` / `debug_id`를 이미 정의했으면 그대로 옮긴다. 둘 다 없는
노드를 implement에서 임의 추가 금지 — design 또는 background를 먼저 갱신할 것.

| debug_id | role | value | 노출 조건 |
|----------|------|-------|-----------|
| `<product>.<area>.<element>` | `Button` / `Switch` / `MenuItem` / ... | `"on"`/`"off"` 등 locale-stable | 어떤 state일 때 보이나 |

`role`은 `crates/tench-ui/src/core/widget.rs::AccessRole` 변형 이름만 사용
(`template/test-template.md` §3 매핑 표 참조).

## 의존
- 선행 implement: `plans/implement/<product>/<other-id>.md`
- 영향 받는 implement: `plans/implement/<product>/<other-id>.md` (이 변경이 그쪽도 깨뜨릴 수 있으면 명시)
- 라이브러리 보강 의존: hardening plan §X (있으면)

## 작업 절차 (실행 에이전트가 매번 따른다)
1. spec / design / background 중 존재하는 것을 모두 먼저 읽음. 컴포넌트 트리 + 시각 속성은
   design, runtime 동작 + 옵저버빌리티는 background, 사용자 동작 계약은 spec.
2. "영향 받는 영역" 표의 각 항목에 대해 **현재 코드를 먼저 읽고** grep으로 위치 확정.
3. 위치가 표와 다르면 (리네임/이동 발생) 표를 갱신 (PR에 포함).
4. "필요한 변경"의 의도대로 코드 변경. design의 Visual properties는 `theme.*` 토큰 그대로,
   background의 timing/한계 수치도 명세 그대로.
5. 새 자동화 노드는 design + background 표를 합집합으로 추가. role은 AccessRole 변형 이름.
6. `cargo check --workspace --locked` 통과 확인.
7. 새 노드가 `harness.automation_report()`에 노출되는지 한 번 확인.
8. `plans/test/<product>/<id>.md`가 있으면 거기 명시된 테스트 실행하여 통과 확인.
```

---

## 작성 규칙 (rot-resistance)

| 박으면 안 되는 것 | 대신 적는 것 |
|--------------------|---------------|
| `events.rs:1652-1710` | `events.rs::handle_pointer_down` 또는 `grep -n 'PointerEvent::Down'` |
| 현재 코드 스니펫 | 변경의 **의도**와 **계약** (입력/처리/출력) |
| 특정 라이브러리 버전에 의존하는 호출 시그니처 | 트레이트/타입 이름 (시그니처는 매번 확인) |
| state 필드의 정확한 이름이 자주 바뀐다면 그 이름 자체 | "<역할 설명>을 담는 필드" (이름은 grep) |

| 박아도 되는 것 (충분히 stable) |
|--------------------------------|
| 모듈 경로 (`apps/<product>/src-tauri/src/ui/`) |
| public 함수 이름 (`paint_sidebar`, `handle_pointer_down` 등 도메인 동사) |
| 타입/트레이트 이름 (`DocsApp`, `Widget`, `EventCtx`) |
| `debug_id` 문자열 (자동화 계약) |
| AccessRole 변형 이름 |
| crate 이름 (`tench-document-core`, `tench-ui`) |

## 예시 (id: file-menu-open) — 발췌

```markdown
# Implement: file-menu-open

## 영향 받는 영역
| 영역 | 무엇이 바뀌나 | 찾기 전략 |
|------|----------------|-----------|
| `apps/docs/src-tauri/src/ui/dispatch.rs` (메뉴 라우팅) | File → Open 분기 추가 | `fn handle_menu_item` |
| `apps/docs/src-tauri/src/ui/mod.rs` (다이얼로그 트리거) | OpenFile dialog 시작 | `dialog_sender` 검색 |
| `apps/docs/src-tauri/src/ui/mod.rs` (다이얼로그 결과 폴링) | OpenFile 결과 처리: parse → 새 session push → activate | `DialogResult::OpenFile` 검색 |
| `apps/docs/src-tauri/src/ui/mod.rs` (자동화 노드) | tab/toast/recent_files 노드 추가 | `fn docs_automation_nodes` |

## 필요한 변경
### 1. 메뉴 라우팅에 Open 분기
- 입력: `docs.menu.file.open` 클릭
- 처리: 메뉴 닫고 native 파일 다이얼로그 시작 (.docx 필터)
- 출력: `dialog_sender`로 dialog 등록, 메뉴 state 클리어

### 2. 다이얼로그 결과 처리
- 입력: `DialogResult::OpenFile(path)` 수신
- 처리: `tench_office_io::docs::format::parse_from_path(path)` 호출 →
  성공 시 새 `DocumentSession` 생성, sessions에 push, active idx 갱신.
  실패 시 에러 토스트.
- 출력: 새 탭 활성, 본문 repaint, 토스트 표시
```
