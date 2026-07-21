// 게임의 vibrate(duration_ms, intensity) 요청을 Taptic Engine으로 재생.
// Core Haptics는 duration/intensity를 실제로 반영할 수 있어 단순 진동보다 표현력이 좋다.
// 지원 안 되는 기기(구형/시뮬레이터)에서는 조용히 무시된다.

import CoreHaptics

final class Haptics {
    private var engine: CHHapticEngine?
    private let supported = CHHapticEngine.capabilitiesForHardware().supportsHaptics

    /// 키패드 탭용 캐시 플레이어 (연타 대응 — 매번 makePlayer 하지 않음)
    private var tapPlayer: CHHapticPatternPlayer?

    init() {
        guard supported else { return }
        engine = try? CHHapticEngine()
        // 오디오 인터럽션 등으로 엔진이 멈추면 자동 복구
        engine?.resetHandler = { [weak self] in try? self?.engine?.start() }
        engine?.stoppedHandler = { _ in }
        try? engine?.start()

        // 키패드 탭: 짧고 가벼운 트랜지언트.
        // UIImpactFeedbackGenerator는 CHHapticEngine 가동 중 무시되는 충돌이 있어 같은 엔진으로 재생한다.
        let tap = CHHapticEvent(
            eventType: .hapticTransient,
            parameters: [
                CHHapticEventParameter(parameterID: .hapticIntensity, value: 0.5),
                CHHapticEventParameter(parameterID: .hapticSharpness, value: 0.7),
            ],
            relativeTime: 0
        )
        if let pattern = try? CHHapticPattern(events: [tap], parameters: []) {
            tapPlayer = try? engine?.makePlayer(with: pattern)
        }
    }

    /// 키패드 키 다운 햅틱 (게임 진동과 독립적인 호스트 UI 피드백)
    func tap() {
        try? tapPlayer?.start(atTime: CHHapticTimeImmediate)
    }

    /// 게임이 지정한 duration/intensity를 재현하되, 사용자 강도 배율(scale)을 곱한다.
    /// - intensity 0~100(= level 0~10 × 10)을 세기에 반영. WIPI API는 진동 세기를 level로
    ///   정의하므로 단계 구분을 살린다 (게임 개발 의도).
    /// - Taptic Engine에서 낮은 값이 안 느껴지는 문제만 0.5 + 0.5x 선형 보정으로 바닥을 올린다.
    /// - scale은 설정의 "진동 세기"(0~1.5). 게임 세기 위에 곱하는 전역 게인 (RetroArch rumble gain과 동일).
    /// - duration은 지정 시간만큼 연속 진동. duration/intensity/scale이 0이면 건너뛴다.
    func play(durationMs: UInt64, intensity: UInt8, scale: Float) {
        guard supported, let engine else { return }
        guard durationMs > 0, intensity > 0, scale > 0 else { return }

        let level = Float(intensity) / 100.0 // API level(0~10)을 0~1로
        let base = 0.5 + 0.5 * level // level에 비례, 최소 0.55 보장 (개발 의도)
        let hapticIntensity = min(base * scale, 1.0) // 사용자 강도 배율 적용
        let intensityParam = CHHapticEventParameter(parameterID: .hapticIntensity, value: hapticIntensity)
        // 피처폰 편심모터의 둔탁한 진동에 가깝게 sharpness는 낮게
        let sharpnessParam = CHHapticEventParameter(parameterID: .hapticSharpness, value: 0.3)

        let duration = min(TimeInterval(durationMs) / 1000.0, 5.0) // 상한 5초 (안전)
        let event = CHHapticEvent(
            eventType: .hapticContinuous,
            parameters: [intensityParam, sharpnessParam],
            relativeTime: 0,
            duration: duration
        )

        do {
            let pattern = try CHHapticPattern(events: [event], parameters: [])
            let player = try engine.makePlayer(with: pattern)
            try player.start(atTime: CHHapticTimeImmediate)
        } catch {
            // 진동 실패는 게임 진행에 영향 없음 — 무시
        }
    }
}
