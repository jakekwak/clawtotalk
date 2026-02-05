# 📚 문서 인덱스 (Documentation Index)

Dioxus Voice Assistant 프로젝트의 모든 문서를 한눈에 볼 수 있는 인덱스입니다.

---

## 🚀 시작하기

처음 시작하시나요? 이 순서대로 읽어보세요:

1. **[README.md](README.md)** - 프로젝트 개요 및 기능 소개
2. **[QUICKSTART.md](QUICKSTART.md)** - 5분 안에 실행하기 (추천!)
3. **[LOCAL_TESTING_GUIDE.md](LOCAL_TESTING_GUIDE.md)** - 상세한 테스트 가이드

---

## 📖 핵심 문서

### 프로젝트 이해하기

| 문서 | 설명 | 대상 |
|------|------|------|
| **[README.md](README.md)** | 프로젝트 개요, 기능, 빠른 시작 | 모든 사용자 |
| **[ARCHITECTURE.md](ARCHITECTURE.md)** | 시스템 아키텍처 및 설계 | 개발자 |
| **[BUILD.md](BUILD.md)** | 빌드 및 배포 가이드 | 개발자 |

### 테스트 및 검증

| 문서 | 설명 | 대상 |
|------|------|------|
| **[QUICKSTART.md](QUICKSTART.md)** | 5분 빠른 시작 가이드 | 처음 사용자 |
| **[LOCAL_TESTING_GUIDE.md](LOCAL_TESTING_GUIDE.md)** | 포괄적인 로컬 테스트 가이드 | 테스터, 개발자 |
| **[FINAL_VERIFICATION_REPORT.md](FINAL_VERIFICATION_REPORT.md)** | 최종 검증 보고서 (90개 테스트 결과) | 모든 사용자 |

### 스펙 문서

| 문서 | 설명 | 대상 |
|------|------|------|
| **[.kiro/specs/dioxus-voice-assistant/requirements.md](.kiro/specs/dioxus-voice-assistant/requirements.md)** | 요구사항 명세 | 개발자, PM |
| **[.kiro/specs/dioxus-voice-assistant/design.md](.kiro/specs/dioxus-voice-assistant/design.md)** | 설계 문서 (아키텍처, 데이터 모델, 속성) | 개발자 |
| **[.kiro/specs/dioxus-voice-assistant/tasks.md](.kiro/specs/dioxus-voice-assistant/tasks.md)** | 구현 작업 목록 (52개 작업, 100% 완료) | 개발자, PM |

---

## 🖥️ 플랫폼별 가이드

### Windows

| 문서 | 설명 |
|------|------|
| **[WINDOWS_OPTIMIZATION.md](WINDOWS_OPTIMIZATION.md)** | Windows 플랫폼 최적화 가이드 |
| **[windows/README.md](windows/README.md)** | Windows 빌드 및 배포 |

### macOS

| 문서 | 설명 |
|------|------|
| **[MACOS_OPTIMIZATION.md](MACOS_OPTIMIZATION.md)** | macOS 플랫폼 최적화 가이드 |
| **[macos/README.md](macos/README.md)** | macOS 빌드 및 앱 번들 생성 |

### Android

| 문서 | 설명 |
|------|------|
| **[ANDROID_OPTIMIZATION.md](ANDROID_OPTIMIZATION.md)** | Android 플랫폼 최적화 가이드 |
| **[android/README.md](android/README.md)** | Android 빌드 및 APK 생성 |

### iOS

| 문서 | 설명 |
|------|------|
| **[IOS_OPTIMIZATION.md](IOS_OPTIMIZATION.md)** | iOS 플랫폼 최적화 가이드 |
| **[ios/README.md](ios/README.md)** | iOS 빌드 및 배포 |
| **[ios/QUICK_START.md](ios/QUICK_START.md)** | iOS 빠른 시작 가이드 |
| **[ios/VERIFICATION_CHECKLIST.md](ios/VERIFICATION_CHECKLIST.md)** | iOS 검증 체크리스트 |

---

## 📊 작업 요약 문서

각 주요 작업의 완료 요약:

| 문서 | 설명 |
|------|------|
| **[TASK_10.1_SUMMARY.md](TASK_10.1_SUMMARY.md)** | Windows 플랫폼 최적화 완료 요약 |
| **[TASK_10.2_SUMMARY.md](TASK_10.2_SUMMARY.md)** | macOS 플랫폼 최적화 완료 요약 |
| **[TASK_10.3_SUMMARY.md](TASK_10.3_SUMMARY.md)** | Android 플랫폼 최적화 완료 요약 |
| **[TASK_10.4_SUMMARY.md](TASK_10.4_SUMMARY.md)** | iOS 플랫폼 최적화 완료 요약 |
| **[SETUP_COMPLETE.md](SETUP_COMPLETE.md)** | 전체 설정 완료 요약 |

---

## 🔧 설정 파일

| 파일 | 설명 |
|------|------|
| **[.env.example](.env.example)** | 환경 변수 템플릿 (API 키, 서버 설정) |
| **[Cargo.toml](Cargo.toml)** | Rust 프로젝트 설정 |
| **[Dioxus.toml](Dioxus.toml)** | Dioxus 프레임워크 설정 |
| **[package.json](package.json)** | Node.js 서버 의존성 |

---

## 🧪 테스트 파일

| 파일 | 설명 | 테스트 수 |
|------|------|-----------|
| **[tests/proptest.rs](tests/proptest.rs)** | 속성 기반 테스트 (10개 속성) | 10 |
| **[tests/integration_tests.rs](tests/integration_tests.rs)** | 통합 테스트 | 16 |
| **[tests/performance_proptest.rs](tests/performance_proptest.rs)** | 성능 테스트 | 5 |
| **단위 테스트** (src/ 내부) | 각 모듈의 단위 테스트 | 59 |
| **총계** | | **90** |

---

## 🛠️ 유틸리티 스크립트

| 스크립트 | 설명 | 사용법 |
|----------|------|--------|
| **[quick_test.sh](quick_test.sh)** | 자동화된 전체 테스트 스크립트 | `./quick_test.sh` |
| **[macos/build_app_bundle.sh](macos/build_app_bundle.sh)** | macOS 앱 번들 생성 | `cd macos && ./build_app_bundle.sh` |
| **[macos/build_universal.sh](macos/build_universal.sh)** | macOS 유니버설 바이너리 생성 | `cd macos && ./build_universal.sh` |
| **[ios/build_ios.sh](ios/build_ios.sh)** | iOS 빌드 스크립트 | `cd ios && ./build_ios.sh` |
| **[android/build_android.sh](android/build_android.sh)** | Android APK 빌드 | `cd android && ./build_android.sh` |

---

## 📁 소스 코드 구조

### 핵심 모듈

| 파일 | 설명 |
|------|------|
| **[src/main.rs](src/main.rs)** | 애플리케이션 엔트리 포인트 |
| **[src/lib.rs](src/lib.rs)** | 라이브러리 루트 |
| **[src/audio.rs](src/audio.rs)** | 오디오 녹음 및 재생 |
| **[src/api.rs](src/api.rs)** | 서버 API 클라이언트 |
| **[src/vad.rs](src/vad.rs)** | 음성 활동 감지 (VAD) |
| **[src/recording.rs](src/recording.rs)** | 녹음 모드 관리 |
| **[src/models.rs](src/models.rs)** | 데이터 모델 |
| **[src/state.rs](src/state.rs)** | 애플리케이션 상태 |
| **[src/error.rs](src/error.rs)** | 오류 타입 정의 |
| **[src/error_handler.rs](src/error_handler.rs)** | 오류 처리 로직 |
| **[src/connection.rs](src/connection.rs)** | 서버 연결 관리 |
| **[src/compression.rs](src/compression.rs)** | 오디오 압축 |
| **[src/memory.rs](src/memory.rs)** | 메모리 최적화 |
| **[src/performance.rs](src/performance.rs)** | 성능 모니터링 |

### UI 컴포넌트

| 파일 | 설명 |
|------|------|
| **[src/ui/app.rs](src/ui/app.rs)** | 메인 앱 컴포넌트 |
| **[src/ui/recording_button.rs](src/ui/recording_button.rs)** | 녹음 버튼 |
| **[src/ui/conversation_history.rs](src/ui/conversation_history.rs)** | 대화 기록 |
| **[src/ui/settings_screen.rs](src/ui/settings_screen.rs)** | 설정 화면 |
| **[src/ui/connection_status.rs](src/ui/connection_status.rs)** | 연결 상태 표시 |
| **[src/ui/error_notification.rs](src/ui/error_notification.rs)** | 오류 알림 |

### 플랫폼별 코드

| 파일 | 설명 |
|------|------|
| **[src/platform/mod.rs](src/platform/mod.rs)** | 플랫폼 모듈 루트 |
| **[src/platform/windows.rs](src/platform/windows.rs)** | Windows 최적화 |
| **[src/platform/macos.rs](src/platform/macos.rs)** | macOS 최적화 |
| **[src/platform/android.rs](src/platform/android.rs)** | Android 최적화 |
| **[src/platform/ios.rs](src/platform/ios.rs)** | iOS 최적화 |

---

## 🎯 사용 시나리오별 문서

### "처음 시작합니다"
1. [README.md](README.md) - 프로젝트 이해
2. [QUICKSTART.md](QUICKSTART.md) - 5분 안에 실행
3. [LOCAL_TESTING_GUIDE.md](LOCAL_TESTING_GUIDE.md) - 상세 테스트

### "특정 플랫폼에 빌드하고 싶습니다"
1. [BUILD.md](BUILD.md) - 전체 빌드 가이드
2. 플랫폼별 최적화 문서:
   - [WINDOWS_OPTIMIZATION.md](WINDOWS_OPTIMIZATION.md)
   - [MACOS_OPTIMIZATION.md](MACOS_OPTIMIZATION.md)
   - [ANDROID_OPTIMIZATION.md](ANDROID_OPTIMIZATION.md)
   - [IOS_OPTIMIZATION.md](IOS_OPTIMIZATION.md)

### "아키텍처를 이해하고 싶습니다"
1. [ARCHITECTURE.md](ARCHITECTURE.md) - 전체 아키텍처
2. [.kiro/specs/dioxus-voice-assistant/design.md](.kiro/specs/dioxus-voice-assistant/design.md) - 상세 설계
3. 소스 코드 탐색

### "테스트를 실행하고 싶습니다"
1. [QUICKSTART.md](QUICKSTART.md) - 빠른 테스트
2. [LOCAL_TESTING_GUIDE.md](LOCAL_TESTING_GUIDE.md) - 포괄적인 테스트
3. `./quick_test.sh` 실행

### "프로젝트 상태를 확인하고 싶습니다"
1. [FINAL_VERIFICATION_REPORT.md](FINAL_VERIFICATION_REPORT.md) - 최종 검증 보고서
2. [.kiro/specs/dioxus-voice-assistant/tasks.md](.kiro/specs/dioxus-voice-assistant/tasks.md) - 작업 진행 상황

---

## 📈 프로젝트 통계

- **총 문서 수**: 30+
- **총 테스트 수**: 90 (모두 통과 ✅)
- **지원 플랫폼**: 4 (Windows, macOS, Android, iOS)
- **작업 완료율**: 100% (52/52)
- **요구사항 충족**: 100% (11/11)
- **코드 라인 수**: ~10,000+ (Rust + JavaScript)

---

## 🔍 문서 검색 팁

### 키워드로 찾기

- **"시작"** → QUICKSTART.md, README.md
- **"테스트"** → LOCAL_TESTING_GUIDE.md, quick_test.sh
- **"빌드"** → BUILD.md, 플랫폼별 README
- **"오류"** → LOCAL_TESTING_GUIDE.md (문제 해결 섹션)
- **"성능"** → FINAL_VERIFICATION_REPORT.md, performance_proptest.rs
- **"API"** → design.md, api.rs

### 플랫폼별 찾기

- **Windows** → WINDOWS_OPTIMIZATION.md, windows/
- **macOS** → MACOS_OPTIMIZATION.md, macos/
- **Android** → ANDROID_OPTIMIZATION.md, android/
- **iOS** → IOS_OPTIMIZATION.md, ios/

---

## 📝 문서 업데이트 이력

| 날짜 | 문서 | 변경 사항 |
|------|------|-----------|
| 2026-02-05 | 전체 | 초기 문서 세트 생성 |
| 2026-02-05 | LOCAL_TESTING_GUIDE.md | 로컬 테스트 가이드 추가 |
| 2026-02-05 | QUICKSTART.md | 5분 빠른 시작 가이드 추가 |
| 2026-02-05 | FINAL_VERIFICATION_REPORT.md | 최종 검증 보고서 생성 |
| 2026-02-05 | README.md | 프로젝트 개요 업데이트 |

---

## 💡 문서 기여 가이드

문서를 개선하고 싶으신가요?

1. 오타나 오류 발견 시 이슈 생성
2. 새로운 가이드 제안
3. 번역 기여 (영어 ↔ 한국어)
4. 스크린샷 및 다이어그램 추가

---

**문서 버전**: 1.0.0  
**마지막 업데이트**: 2026-02-05  
**프로젝트 버전**: 0.1.0
