# Background Template

`plans/background/<product>/<id>.md`에 들어갈 문서의 골격.

**원칙: spec(behavior)과 implement(code) 사이의 *runtime 계약*.** 시각이 없거나 시각은
부수효과인 기능 — autosave, IPC 메시지 처리, 인덱싱, AI inference, 파일 watcher,
sync 루프 등 — 이 *언제 어떻게 도는가*를 정의.

UI 표현이 있는 기능은 design(`template/design-template.md`)을, 백그라운드 동작이
있는 기능은 이 background 문서를 쓴다. **둘 다 있는 hybrid 기능은 둘 다 작성.**

`<product>`는 `apps/<product>/`. `<id>`는 spec/design/background/implement/test 폴더에서
동일 슬러그.

---

## 골격

```markdown
# Background: <id>

## 한 줄 정의
무엇이 백그라운드에서 도는가. spec(`plans/spec/<product>/<id>.md`)의 어떤 부분을
사용자 액션 없이 자동으로 처리하나.

## Trigger / Schedule
무엇이 이 동작을 발동시키나.

| Trigger | 조건 | 빈도 |
|---------|------|------|
| 사용자 입력 후 idle | <조건> | <debounce 시간> |
| 타이머 | <interval> | <주기> |
| IPC 메시지 | `<channel>::<message>` | 메시지 수신 시 |
| 외부 이벤트 (파일 변경 등) | <조건> | watcher |
| 앱 시작 | startup | 1회 |

## Lifecycle & State
프로세스가 거치는 state 전이.

\`\`\`
idle ──[trigger]──→ running ──[ok]──→ idle
                       │
                       └──[error]──→ error_recovery ──→ idle
\`\`\`

각 state에 머무는 동안 무엇을 하나 / 무엇을 하지 않나 명시.

## Concurrency
- **인스턴스 정책**: 단일 / 세션당 단일 / 다중 / 큐잉.
- **동시성 모델**: std::sync::mpsc / std::thread / 기존 crate의 actor / 동기 직렬.
  외부 async runtime(tokio 등) 도입 금지.
- **재진입성**: 실행 중 같은 trigger가 또 와도 안전한가? 중복 발동 무시 / 큐잉 / 가장 최근만.
- **취소**: 진행 중 동작을 사용자가 취소할 수 있나? (예: 모달 닫음, 앱 종료).

## Resource budget
| 자원 | 데스크톱 한계 | 모바일 한계 |
|------|----------------|--------------|
| 메모리 | <상한> | <상한> |
| CPU | <상한, 또는 "유휴 시간만"> | <상한> |
| 디스크 I/O | <빈도/크기 한계> | <한계> |
| 네트워크 | <필요 / 없음> | <필요 / 없음> |
| 배터리 영향 | <측정 / 무시 가능> | <측정 / 무시 가능> |

모바일 제약이 데스크톱과 다른 경우만 표 채움. 동일하면 한 줄로.

## Data flow
어떤 crate의 state를 read/write하나. IPC가 있다면 protocol.

- **Read**: `<crate>::<state>` (`<누가 mutate하는가>`)
- **Write**: `<crate>::<state>` (`<sync/async write, lock 정책>`)
- **Persistence**: `<crate>` (디스크 경로 / 캐시 / 메모리만)
- **IPC**: `<channel>` 또는 `<message format>` (engine과 통신 등). 없으면 "없음".

## Failure & Recovery
| 실패 모드 | 감지 | 처리 | 사용자 통보 |
|-----------|------|------|--------------|
| <에러 1> | <어떻게 detect> | <retry / abort / fallback> | <토스트 / status / 무알림> |
| <에러 2> | ... | ... | ... |

복구 정책: 자동 재시도 횟수, backoff, 영구 실패 시 동작.

## Observability
- **Log**: `tracing::<level>!` 또는 `eprintln!` 위치 / 내용. release build에서도 켜지나.
- **Metric**: 카운터/타이머 (있으면 — 현재 프로젝트에 metric 인프라 없으면 "N/A").
- **사용자 가시 상태**: 이 background가 만들어내는 변화를 사용자가 어떻게 보는가.
  반드시 자동화 노드로 노출되어야 함 (test가 그걸로 검증).

| debug_id | role | value | 의미 |
|----------|------|-------|------|
| `<product>.save_status` | `Label` | `"saved"` / `"saving"` / `"dirty"` | 저장 상태 |
| `<product>.toast.0` | `Label` | `"<message>"` | 사용자 통보 |

## UI 인터페이스 (design 문서와의 hand-off)
이 background가 노출하는 자동화 노드를 UI design이 어떻게 시각화하나.

- design(`plans/design/<product>/<id>.md`)에 **있으면** 거기 Component breakdown
  표가 정전. 위 Observability 표는 그 일부.
- design이 **없으면** (순수 background, UI 노출 0) 이 절은 "사용자에게 보이는
  변화 없음. 자동화 노드는 test에서만 사용." 한 줄로.

## Out of scope
- 이 background에서 다루지 않는 것 (별도 spec / 후속).
- 의도적으로 동기 처리하는 것 (background 아님).
```

---

## 작성 규칙

- **단위는 spec과 동일.** "autosave 시스템 전체"가 아니라 "활성 탭 dirty 후 5초 idle
  시 자동 저장" 같은 한 동작.
- **시간/한계 수치는 명시.** "잠시 후" / "적당히 자주" 금지. 5초, 100ms, 10MB 등.
- **외부 async runtime(tokio, async-std) 도입 금지.** 본 프로젝트는 std::thread + std::sync::mpsc
  기반. 다른 패턴이 필요하면 spec/architecture 결정 먼저.
- **Observability 표가 비면 안 됨.** background는 사용자가 직접 보지 못하므로 자동화 노드로
  관찰 가능해야 test가 검증할 수 있다. 노드 0개라면 "사용자가 결과를 알 방법이
  없는 동작"이므로 spec을 다시 검토.
- **길이**: 100–180줄 권장. 단순 타이머 동작은 80줄로 충분.

---

## 4단계 워크플로우 안에서

| Phase | UI feature | Background feature | Hybrid |
|-------|------------|---------------------|--------|
| 1. spec | 필수 | 필수 | 필수 |
| 2a. design (UI) | 필수 | 생략 | 필수 |
| 2b. background | 생략 | 필수 | 필수 |
| 3. implement | 필수 | 필수 | 필수 |
| 4. test | 필수 | 필수 | 필수 |

implement는 design과 background 둘 다 입력으로 받는다 (있는 쪽만).
test는 둘 다 검증할 수 있어야 한다 — UI 변화는 selector value 비교, background 변화는
Observability 표의 자동화 노드 value 비교.

---

## 예시 (id: docs-autosave)

```markdown
# Background: docs-autosave

## 한 줄 정의
docs 활성 세션이 dirty 상태에서 5초간 입력이 없으면 자동으로 디스크에 저장한다.

## Trigger / Schedule
| Trigger | 조건 | 빈도 |
|---------|------|------|
| 사용자 입력 후 debounce | 활성 세션 dirty=true 진입 | 마지막 입력 후 5초 |
| 강제 저장 (Ctrl+S) | 즉시 | 사용자 액션 |
| 앱 종료 직전 | 모든 dirty 세션 | 1회 |

## Lifecycle & State
\`\`\`
saved ──[edit]──→ dirty ──[5s idle]──→ saving ──[ok]──→ saved
                                          │
                                          └──[error]──→ dirty (retry on next idle)
\`\`\`

## Concurrency
- 인스턴스 정책: 세션당 단일. 같은 세션의 두 번째 saving은 첫 번째 완료 후 재평가.
- 동시성 모델: 메인 스레드 timer(`AnimFrame` 기반) + std::thread로 디스크 write 분리.
- 재진입성: 입력이 saving 중에 또 오면 saving 완료 후 다시 dirty → 5초 카운트 리셋.
- 취소: 앱 종료 시 진행 중 saving은 끝까지 기다림 (사용자 데이터 보존).

## Resource budget
- 디스크 I/O만, CPU 거의 0. 메모리 추가 할당 없음 (engine이 이미 Document 보유).
- 모바일/데스크톱 동일.

## Data flow
- Read: `DocumentSession.engine.document` (직렬화 직전 snapshot).
- Write: `tench_office_io::docs::format::write_to_path(path, document)`.
- Persistence: 디스크 경로 = 세션 file_path. 새 문서면 별도 trigger 안 됨 (Save As 필요).
- IPC: 없음.

## Failure & Recovery
| 실패 모드 | 감지 | 처리 | 사용자 통보 |
|-----------|------|------|--------------|
| 디스크 가득 | I/O Err | dirty로 복귀, 다음 5s idle에 재시도 | 토스트 에러 (반복 표시 안 함) |
| 권한 없음 | I/O Err | 동일 | 토스트 에러 |
| 파일 사용 중 (잠김) | I/O Err | 동일 | 토스트 에러 |
| 직렬화 실패 | parse_to_bytes Err | dirty 유지, retry | 토스트 (드물고 심각) |

자동 재시도 무한 (사용자가 끄지 않는 한). backoff 없음 — 단순 idle 재트리거.

## Observability
- Log: `tracing::info!("autosave session={idx} bytes={n}")` on success.
  `tracing::error!` on failure.
- Metric: N/A (인프라 없음).
- 사용자 가시 상태:

| debug_id | role | value | 의미 |
|----------|------|-------|------|
| `docs.save_status` | `Label` | `"saved"` / `"saving"` / `"dirty"` | 상태 표시 |
| `docs.toast.0` | `Label` | `"Cannot save: <reason>"` | 실패 통보 |

## UI 인터페이스
design(`plans/design/docs/docs-autosave.md`)이 status badge / 토스트의 시각 정의.
이 background는 위 자동화 노드의 value를 갱신할 책임만.

## Out of scope
- 클라우드 동기화 (별도 spec).
- 충돌 해결 (사용자가 외부에서 같은 파일 편집한 경우 — 별도 spec).
- Save As 흐름 (사용자 액션, 별도 spec).
```
