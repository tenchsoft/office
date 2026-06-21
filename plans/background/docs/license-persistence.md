# Background: license-persistence

## 한 줄 정의
앱 시작 시 로컬 암호화 저장소에서 라이선스 자격 증명(license_key, device_id,
device_token, expires_at)을 읽어 메모리에 올리고, 활성화/갱신/해제 이벤트를
받아 저장소에 atomic write 한다. spec(`plans/spec/docs/license-persistence.md`)의
모든 상태 전이를 사용자 액션 없이 자동으로 처리한다.

## Trigger / Schedule

| Trigger | 조건 | 빈도 |
|---------|------|------|
| 앱 시작 | 메인 윈도우 생성 직후 | 1회 |
| 활성화 완료 | License 탭에서 license_key 입력 후 서버 응답 수신 | 활성화 시 |
| Token 갱신 | device_token 만료 24시간 이내 | 갱신 주기 |
| 기기 해제 | License 탭에서 "Release" 클릭 | 사용자 액션 |
| 만료 감지 | token 검증 시 license.expires_at 과거 | 검증 시 |

## Lifecycle & State

```
                            ┌──────────────┐
                            │ uninitialized │
                            └──────┬───────┘
                                   │ load_or_init()
                  ┌────────────────┼────────────────┐
                  │                 │                │
                  ▼                 ▼                ▼
          ┌─────────────┐   ┌─────────────┐   ┌─────────────┐
          │ unactivated │   │   active    │   │  expired    │
          │ (no creds)  │   │ (valid tok) │   │ (token dead)│
          └──────┬──────┘   └──────┬──────┘   └──────┬──────┘
                 │                 │                  │
                 │ activate        │ release /        │ reactivate
                 │                 │ revoke           │ (license_key 재입력)
                 ▼                 ▼                  │
       (active로 전이)    (unactivated로 전이) ────────┘
```

각 state:
- **uninitialized**: 초기. device_id 도출 전. 모든 읽기는 None 반환.
- **unactivated**: device_id는 도출됨. license 자격 증명 없음. 활성화 가능.
- **active**: device_token 보유 + 만료 전. 모든 인증 요구 기능 사용 가능.
- **expired**: license_keys.expires_at 과거이거나 device_token 만료. 갱신 필요.

## Concurrency

- **인스턴스 정책**: 앱 전체 단일 인스턴스 (`Arc<LicenseStore>`).
- **동시성 모델**: `std::sync::RwLock<LicenseState>` 로 보호. 읽기 다중, 쓰기 단일.
  외부 async runtime (tokio 등) 금지 — `std::thread::spawn`만 허용.
- **재진입성**: 읽기는 언제나 안전. 쓰기는 순차화됨. 쓰기 중 읽기는 이전 상태 관찰.
- **취소**: 쓰기 도중 앱 강제 종료 시 → atomic write 보장 (rename 방식)으로
  완전히 새 상태거나 완전히 이전 상태거나 둘 중 하나. 중간 상태 불가.

## Resource budget

| 자원 | 데스크톱 한계 | 모바일 한계 |
|------|----------------|--------------|
| 메모리 | 4 KB 고정 (state + token) | N/A (본 spec 데스크톱 전용) |
| CPU | 시작 시 1회 <50ms (file read + AES-GCM decrypt), 이후 0 | N/A |
| 디스크 I/O | 시작 1회 읽기 + 활성화/갱신/해제 시마다 1회 atomic write (~1 KB) | N/A |
| 네트워크 | 본 spec에서 사용 ❌ (갱신은 `update-check-weekly`에서) | N/A |
| 배터리 영향 | 무시 가능 | N/A |

## Data flow

- **Read**: 로컬 파일 `<app_data_dir>/license-store.bin` 읽기.
  - Windows: `%APPDATA%\Tench\<product>\license-store.bin`
  - macOS: `~/Library/Application Support/Tench/<product>/license-store.bin`
  - Linux: `~/.local/share/Tench/<product>/license-store.bin`
- **Write**: 같은 경로. 임시 파일(`.tmp` 접미사)에 쓰고 `rename`으로 atomic 교체.
- **Crypto**: AES-256-GCM (workspace dep `aes-gcm` 이미 사용 중).
  - 키 도출: device_id 자체를 키 파생 seed로 사용 (HKDF-SHA256).
  - 즉, 같은 기기에서만 복호화 가능. 파일 복사해 다른 기기로 가면 무의미.
- **State (in-memory)**:
  ```rust
  pub struct LicenseState {
      pub device_id: String,           // SHA-256 hex, 항상 존재
      pub license_key: Option<String>, // TENCH-... 형태, 미인증 시 None
      pub device_token: Option<String>,// base64url, 만료 시 None
      pub token_expires_at: Option<DateTime<Utc>>,
      pub last_refreshed_at: Option<DateTime<Utc>>,
  }
  ```
- **IPC**: 없음. 본 crate는 순수 라이브러리.

## Device ID 도출

플랫폼별 안정 식별자 → SHA-256 hex (64자):

| 플랫폼 | 소스 | 접근 방식 |
|---|---|---|
| Windows | `HKLM\SOFTWARE\Microsoft\Cryptography\MachineGuid` | `windows-sys` crate (이미 workspace dep) |
| macOS | `IOPlatformUUID` | `IOKit` framework FFI 또는 `ioreg` 명령 |
| Linux | `/etc/machine-id` 또는 `/var/lib/dbus/machine-id` | 파일 읽기 |

도출 실패 시 폴백:
1. 두 번째 소스 시도 (Linux: dbus 버전)
2. 모두 실패 → 1회성 랜덤 UUID 생성. 메모리에만 보관, 디스크 저장 ❌.
   → 자격 증명 영구 저장 불가 → 사실상 미인증 상태로 동작.
   → 사용자에게 "기기 식별 불가" 에러 로그만.

## PC 요청 코드 인코딩

`encodePcRequestCode(device_id, device_meta, nonce)` — `tench-license-store`가 제공.

형식: `TENCHPC-<base64url(JSON)>`

JSON 스키마는 `tench-docs/plans/contracts/Licensing/licensing-auth.md` §10.2 참조.
TTL = 10분. 서명 ❌ (양쪽 모두 사용자 인증된 경로).

## Failure & Recovery

| 실패 모드 | 감지 | 처리 | 사용자 통보 |
|-----------|------|------|--------------|
| 자격 증명 파일 없음 | `File::open` Err(NotFound) | 정상 케이스, unactivated로 진입 | 없음 |
| 자격 증명 파일 손상 | AES-GCM 복호화 실패 / JSON 파싱 실패 | unactivated로 폴백, 파일은 그대로 둠 | 에러 로그만 |
| 파일 읽기 권한 없음 | `File::open` Err(PermissionDenied) | unactivated로 진입 | 에러 로그만 |
| Atomic write 실패 (디스크 가득) | rename 실패 | 이전 상태 유지, 호출자에게 Err 반환 | License 탭에서 에러 토스트 |
| OS 식별자 접근 실패 | 레지스트리/ioreg/machine-id 읽기 Err | 임시 UUID 사용, 영구 저장 ❌ | 에러 로그 + License 탭 경고 |
| Token 만료 | `token_expires_at < now` | expired 상태로 전이 | UI에서 자동 갱신 시도 (update-check-weekly) |

복구 정책:
- 읽기 실패는 모두 non-fatal. 앱은 항상 시작됨.
- 쓰기 실패는 호출자에게 전파 (License 탭 UI가 토스트로 표시).

## Observability

- **Log**: `tracing` 매크로 사용.
  - 시작: `tracing::info!("license store loaded state=...")
  - 쓰기: `tracing::info!("license store wrote state=...")`
  - 실패: `tracing::error!("license store failed: {error}")`
  - release build에서도 info/error는 유지.
- **Metric**: N/A (metric 인프라 없음).
- **사용자 가시 상태**: 자동화 노드로 노출.

| debug_id | role | value | 의미 |
|----------|------|-------|------|
| `<product>.license.state` | `Label` | `"uninitialized"` / `"unactivated"` / `"active"` / `"expired"` | 현재 라이선스 상태 |
| `<product>.license.device_id` | `Label` | `"<sha256-hex-64chars>"` | 이 기기의 식별자 (활성화 진단용) |
| `<product>.license.license_key` | `Label` | `"TENCH-..."` 또는 `""` | 활성화된 라이선스 키 (미인증 시 빈 문자열) |
| `<product>.license.expires_at` | `Label` | `"2026-07-21..."` 또는 `""` | 라이선스 만료 시각 |

> **Privacy**: `device_id`와 `license_key`는 automation 노드에 노출되지만
> 이는 debug/test 환경에서만 접근 가능 (release build에서는 automation 비활성).
> 자세한 내용은 AGENTS.md "UI Automation & Headless Visual Capture" 참조.

## UI 인터페이스 (design 문서와의 hand-off)

design 문서는 별도 없음 (순수 background). 단, `license-tab-ui` design에서 이
background가 노출한 자동화 노드를 읽어 License 탭의 상태 표시 위젯 값을 채운다.

- `license-tab-ui`의 design (`plans/design/docs/license-tab-ui.md`)에서:
  - 상태 배지: `<product>.license.state` 값을 읽어 "Active" / "Expired" /
    "Not activated" 라벨로 표시
  - 라이선스 키 표시: `<product>.license.license_key` 값을 읽어 표시
  - 만료일 표시: `<product>.license.expires_at` 값을 읽어 포맷팅

이 background는 위 노드의 value를 갱신할 책임만. UI 렌더링은 design 쪽에서.

## Out of scope
- License 탭의 시각적 레이아웃 — `license-tab-ui` design.
- 서버와의 인증 통신 (HTTP 클라이언트) — `license-tab-ui` implement에서
  `tench-update-client`를 통해 호출. 본 crate은 순수 저장소.
- 매니페스트 fetch 및 업데이트 적용 — `update-check-weekly`, `update-install-flow`.
- Token 갱신 HTTP 호출 — `update-check-weekly` background. 단, 갱신 후 받은
  새 token을 저장하는 동작은 본 spec의 쓰기 경로를 재사용.
- 파일 백업/마이그레이션 — 저장소 포맷이 변경되면 버전 필드로 마이그레이션.
  현재는 v1만 존재, 마이그레이션 경로 ❌.
