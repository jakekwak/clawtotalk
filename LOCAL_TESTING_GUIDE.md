# 로컬 테스트 가이드 (Local Testing Guide)

이 가이드는 Dioxus Voice Assistant를 로컬 환경에서 셋업하고 테스트하는 방법을 설명합니다.

---

## 📋 목차

1. [사전 요구사항](#사전-요구사항)
2. [빠른 시작](#빠른-시작)
3. [서버 셋업](#서버-셋업)
4. [클라이언트 앱 빌드 및 실행](#클라이언트-앱-빌드-및-실행)
5. [테스트 실행](#테스트-실행)
6. [플랫폼별 테스트](#플랫폼별-테스트)
7. [문제 해결](#문제-해결)

---

## 사전 요구사항

### 필수 도구

1. **Rust 툴체인** (1.75 이상)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup update
   ```

2. **Node.js** (18 이상) - 테스트 서버용
   ```bash
   # macOS
   brew install node
   
   # 또는 https://nodejs.org 에서 다운로드
   ```

3. **Dioxus CLI** (선택사항, 핫 리로딩용)
   ```bash
   cargo install dioxus-cli
   ```

### 플랫폼별 추가 요구사항

#### macOS
- Xcode Command Line Tools
  ```bash
  xcode-select --install
  ```

#### Windows
- Visual Studio Build Tools
- Windows SDK

#### Android
- Android Studio
- Android NDK
- Java Development Kit (JDK)

#### iOS
- Xcode (macOS에서만)
- iOS SDK

---

## 빠른 시작

가장 빠르게 테스트하는 방법:

```bash
# 1. 저장소 클론 (이미 있다면 생략)
cd /path/to/dioxus-voice-assistant

# 2. 의존성 확인
cargo check

# 3. 테스트 실행
cargo test

# 4. 데스크톱 앱 실행 (서버 없이 UI만 확인)
cargo run
```

---

## 서버 셋업

클라이언트 앱이 실제로 음성 인식, AI 응답, TTS를 사용하려면 서버가 필요합니다.

### 옵션 1: 목 서버 (Mock Server) - 추천

프로젝트에 포함된 Node.js 목 서버를 사용하면 API 키 없이 테스트할 수 있습니다.

#### 1. 환경 설정

```bash
# .env 파일 생성
cp .env.example .env
```

`.env` 파일 편집:
```env
# 실제 API 키가 있다면 입력 (없어도 목 서버는 작동)
OPENAI_API_KEY=your_openai_key_here
ELEVENLABS_API_KEY=your_elevenlabs_key_here
ELEVENLABS_VOICE_ID=your_voice_id_here

# 서버 설정
PORT=3333
BOT_NAME=Gerty
USE_OPENCLAW=false

# 목 모드 활성화 (API 키 없이 테스트)
MOCK_MODE=true
```

#### 2. 의존성 설치

```bash
npm install
```

#### 3. 서버 실행

```bash
npm start
```

서버가 `http://localhost:3333`에서 실행됩니다.

#### 4. 서버 테스트

```bash
# 헬스 체크
curl http://localhost:3333/health

# 예상 응답:
# {"status":"ok","mode":"mock","time":"2026-02-05T..."}
```

### 옵션 2: 실제 OpenClaw 서버

실제 OpenClaw 서버를 사용하려면:

1. OpenClaw 설치 및 설정
2. `.env`에서 `USE_OPENCLAW=true` 설정
3. 필요한 API 키 설정

자세한 내용은 [OpenClaw 문서](https://github.com/your-org/openclaw)를 참조하세요.

---

## 클라이언트 앱 빌드 및 실행

### 1. 개발 모드 (빠른 테스트)

```bash
# 데스크톱 앱 실행
cargo run

# 또는 Dioxus CLI 사용 (핫 리로딩)
dx serve --platform desktop
```

앱이 실행되면:
1. 설정 화면에서 서버 URL 입력: `http://localhost:3333`
2. 연결 타입 선택: `LocalNetwork`
3. 녹음 모드 선택: `Hold`, `Toggle`, 또는 `Auto`
4. 저장 버튼 클릭

### 2. Release 빌드 (성능 테스트)

```bash
# 최적화된 빌드
cargo build --release

# 실행
./target/release/dioxus-voice-assistant
```

### 3. 웹 버전 (브라우저)

```bash
dx serve --platform web
```

브라우저에서 `http://localhost:8080` 접속

---

## 테스트 실행

### 전체 테스트 스위트

```bash
# 모든 테스트 실행
cargo test --all-features

# 출력 포함
cargo test --all-features -- --nocapture

# 특정 테스트만
cargo test test_audio_manager
```

### 테스트 카테고리별

#### 1. 단위 테스트

```bash
# 라이브러리 단위 테스트
cargo test --lib

# 특정 모듈
cargo test --lib audio::tests
cargo test --lib api::tests
cargo test --lib vad::tests
```

#### 2. 통합 테스트

```bash
# 전체 플로우 테스트
cargo test --test integration_tests

# 특정 통합 테스트
cargo test --test integration_tests full_flow_tests
cargo test --test integration_tests error_recovery_tests
```

#### 3. 속성 기반 테스트 (Property-Based Tests)

```bash
# 모든 속성 테스트
cargo test --test proptest

# 특정 속성
cargo test --test proptest test_recording_mode_consistency
cargo test --test proptest test_audio_data_roundtrip
```

**주의**: 속성 테스트는 시간이 오래 걸릴 수 있습니다 (각 100회 반복).

#### 4. 성능 테스트

```bash
# 성능 요구사항 검증
cargo test --test performance_proptest

# 출력 포함하여 실행 시간 확인
cargo test --test performance_proptest -- --nocapture
```

### 테스트 결과 예시

```
running 90 tests
test api::tests::test_server_client_creation ... ok
test audio::tests::test_audio_manager_creation ... ok
test vad::tests::test_vad_speech_detection ... ok
...
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured
```

---

## 플랫폼별 테스트

### macOS

#### 데스크톱 앱

```bash
# 빌드
cargo build --release

# 실행
./target/release/dioxus-voice-assistant

# 또는 앱 번들 생성
cd macos
chmod +x build_app_bundle.sh
./build_app_bundle.sh

# 앱 실행
open ../target/release/bundle/osx/DioxusVoiceAssistant.app
```

#### iOS (시뮬레이터)

```bash
# iOS 타겟 추가
rustup target add aarch64-apple-ios x86_64-apple-ios

# 빌드 (시뮬레이터용)
cd ios
chmod +x build_ios.sh
./build_ios.sh simulator

# Xcode에서 실행
open ios/DioxusVoiceAssistant.xcodeproj
```

자세한 내용은 `ios/QUICK_START.md` 참조.

### Windows

```bash
# 빌드
cargo build --release --target x86_64-pc-windows-msvc

# 실행
.\target\x86_64-pc-windows-msvc\release\dioxus-voice-assistant.exe
```

자세한 내용은 `WINDOWS_OPTIMIZATION.md` 참조.

### Android

```bash
# Android 타겟 추가
rustup target add aarch64-linux-android armv7-linux-androideabi

# 빌드
cd android
chmod +x build_android.sh
./build_android.sh

# APK 설치 (디바이스 연결 필요)
adb install ../target/android/release/dioxus-voice-assistant.apk
```

자세한 내용은 `android/README.md` 참조.

---

## 수동 테스트 시나리오

### 시나리오 1: Hold 모드 테스트

1. 앱 실행
2. 설정에서 녹음 모드를 "Hold"로 선택
3. 녹음 버튼을 누르고 있는 동안 말하기: "Hello, how are you?"
4. 버튼을 놓으면 녹음 중지
5. 확인 사항:
   - ✅ 버튼을 누르는 동안만 녹음
   - ✅ 음성이 텍스트로 변환됨
   - ✅ AI 응답이 표시됨
   - ✅ 응답이 음성으로 재생됨

### 시나리오 2: Toggle 모드 테스트

1. 설정에서 녹음 모드를 "Toggle"로 선택
2. 녹음 버튼 클릭 (녹음 시작)
3. 말하기: "What's the weather like?"
4. 녹음 버튼 다시 클릭 (녹음 중지)
5. 확인 사항:
   - ✅ 첫 클릭으로 녹음 시작
   - ✅ 두 번째 클릭으로 녹음 중지
   - ✅ 전체 플로우 작동

### 시나리오 3: Auto 모드 테스트

1. 설정에서 녹음 모드를 "Auto"로 선택
2. 말하기 시작 (자동으로 녹음 시작됨)
3. 말을 멈추고 2초 대기
4. 확인 사항:
   - ✅ 음성 감지 시 자동 녹음 시작
   - ✅ 침묵 감지 시 자동 녹음 중지
   - ✅ 배경 소음은 무시됨

### 시나리오 4: 서버 연결 테스트

#### Tailscale 연결
1. Tailscale 설치 및 로그인
2. 서버 Tailscale IP 확인: `tailscale ip`
3. 앱 설정에서:
   - 서버 URL: `http://100.x.x.x:3333`
   - 연결 타입: `Tailscale`
4. 연결 상태 확인

#### 공개 URL 연결
1. 서버 URL: `https://your-domain.com`
2. 연결 타입: `PublicUrl`
3. 연결 상태 확인

#### 로컬 네트워크 연결
1. 서버 URL: `http://localhost:3333`
2. 연결 타입: `LocalNetwork`
3. 연결 상태 확인

### 시나리오 5: 오류 복구 테스트

#### 네트워크 끊김
1. 앱 실행 및 서버 연결
2. 서버 중지 (`Ctrl+C`)
3. 음성 녹음 시도
4. 확인 사항:
   - ✅ 명확한 오류 메시지 표시
   - ✅ 재연결 버튼 표시
5. 서버 재시작
6. 재연결 버튼 클릭
7. 확인 사항:
   - ✅ 자동 재연결 성공

#### 권한 거부
1. 시스템 설정에서 마이크 권한 거부
2. 앱에서 녹음 시도
3. 확인 사항:
   - ✅ 권한 요청 안내 메시지
   - ✅ 설정으로 이동하는 방법 안내

---

## 성능 측정

### 시작 시간 측정

```bash
# 시간 측정과 함께 실행
time cargo run --release

# 예상 결과: < 3초
```

### 녹음 지연 측정

앱 실행 후:
1. 녹음 버튼 클릭
2. 콘솔에서 지연 시간 확인
3. 예상 결과: < 100ms

로그 예시:
```
[Performance] Recording started in 45ms
[Performance] Audio level: 0.75
```

### 메모리 사용량 확인

```bash
# macOS
ps aux | grep dioxus-voice-assistant

# 또는 Activity Monitor 사용
```

---

## 문제 해결

### 문제: 오디오 디바이스를 찾을 수 없음

**증상**: `AudioError::DeviceNotFound`

**해결**:
1. 마이크가 연결되어 있는지 확인
2. 시스템 설정에서 마이크 권한 확인
3. 다른 앱이 마이크를 사용 중인지 확인

### 문제: 서버 연결 실패

**증상**: `ApiError::ConnectionRefused`

**해결**:
1. 서버가 실행 중인지 확인: `curl http://localhost:3333/health`
2. 방화벽 설정 확인
3. 포트가 사용 가능한지 확인: `lsof -i :3333`

### 문제: 테스트 실패

**증상**: 일부 테스트가 실패함

**해결**:
```bash
# 캐시 정리
cargo clean

# 의존성 재빌드
cargo build

# 테스트 재실행
cargo test
```

### 문제: 빌드 오류

**증상**: 컴파일 오류

**해결**:
```bash
# Rust 업데이트
rustup update

# 의존성 업데이트
cargo update

# 특정 플랫폼 타겟 추가
rustup target add x86_64-apple-darwin
```

### 문제: 성능이 느림

**증상**: 시작 시간 > 3초 또는 녹음 지연 > 100ms

**해결**:
1. Release 모드로 빌드: `cargo build --release`
2. 백그라운드 앱 종료
3. 오디오 스트림 사전 초기화 확인
4. 플랫폼별 최적화 문서 참조

---

## 디버그 모드

### 로그 레벨 설정

```bash
# 상세 로그
RUST_LOG=debug cargo run

# 특정 모듈만
RUST_LOG=dioxus_voice_assistant::audio=debug cargo run

# 모든 로그
RUST_LOG=trace cargo run
```

### 로그 출력 예시

```
[2026-02-05T10:30:00Z DEBUG dioxus_voice_assistant::audio] Initializing audio manager
[2026-02-05T10:30:00Z INFO  dioxus_voice_assistant::audio] Audio device found: Built-in Microphone
[2026-02-05T10:30:01Z DEBUG dioxus_voice_assistant::api] Connecting to server: http://localhost:3333
[2026-02-05T10:30:01Z INFO  dioxus_voice_assistant::api] Server connection established
```

---

## 자동화된 테스트 스크립트

프로젝트 루트에 테스트 스크립트를 생성할 수 있습니다:

```bash
#!/bin/bash
# test_all.sh

echo "🧪 Running all tests..."

echo "1️⃣ Unit tests..."
cargo test --lib || exit 1

echo "2️⃣ Integration tests..."
cargo test --test integration_tests || exit 1

echo "3️⃣ Property tests..."
cargo test --test proptest || exit 1

echo "4️⃣ Performance tests..."
cargo test --test performance_proptest || exit 1

echo "5️⃣ Build check..."
cargo build --release || exit 1

echo "✅ All tests passed!"
```

실행:
```bash
chmod +x test_all.sh
./test_all.sh
```

---

## CI/CD 통합

GitHub Actions 예시 (`.github/workflows/test.yml`):

```yaml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
    
    - name: Run tests
      run: cargo test --all-features
    
    - name: Build release
      run: cargo build --release
```

---

## 추가 리소스

- **아키텍처 문서**: `ARCHITECTURE.md`
- **빌드 가이드**: `BUILD.md`
- **플랫폼별 가이드**:
  - Windows: `WINDOWS_OPTIMIZATION.md`
  - macOS: `MACOS_OPTIMIZATION.md`
  - Android: `ANDROID_OPTIMIZATION.md`
  - iOS: `IOS_OPTIMIZATION.md`
- **최종 검증 보고서**: `FINAL_VERIFICATION_REPORT.md`

---

## 피드백 및 문의

테스트 중 문제가 발생하거나 질문이 있으면:
1. 이슈 트래커에 보고
2. 문서 확인
3. 로그 파일 첨부

**Happy Testing! 🚀**
