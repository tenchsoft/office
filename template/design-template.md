# Design Template

`plans/design/<product>/<id>.md`에 들어갈 문서의 골격.

**원칙: spec(behavior)과 implement(code) 사이의 시각적 계약.** 사용자가 보는 *모양과
느낌*을 정의. 코드 라인 번호 박지 말 것 — 대신 `Theme` 토큰 이름과 컴포넌트 트리로
표현해, 토큰/픽셀 값이 바뀌어도 디자인 의도는 유효하게.

`<product>`는 `apps/<product>/`. `<id>`는 spec/design/implement/test 네 폴더에서 동일.
간단한 기능(기존 컴포넌트 재사용 only)은 design 1줄로 끝낼 수 있음 — 무조건 길게 쓰지 마라.

---

## 골격

```markdown
# Design: <id>

## 한 줄 정의
spec(`plans/spec/<product>/<id>.md`)이 정의한 동작을 **시각적으로** 어떻게 표현하나.

## 시각적 레이아웃
ASCII art 또는 component tree로 공간 관계 표현. 정확한 픽셀 대신 **상대 비율**.

\`\`\`
┌─ Modal (centered, 400×200) ────────────────┐
│  Header: "Find"                       [×]  │
│  ─────────────────────────────────────     │
│  [ Query input            ] [ Aa ] [ .* ]  │
│  matches: 3                                │
│  [ Previous ] [ Next ]                     │
└────────────────────────────────────────────┘
\`\`\`

## Component breakdown
implement의 자동화 노드 트리와 1:1 일치해야.

| Component | role | 자식 (debug_id) |
|-----------|------|-----------------|
| Modal container | `Dialog` | `<product>.modal.<name>` |
| Header | `Label` | `<product>.modal.<name>.title` |
| Close button | `Button` | `<product>.modal.<name>.close` |
| Query input | `TextInput` | `<product>.modal.<name>.query` |
| Case toggle | `Switch` | `<product>.modal.<name>.case_sensitive` |
| ... | | |

## Visual properties
Theme 토큰 (`crates/tench-ui/src/theme/mod.rs::Theme`)을 우선 참조. 토큰에 없는 값은
constant로 명시 + 가능하면 토큰 추가 제안.

| 속성 | 값 |
|------|----|
| Modal background | `theme.surface` |
| Modal border | `theme.border`, `border_radius = theme.border_radius` |
| Modal shadow | `theme.surface` 위 16px blur, 흑색 24% (새 토큰 `shadow_modal` 제안) |
| Title typography | `theme.font_size_large`, weight 600 |
| Body typography | `theme.font_size`, weight 400 |
| Outer padding | `theme.spacing_large` (16px) |
| Inner gap | `theme.spacing` (8px) |
| Input height | `theme.input_height` (32px) |
| Button height | `theme.button_height` (36px) |

## States
모든 인터랙티브 컴포넌트마다.

| Component | Default | Hover | Active/Pressed | Focus | Disabled |
|-----------|---------|-------|----------------|-------|----------|
| Close button | `theme.surface` | `theme.surface` lighten 8% | lighten 16% | 2px outline `theme.primary` | opacity 0.5 |
| Aa toggle | bg `theme.surface`, fg `theme.on_surface` | bg lighten 8% | — | 2px outline | opacity 0.5 |
| Aa toggle (on) | bg `theme.primary`, fg `theme.on_primary` | bg `theme.primary` darken 8% | — | 2px outline | opacity 0.5 |

## Animations / transitions
필요한 경우만. 시각적 정적 컴포넌트는 생략.

| Trigger | Property | Duration | Easing |
|---------|----------|----------|--------|
| Modal open | opacity 0→1 + translateY 8px→0 | 150ms | ease-out |
| Modal close | 역방향 | 100ms | ease-in |
| Aa toggle | bg color | 80ms | linear |

## Responsive 변형
viewport별 차이가 있으면.

- **Desktop (1280×820 기준)**: 위 명세 그대로.
- **Mobile (390×844)**: Modal full-width minus 16px margin, height auto.
  Buttons stack vertically below `< 360px`.

## Accessibility
- Focus indicator는 모든 인터랙티브 컴포넌트에 명시적으로 (위 States 표).
- Color contrast: `on_*` 텍스트가 배경 대비 WCAG AA (4.5:1) 이상.
- Disabled 상태도 텍스트가 식별 가능 (3:1 이상).
- 키보드만으로 모든 동작 가능 (탭 순서, Enter/Escape 처리).

## Design tokens — 사용 / 제안
- **사용**: `theme.surface`, `theme.border`, `theme.primary`, `theme.on_primary`,
  `theme.font_size`, `theme.font_size_large`, `theme.spacing`, `theme.spacing_large`,
  `theme.border_radius`, `theme.input_height`, `theme.button_height`.
- **신규 제안 (있으면)**:
  - `shadow_modal` — modal 그림자 색/blur. 다수 modal에서 재사용 예상.
  - `transition_fast` — 80ms linear (토글 등 즉시성 강조).

신규 토큰은 `Theme` 구조체에 추가하는 implement 항목으로 등록.

## Out of scope / open questions
- 배제 사항 또는 디자이너 결정 대기 항목.
- ?
```

---

## 작성 규칙

- **길이는 기능 복잡도에 비례.** 기존 컴포넌트만 재사용하면 1–2줄로 끝. 새 시각 요소가
  있으면 위 골격 채움. 강제로 모든 섹션 채우지 말 것.
- **픽셀 하드코딩 금지** — `theme.*` 토큰 우선. 토큰에 없으면 `Theme`에 추가 제안.
- **컴포넌트 트리는 implement의 debug_id 트리와 1:1**. 두 곳이 어긋나면 design을 정전으로
  보고 implement를 맞춤.
- **상태(States) 누락이 가장 흔한 회귀.** focus / disabled 명세 빠지면 접근성 깨짐.
- **animation은 선택적**. 정적 UI는 생략. 단 toast 만료, modal open/close 등 시간 기반
  동작이 spec에 있으면 design에서 timing 명시.

---

## 워크플로우 안에서

| Phase | 답하는 질문 |
|-------|-------------|
| spec | 사용자가 무엇을 하고 무엇을 보는가 (behavior) |
| **design** | **그것이 어떻게 보이고 느껴지는가 (visual + interaction)** |
| background | (해당 시) 백그라운드에서 어떻게 도는가 (runtime contract) |
| implement | 그것이 어떻게 구현되는가 (code recipe) |
| test | 그것이 동작함을 어떻게 증명하는가 (behavioral verification) |

design은 spec 없이 작성 못함. background와는 sibling — UI 표면이 있는 기능은 design,
백그라운드 동작이 있는 기능은 `template/background-template.md`. 둘 다 있는 hybrid는 둘 다.
implement는 design + background 둘 다 입력으로 받는다. test는 둘을 직접 참조하지 않지만
스냅샷 baseline (test-template §6.6)이 있을 때는 design이 baseline 의도의 출처.

---

## 예시 (id: file-menu-open)

```markdown
# Design: file-menu-open

## 한 줄 정의
File → Open은 기존 메뉴 시스템의 항목으로, native 파일 다이얼로그를 호출. 새 시각 요소
없음. 결과 토스트 / 새 탭만 기존 컴포넌트 재사용.

## 시각적 레이아웃
신규 시각 요소 없음. 메뉴 항목 / 토스트 / 탭은 모두 기존 컴포넌트.

## Component breakdown
| Component | role | debug_id |
|-----------|------|----------|
| Open menu item | `MenuItem` | `docs.menu.file.open` |
| Success toast | `Label` | `docs.toast.0` |
| New tab | `Tab` | `docs.tab.<idx>` |

모두 기존 디자인 사용. 별도 visual properties / states 명세 불필요.

## Out of scope
- 토스트 자체의 디자인 (별도 design `toast-system`).
- 메뉴 항목 아이콘 (현재 텍스트 only, 후속 design).
```

길이 ~10줄. 신규 시각 요소가 없으면 design은 이 정도로 충분.
