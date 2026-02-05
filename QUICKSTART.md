# 빠른 시작 가이드 (Quick Start Guide)

5분 안에 Dioxus Voice Assistant를 실행하고 테스트하는 방법입니다.

---

## 🎯 목표

이 가이드를 따라하면:
1. ✅ 모든 테스트가 통과하는지 확인
2. ✅ 목 서버를 실행 (API 키 불필요)
3. ✅ 클라이언트 앱을 실행
4. ✅ 음성 녹음 및 재생 테스트

**소요 시간**: 약 5분

---

## 📋 사전 요구사항

- **Rust** (1.75 이상): https://rustup.rs/
- **Node.js** (18 이상): https://nodejs.org/

확인:
```bash
rustc --version  # rust 1.75.0 이상
node --version   # v18.0.0 이상
```

---

## 🚀 단계별 가이드

### 1단계: 프로젝트 확인 (30초)

```bash
# 현재 디렉토리 확인
pwd
# 출력: /path/to/dioxus-voice-assistant

# 파일 확인
ls -la
# Cargo.toml, server.js, quick_test.sh 등이 보여야 함
```

### 2단계: 자동 테스트 실행 (2분)

```bash
# 실행 권한 부여 (처음 한 번만)
chmod +x quick_test.sh

# 모든 테스트 실행
./quick_test.sh
```

**예상 출력**:
```
🚀 Dioxus Voice Assistant - Quick Test
======================================

▶ Checking Rust installation...
✓ Rust is installed: rustc 1.75.0

▶ Step 1: Running unit tests...
✓ Unit tests passed

▶ Step 2: Running integration tests...
✓ Integration tests passed

▶ Step 3: Running property-based tests...
✓ Property-based tests passed

▶ Step 4: Running performance tests...
✓ Performance tests passed

▶ Step 5: Building release binary...
✓ Release build successful

▶ Step 6: Testing mock server...
✓ Mock server is running

======================================
✓ All tests passed!
```

### 3단계: 서버 시작 (30초)

새 터미널 창을 열고:

```bash
# .env 파일 생성 (목 모드)
cat > .env << EOF
MOCK_MODE=true
PORT=3333
BOT_NAME=TestBot
USE_OPENCLAW=false
EOF

# 의존성 설치 (처음 한 번만)
npm install

# 서버 시작
npm start
```

**예상 출력**:
```
🎙️ TestBot voice server running on http://localhost:3333
   Mode: MOCK (no API keys required)
   Session: voice-clawtotalk

   Test with: curl http://localhost:3333/health
```

**서버 테스트**:
```bash
# 다른 터미널에서
curl http://localhost:3333/health

# 예상 응답:
# {"status":"ok","mode":"mock","time":"2026-02-05T...","services":{"stt":"mock","chat":"mock","tts":"mock"}}
```

### 4단계: 클라이언트 앱 실행 (1분)

원래 터미널로 돌아가서:

```bash
# 앱 실행
cargo run --release
```

**첫 실행 시**: 컴파일에 1-2분 소요될 수 있습니다.

**앱이 실행되면**:
1. 설정 화면이 나타남
2. 다음과 같이 설정:
   - **서버 URL**: `http://localhost:3333`
   - **연결 타입**: `LocalNetwork`
   - **녹음 모드**: `Hold` (처음 테스트 시 추천)
3. **저장** 버튼 클릭

### 5단계: 음성 테스트 (1분)

#### Hold 모드 테스트

1. **녹음 버튼을 누르고 있으면서** 말하기:
   ```
   "Hello, this is a test"
   ```

2. **버튼을 놓으면** 녹음 중지

3. **확인 사항**:
   - ✅ 녹음 중 버튼 색상 변경
   - ✅ 텍스트가 화면에 표시: "Hello, this is a test"
   - ✅ AI 응답 표시: "This is a mock response from TestBot..."
   - ✅ (목 모드에서는 실제 음성 재생 없음)

#### Toggle 모드 테스트

1. 설정에서 녹음 모드를 **Toggle**로 변경
2. 버튼 **클릭** (녹음 시작)
3. 말하기
4. 버튼 다시 **클릭** (녹음 중지)
5. 응답 확인

#### Auto 모드 테스트

1. 설정에서 녹음 모드를 **Auto**로 변경
2. 말하기 시작 (자동으로 녹음 시작)
3. 2초간 침묵 (자동으로 녹음 중지)
4. 응답 확인

---

## ✅ 성공 확인

다음이 모두 작동하면 성공입니다:

- [x] 모든 테스트 통과 (90/90)
- [x] 서버가 실행 중 (http://localhost:3333/health 응답)
- [x] 클라이언트 앱이 실행됨
- [x] 서버에 연결됨 (연결 상태 표시)
- [x] 음성 녹음 작동
- [x] 텍스트 변환 표시 (목 모드)
- [x] AI 응답 표시

---

## 🎉 다음 단계

### 실제 API 사용하기

목 모드 대신 실제 API를 사용하려면:

1. **API 키 발급**:
   - OpenAI: https://platform.openai.com/api-keys
   - ElevenLabs: https://elevenlabs.io/
   - Anthropic: https://console.anthropic.com/

2. **.env 파일 수정**:
   ```bash
   MOCK_MODE=false
   OPENAI_API_KEY=sk-proj-your-key-here
   ELEVENLABS_API_KEY=your-key-here
   ELEVENLABS_VOICE_ID=SAz9YHcvj6GT2YYXdXww
   ANTHROPIC_API_KEY=sk-ant-your-key-here
   USE_OPENCLAW=false
   PORT=3333
   BOT_NAME=Assistant
   ```

3. **서버 재시작**:
   ```bash
   # Ctrl+C로 서버 중지
   npm start
   ```

### 다른 플랫폼에서 실행

#### iOS
```bash
cd ios
./build_ios.sh simulator
# Xcode에서 실행
```

#### Android
```bash
cd android
./build_android.sh
adb install ../target/android/release/dioxus-voice-assistant.apk
```

#### Windows
```bash
cargo build --release --target x86_64-pc-windows-msvc
```

자세한 내용은 각 플랫폼별 최적화 문서를 참조하세요.

---

## 🐛 문제 해결

### 문제: 테스트 실패

```bash
# 캐시 정리 후 재시도
cargo clean
cargo test
```

### 문제: 서버가 시작되지 않음

```bash
# 포트 사용 확인
lsof -i :3333

# 다른 포트 사용
PORT=8080 npm start
```

### 문제: 앱이 서버에 연결되지 않음

1. 서버가 실행 중인지 확인: `curl http://localhost:3333/health`
2. 방화벽 설정 확인
3. 앱 설정에서 서버 URL 확인

### 문제: 오디오 디바이스를 찾을 수 없음

1. 마이크가 연결되어 있는지 확인
2. 시스템 설정에서 마이크 권한 확인
3. 다른 앱이 마이크를 사용 중인지 확인

---

## 📚 더 알아보기

- **상세 테스트 가이드**: [LOCAL_TESTING_GUIDE.md](LOCAL_TESTING_GUIDE.md)
- **아키텍처 설명**: [ARCHITECTURE.md](ARCHITECTURE.md)
- **빌드 가이드**: [BUILD.md](BUILD.md)
- **최종 검증 보고서**: [FINAL_VERIFICATION_REPORT.md](FINAL_VERIFICATION_REPORT.md)

---

## 💬 도움이 필요하신가요?

- 이슈 트래커에 문제 보고
- 문서 확인
- 로그 파일 첨부

**Happy Testing! 🚀**
