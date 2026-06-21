<div align="center">

# Tench Office

**100% Rust 오피스 스위트. 로컬 우선, AI 보조, 클라우드 없음.**

Docs · Sheets · Slides · Kodocs — 네 개의 편집기가 내 PC에서 그대로 실행됩니다.

[![Language: Rust](https://img.shields.io/badge/Language-Rust-dea584.svg)](https://www.rust-lang.org/)
[![Framework: Tauri 2](https://img.shields.io/badge/Framework-Tauri_2-FFC140.svg)](https://v2.tauri.app/)
[![License: UNLICENSED](https://img.shields.io/badge/License-UNLICENSED-red.svg)](#license)
[![Status: Preview](https://img.shields.io/badge/Status-Preview-orange.svg)](#roadmap)
[![Pricing: $1/mo](https://img.shields.io/badge/Pricing-%241%2Fmo-1ca096.svg)](https://tenchsoft.com/pricing)

[English](README.md) · [한국어](README.ko.md)

</div>

---

## 개요

Tench Office는 100% Rust + Tauri 2로 구축된 크로스플랫폼 오피스 스위트입니다. 네 개의 편집기가 모두 로컬에서 실행되며, 텔레메트리·계정·클라우드 동기화가 없습니다. AI 보조는 로컬 Tench Engine을 통해 처리되어 문서가 기기 밖으로 나가지 않습니다.

Windows, macOS, Linux를 지원합니다. 기기당 월 $1 구독으로 네 개 편집기의 모든 업데이트를 받습니다. 언제든 취소 가능, 현재 버전은 계속 사용.

## 제품

| | 제품 | 설명 | 기본 포맷 |
|---|---|---|---|
| 📝 | **Docs** | 워드 프로세서 — 변경 추적, 코멘트, 템플릿 | `.docx` |
| 📊 | **Sheets** | 스프레드시트 — 200+ 수식, 차트, 피벗 | `.xlsx` |
| 🎤 | **Slides** | 프레젠테이션 — AI 레이아웃, 발표자 모드 | `.pptx` |
| 🇰🇷 | **Kodocs** | 한글 문서 편집기 — HWP/HWPX 정밀 호환 | `.hwp` / `.hwpx` |

Kodocs는 한국 시장 차별점입니다 — 한글과컴퓨터 포맷을 정밀하게 렌더링하는 네이티브 HWP 편집기.

## 특징

- **100% Rust** — 엔진, UI, 포맷, AI까지 전부 Rust. JavaScript 없음.
- **로컬 우선 AI** — 추론은 IPC로 Tench Engine을 통해 처리. HTTP 사용 안 함.
- **포맷 라운드트립** — `.docx`, `.xlsx`, `.pptx`, `.odt`, `.ods`, `.odp`, `.hwp`, `.hwpx`, Markdown, PDF, HTML, 일반 텍스트.
- **구조적 프라이버시** — 파일은 디스크에, 분석 없음, 자동 클라우드 동기화 없음.
- **크로스플랫폼** — 단일 Rust 코드베이스로 Windows, macOS, Linux, Android, iOS.
- **자체 UI** — `tench-ui`는 Vello + wgpu로 렌더링되는 자체 retained-mode 위젯 프레임워크. Electron 없음, 웹뷰 없음.

## 아키텍처

```
apps/<product>/src-tauri/        제품 셸 (Tauri 2)
crates/document-core/            공유 문서 모델 + 엔진
crates/office-io/                포맷 리더/라이터 (docx, xlsx, pptx, odt, ...)
crates/office-runtime/           제품 간 디스패치 (열기, 저장, 내보내기)
crates/hwp-io/                   HWP / HWPX 정밀 I/O
crates/sheets-core/              수식 엔진, 그리드 모델
crates/engine-core/              Tench Engine 클라이언트 (라우팅, 제공자)
crates/engine-native/            로컬 모델 발견 (GGUF, SafeTensors)
crates/storage-core/             로컬 우선 저장 + 암호화 (AES-GCM)
crates/fs-core/                  파일 시스템 접근 정책
crates/tench-ui/                 자체 위젯 프레임워크
crates/tench-ui-test/            헤드리스 시각 캡처 / E2E 하네스
tools/architecture-guard/        레포 구조 강제
```

전체 레이어 맵과 계획 폴더 규칙은 [`ARCHITECTURE.md`](ARCHITECTURE.md)를 참고하세요.

## 빌드

```bash
# 필요: Rust stable, 플랫폼 빌드 도구 (MSVC / Xcode / gcc).
cargo check --workspace --locked
cargo build --workspace --locked
cargo test --workspace --locked
```

단일 제품 실행:

```bash
cargo run --locked -p docs    # 또는: sheets, slides, kodocs
```

## 로드맵

- [x] Tench-One 모노레포에서 최초 분리
- [x] 공유 office-io + office-runtime 레이어
- [x] HWP/HWPX 정밀 I/O (`hwp-io`)
- [ ] Docs/Sheets/Slides 첫 안정 릴리스
- [ ] Kodocs 1.0 (한글 템플릿 라이브러리 + 맞춤법)
- [ ] 모바일 동반 앱

## 가격

- **기기당 월 $1** — 모든 업데이트, 패치, 새 기능 포함.
- 언제든 취소, 현재 버전 유지. 잠금 없음.

→ https://tenchsoft.com/pricing

## 라이선스

UNLICENSED — 소스는 검토용으로 공개, 바이너리 배포는 구독 필요. `LICENSE` 참조 또는 hello@tenchsoft.com.

## 자매 프로젝트

- **[Tench Engine](https://github.com/tenchsoft/engine)** — 모든 Tench 앱이 사용하는 로컬 AI 추론 런타임
- **[Tench Media](https://github.com/tenchsoft/media)** — 이미지/비디오 스위트 (View, Pixel Design, Player, Composer)
- **[Tench Authoring](https://github.com/tenchsoft/authoring)** — 장편 집필 도구 (Story, Universe)
- **[Tench Knowledge](https://github.com/tenchsoft/knowledge)** — 연구/학습 (Research, Study)
- **[Tench Code](https://github.com/tenchsoft/code)** — AI 보조 코드 편집기
- **[tenchsoft.com](https://tenchsoft.com)** — 계정, 라이선스, 다운로드
- **[Dolphin Labs](https://dolphinelabs.com)** — 오픈소스 자매 레이블 (mymy, OpenNodia)
