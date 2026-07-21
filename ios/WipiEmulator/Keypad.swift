// 터치 키패드 — Android MainActivity의 Keypad/KeyButton 레이아웃 이식.
// 키 반복은 Rust 세션이 처리하므로 여기서는 down/up만 보낸다.

import SwiftUI

private let keypadRows: [[(key: String, label: String)]] = [
    [("SOFT_L", "◁"), ("UP", "▲"), ("SOFT_R", "▷")],
    [("LEFT", "◀"), ("OK", "OK"), ("RIGHT", "▶")],
    [("CALL", "📞"), ("DOWN", "▼"), ("CLR", "CLR")],
    [("1", "1"), ("2", "2"), ("3", "3")],
    [("4", "4"), ("5", "5"), ("6", "6")],
    [("7", "7"), ("8", "8"), ("9", "9")],
    [("*", "*"), ("0", "0"), ("#", "#")],
]

struct Keypad: View {
    let haptics: Haptics

    var body: some View {
        VStack(spacing: 4) {
            ForEach(keypadRows.indices, id: \.self) { rowIndex in
                HStack(spacing: 4) {
                    ForEach(keypadRows[rowIndex], id: \.key) { entry in
                        KeyButton(key: entry.key, label: entry.label, haptics: haptics)
                    }
                }
            }
        }
    }
}

struct KeyButton: View {
    let key: String
    let label: String
    let haptics: Haptics

    @State private var pressed = false
    @AppStorage(SettingsKey.keypadHaptics) private var keypadHaptics = true

    var body: some View {
        Text(label)
            .font(.system(size: 16))
            .foregroundColor(.white)
            .frame(maxWidth: .infinity)
            .frame(height: 40)
            .background(Color(red: 0.25, green: 0.25, blue: 0.25).opacity(pressed ? 0.6 : 1.0))
            .cornerRadius(8)
            .gesture(
                DragGesture(minimumDistance: 0)
                    .onChanged { _ in
                        if !pressed {
                            pressed = true
                            if keypadHaptics { haptics.tap() }
                            WipiCore.keyDown(key)
                        }
                    }
                    .onEnded { _ in
                        pressed = false
                        WipiCore.keyUp(key)
                    }
            )
    }
}
