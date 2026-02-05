# 요구사항 문서

## 소개

ClawToTalk 웹 애플리케이션을 Dioxus 최신 버전을 사용하여 Windows, Mac, Android, iOS용 크로스 플랫폼 네이티브 애플리케이션으로 변환합니다. 기존의 모든 음성 인터페이스 기능을 유지하면서 네이티브 앱의 장점을 활용합니다.

## 용어집

- **Voice_Assistant**: AI 어시스턴트와 음성으로 상호작용하는 메인 시스템
- **Recording_Mode**: 음성 녹음 방식 (Hold, Toggle, Auto)
- **Speech_Processor**: 음성-텍스트 변환을 담당하는 컴포넌트
- **AI_Client**: AI API와 통신하는 클라이언트
- **TTS_Engine**: 텍스트-음성 변환 엔진
- **VAD_Detector**: 음성 활동 감지기 (Voice Activity Detector)
- **Native_App**: Dioxus로 구축된 크로스 플랫폼 네이티브 애플리케이션

## 요구사항

### 요구사항 1: 크로스 플랫폼 네이티브 애플리케이션

**사용자 스토리:** 개발자로서, 단일 코드베이스로 Windows, Mac, Android, iOS 앱을 배포하고 싶습니다.

#### 승인 기준

1. THE Native_App SHALL 최신 Dioxus 프레임워크를 사용하여 구축되어야 합니다
2. WHEN 앱이 빌드될 때, THE Native_App SHALL Windows, Mac, Android, iOS 플랫폼에서 실행 가능한 바이너리를 생성해야 합니다
3. THE Native_App SHALL 각 플랫폼의 네이티브 UI 가이드라인을 준수해야 합니다
4. THE Native_App SHALL 플랫폼별 권한 요청을 적절히 처리해야 합니다

### 요구사항 2: 음성 녹음 및 처리

**사용자 스토리:** 사용자로서, 다양한 방식으로 음성을 녹음하고 AI와 대화하고 싶습니다.

#### 승인 기준

1. THE Voice_Assistant SHALL 3가지 Recording_Mode를 지원해야 합니다 (Hold, Toggle, Auto)
2. WHEN Hold 모드가 선택되면, THE Voice_Assistant SHALL 버튼을 누르고 있는 동안만 녹음해야 합니다
3. WHEN Toggle 모드가 선택되면, THE Voice_Assistant SHALL 첫 번째 클릭으로 녹음 시작, 두 번째 클릭으로 녹음 종료해야 합니다
4. WHEN Auto 모드가 선택되면, THE Voice_Assistant SHALL VAD_Detector를 사용하여 자동으로 음성을 감지하고 녹음해야 합니다
5. THE Voice_Assistant SHALL 각 플랫폼의 마이크 권한을 요청하고 관리해야 합니다

### 요구사항 3: 음성-텍스트 변환

**사용자 스토리:** 사용자로서, 내 음성이 정확하게 텍스트로 변환되기를 원합니다.

#### 승인 기준

1. WHEN 음성 녹음이 완료되면, THE Speech_Processor SHALL OpenAI Whisper API를 사용하여 음성을 텍스트로 변환해야 합니다
2. THE Speech_Processor SHALL 다양한 언어를 지원해야 합니다
3. WHEN 변환이 실패하면, THE Speech_Processor SHALL 사용자에게 오류 메시지를 표시해야 합니다
4. THE Speech_Processor SHALL 변환된 텍스트를 사용자 인터페이스에 표시해야 합니다

### 요구사항 4: AI 응답 생성

**사용자 스토리:** 사용자로서, AI 어시스턴트로부터 지능적인 응답을 받고 싶습니다.

#### 승인 기준

1. THE AI_Client SHALL OpenClaw 또는 Claude API를 통해 AI 응답을 생성해야 합니다
2. WHEN 텍스트 입력이 제공되면, THE AI_Client SHALL 적절한 AI 응답을 요청하고 받아야 합니다
3. THE AI_Client SHALL API 키 설정을 안전하게 관리해야 합니다
4. WHEN API 호출이 실패하면, THE AI_Client SHALL 사용자에게 오류 상태를 알려야 합니다

### 요구사항 5: 텍스트-음성 변환

**사용자 스토리:** 사용자로서, AI의 응답을 음성으로 들을 수 있기를 원합니다.

#### 승인 기준

1. THE TTS_Engine SHALL ElevenLabs API를 사용하여 텍스트를 음성으로 변환해야 합니다
2. WHEN AI 응답이 생성되면, THE TTS_Engine SHALL 자동으로 음성 변환을 시작해야 합니다
3. THE TTS_Engine SHALL 음성 재생을 일시정지, 재개, 중지할 수 있어야 합니다
4. THE TTS_Engine SHALL 각 플랫폼의 오디오 출력 시스템과 통합되어야 합니다

### 요구사항 6: 실시간 음성 활동 감지

**사용자 스토리:** 사용자로서, Auto 모드에서 자동으로 음성이 감지되기를 원합니다.

#### 승인 기준

1. THE VAD_Detector SHALL 실시간으로 마이크 입력을 모니터링해야 합니다
2. WHEN 음성 활동이 감지되면, THE VAD_Detector SHALL 자동으로 녹음을 시작해야 합니다
3. WHEN 음성 활동이 중단되면, THE VAD_Detector SHALL 설정된 지연 시간 후 녹음을 종료해야 합니다
4. THE VAD_Detector SHALL 배경 소음과 실제 음성을 구분할 수 있어야 합니다

### 요구사항 7: 사용자 인터페이스

**사용자 스토리:** 사용자로서, 직관적이고 반응형인 인터페이스를 원합니다.

#### 승인 기준

1. THE Native_App SHALL 모든 플랫폼에서 일관된 사용자 경험을 제공해야 합니다
2. THE Native_App SHALL 터치 및 마우스 입력을 모두 지원해야 합니다
3. WHEN 녹음 중일 때, THE Native_App SHALL 시각적 피드백을 제공해야 합니다
4. THE Native_App SHALL 대화 기록을 표시하고 스크롤 가능해야 합니다
5. THE Native_App SHALL 설정 화면에서 API 키와 모드를 구성할 수 있어야 합니다

### 요구사항 8: 설정 및 구성

**사용자 스토리:** 사용자로서, 앱의 동작을 내 필요에 맞게 구성하고 싶습니다.

#### 승인 기준

1. THE Native_App SHALL API 키 설정 인터페이스를 제공해야 합니다 (OpenAI, Claude/OpenClaw, ElevenLabs)
2. THE Native_App SHALL 녹음 모드 선택 옵션을 제공해야 합니다
3. THE Native_App SHALL 음성 감지 민감도 조절 옵션을 제공해야 합니다
4. THE Native_App SHALL 설정을 로컬에 안전하게 저장해야 합니다
5. WHEN 설정이 변경되면, THE Native_App SHALL 즉시 새 설정을 적용해야 합니다

### 요구사항 9: 오류 처리 및 복구

**사용자 스토리:** 사용자로서, 오류가 발생했을 때 명확한 피드백을 받고 복구할 수 있기를 원합니다.

#### 승인 기준

1. WHEN 네트워크 오류가 발생하면, THE Native_App SHALL 사용자에게 명확한 오류 메시지를 표시해야 합니다
2. WHEN API 키가 유효하지 않으면, THE Native_App SHALL 설정 화면으로 안내해야 합니다
3. WHEN 마이크 권한이 거부되면, THE Native_App SHALL 권한 요청 방법을 안내해야 합니다
4. THE Native_App SHALL 일시적 오류에 대해 자동 재시도 메커니즘을 제공해야 합니다

### 요구사항 10: 성능 및 최적화

**사용자 스토리:** 사용자로서, 빠르고 반응성 좋은 앱을 원합니다.

#### 승인 기준

1. THE Native_App SHALL 앱 시작 시간이 3초 이내여야 합니다
2. THE Native_App SHALL 음성 녹음 시작 지연이 100ms 이내여야 합니다
3. THE Native_App SHALL 메모리 사용량을 효율적으로 관리해야 합니다
4. THE Native_App SHALL 배터리 사용량을 최적화해야 합니다 (모바일 플랫폼)