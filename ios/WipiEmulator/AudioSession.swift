// 오디오 세션 관리. .playback 카테고리라 무음 스위치와 무관하게 게임 사운드가 재생된다
// (게임/미디어 앱의 표준). 게임 화면에서만 활성화하고, 백그라운드/종료 시 비활성화한다.

import AVFoundation

enum AudioSession {
    /// 카테고리 설정 (앱 시작 시 1회). 아직 활성화하진 않는다.
    static func configure() {
        try? AVAudioSession.sharedInstance().setCategory(.playback, mode: .default)
    }

    /// 게임 화면 진입/포그라운드 복귀 시 활성화.
    static func activate() {
        try? AVAudioSession.sharedInstance().setActive(true)
    }

    /// 게임 종료/백그라운드 진입 시 비활성화 (다른 앱 오디오 복구 알림 포함).
    static func deactivate() {
        try? AVAudioSession.sharedInstance().setActive(false, options: .notifyOthersOnDeactivation)
    }
}
