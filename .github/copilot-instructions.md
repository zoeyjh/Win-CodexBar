# Copilot Instructions for Win-CodexBar

## 아키텍처

Win-CodexBar는 AI 프로바이더 사용량 한도를 모니터링하는 Windows 시스템 트레이 앱이다. Cargo 워크스페이스에 두 개의 빌드 타겟이 있다:

- **`rust/` (크레이트: `codexbar`)** — 공유 백엔드 라이브러리 + 독립 CLI 바이너리. 프로바이더 로직, 설정, 브라우저 쿠키 추출, 트레이 아이콘 렌더링, 도메인 모델을 포함.
- **`apps/desktop-tauri/` (크레이트: `codexbar-desktop-tauri`)** — Tauri 데스크톱 셸. React 프론트엔드(`src/`)가 Rust 백엔드(`src-tauri/src/`)와 Tauri 커맨드로 통신.

`rust/src/` 핵심 서브시스템:
- `providers/` — 프로바이더별 fetch/파싱/인증 (Claude, Codex, Copilot 등). 각 프로바이더는 자체 모듈에 격리.
- `core/` — 프로바이더 팩토리(`instantiate_provider`), 자격증명, 비용 산정, JSONL 스캐닝, 속도 윈도우, 사용량 스냅샷.
- `tray/` — Tauri 셸이 공유하는 픽셀 수준 트레이 아이콘 렌더러.
- `browser/` — Windows 브라우저 감지 + 쿠키 추출 (Chrome, Edge, Brave, Firefox).
- `settings/` — 사용자 환경설정 및 프로바이더 구성 영속화.
- `cli/` — CLI 서브커맨드 (`usage`, `cost`, `config`, `serve`).

Tauri 셸(`apps/desktop-tauri/src-tauri/src/`)이 공유 크레이트를 연결하는 방식:
- `commands/` — Tauri invoke 핸들러 (프로바이더, 자격증명, 설정, 브라우저 임포트, 차트, 진단).
- `tray_bridge.rs` — 공유 트레이 렌더러를 시스템 트레이에 연결.
- `floatbar/` — 항상 위에 표시되는 플로팅 사용량 바 윈도우.

## 빌드, 테스트, 실행

```powershell
# 데스크톱 셸 (UI 작업 시 권장)
cd apps/desktop-tauri && npm run tauri:dev        # 핫리로드 개발 모드
cd apps/desktop-tauri && npm run tauri:build      # 릴리스 빌드
.\dev.ps1                                         # 툴체인 자동설치, 빌드, 실행
.\dev.ps1 -Release                                # 최적화 빌드

# CLI만
cargo build -p codexbar
cargo run -p codexbar -- --help
cargo run -p codexbar -- usage -p claude
cargo run -p codexbar -- cost

# 테스트
cargo test --manifest-path rust/Cargo.toml
cargo test --manifest-path apps/desktop-tauri/src-tauri/Cargo.toml
# 단일 테스트 실행:
cargo test --manifest-path rust/Cargo.toml -- test_name_substring

# 린트 & 포맷
cargo fmt --all
cargo clippy --all-targets -- -D warnings
```

## 핵심 컨벤션

- **프로바이더 격리**: 모든 프로바이더 로직은 `rust/src/providers/` 아래 자체 모듈에 위치. 크로스 프로바이더 분기를 추가하지 말고, `rust/src/core/provider_factory.rs`의 팩토리를 사용할 것.
- **프로바이더 생성**: 새 프로바이더는 `codexbar::core::instantiate_provider`를 통해 생성. Tauri 셸이나 CLI 커맨드에 팩토리 로직을 중복하지 말 것.
- **에러 처리**: `anyhow`/`thiserror`를 사용하고 사용자 대상 진단 메시지를 포함. 내부 진단에는 `tracing` 사용.
- **시크릿 안전**: 원시 시크릿, 쿠키, 토큰을 절대 로깅하지 말 것. `core/redactor.rs`의 리댁션 헬퍼 사용. 로컬 저장소는 `secure_file.rs`를 통해 DPAPI로 보호.
- **Tauri 커맨드**: 프론트엔드 ↔ 백엔드 통신은 `apps/desktop-tauri/src-tauri/src/commands/`의 `#[tauri::command]` 사용. 프론트엔드는 `@tauri-apps/api`로 호출.
- **트레이 아이콘 렌더링**: `rust/src/tray/`의 공유 렌더러가 픽셀 데이터를 생성하고 Tauri 셸이 적용. 아이콘 외형 변경은 공유 크레이트에서 수행.
- **커밋 스타일**: 하나의 변경에 대해 짧은 명령형 메시지 (예: `Fix Claude CLI parser`).
- **의존성**: 새 크레이트나 npm 의존성은 명시적 확인 없이 추가하지 말 것.
- **업스트림 문서**: `docs/`의 파일은 macOS/Swift 업스트림을 참조할 수 있음. `apps/desktop-tauri/`와 `rust/src/`의 활성 소스를 신뢰할 것.
