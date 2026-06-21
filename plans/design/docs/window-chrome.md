# Design: window-chrome

> 적용 범위: 제품 전체. 본 문서는 `docs` 기준이며, sheets/slides/kodocs는 헤더
> 영역의 높이/구조만 다를 뿐 동일한 시각/상태 계약을 따른다.

## Component breakdown

| 요소 | role | 설명 |
|------|------|------|
| 최소화 버튼 | `Button` | 46×헤더높이, 하단 가로선 아이콘 |
| 최대화/복원 버튼 | `Button` | 단일 사각형(복원 상태) / 겹친 두 사각형(최대화 상태). `value`: `"restored"`/`"maximized"` |
| 닫기 버튼 | `Button` | X 아이콘. hover 시 빨강 배경(`#E81123`) + 흰색 아이콘 |
| 헤더 빈 영역 | (드래그 핸들) | 별도 노드 없음. 메뉴 항목/버튼 외 영역에서 `WindowAction::StartDrag` |
| 창 테두리 가장자리 | (보이지 않는 핸들) | 상/하/좌/우 6px 폭, 모서리는 6×6px. 커서 모양으로 방향 표시 |

## Visual properties

- 배치: 창 우측 끝에서 좌→우로 [닫기][최대화][최소화]. 각 46px, 총 138px (`WINDOW_CONTROLS_W`).
- 높이: 각 앱의 헤더 높이와 동일 (docs/kodocs: `MENU_BAR_H`, sheets: `DOC_TAB_H + MENU_H`, slides: `TOOLBAR_H`).
- 아이콘 색: `rgba8(0xE4,0xE4,0xE4,0xFF)` (어두운 메뉴 바 위 대비).
- hover 배경 (최소화/최대화): `rgba8(0x2A,0x2A,0x2A,0xFF)`.
- hover 배경 (닫기): `rgba8(0xE8,0x11,0x23,0xFF)` (Windows caption 표준 빨강).
- 기존 헤더 우측 콘텐츠(제목, 저장상태 pill, 통합문서명 등)는 `width - WINDOW_CONTROLS_W` 좌측으로 이동.

## 크기 조절 (Resize)

- 힛존: 각 가장자리 6px (`WINDOW_RESIZE_EDGE`). 모서리는 6×6px에서 대각선 방향.
- 커서 매핑: 상/하 → `NsResize`, 좌/우 → `EwResize`, 좌상·우하 → `NwseResize`, 우상·좌하 → `NeswResize`. 내부 → `Default`.
- 캡션 버튼 영역(우상단 `WINDOW_CONTROLS_W × CAPTION_ZONE_H(80px)`)은 크기 조절에서 **제외** — 캡션 클릭이 우선. `CAPTION_ZONE_H`는 제품별 헤더 높이 최대값(sheets ≈56px) + 여유.
- 상호작용: 네이티브 백엔드가 가장자리에서 `drag_resize_window(direction)` 호출 (OS가 resize loop 처리). 위젯 트리로 이벤트 전달 안 함.

## States

| 상태 | 최소화 | 최대화 | 닫기 |
|------|--------|--------|------|
| default | 투명 배경, 회색 아이콘 | 투명, 회색 (단일 사각형 또는 겹침) | 투명, 회색 X |
| hover | 어두운 배경, 밝은 아이콘 | 어두운 배경, 밝은 아이콘 | 빨강 배경, 흰 X |
| pressed | hover와 동일 (OS가 상태 전환 처리) | 동일 | 동일 |

최대화/복원 아이콘 선택은 `GlobalState.window_maximized` 값으로 결정 (네이티브 백엔드가 매 프레임 동기화).

## Automatio 노드 (셀렉터)

| debug_id | role | value | 노출 조건 |
|----------|------|-------|-----------|
| `<product>.window.minimize` | `Button` | — | 항상 |
| `<product>.window.maximize` | `Button` | `"restored"`/`"maximized"` | 항상 |
| `<product>.window.close` | `Button` | — | 항상 |

## Responsive / mobile
- 데스크톱 전용. 모바일(`HarnessConfig::mobile()`)에서는 캡션 버튼이 보이지 않을 수 있으나(창 데코레이션 개념 없음), 본 spec은 모바일 동작을 규정하지 않는다.

## Animation
- hover 색상 전환: 즉시 (트랜지션 없음, OS caption 버튼 표준).
- 최대화/복원 아이콘 교체: 다음 paint 프레임에 반영 (별도 애니메이션 없음).
