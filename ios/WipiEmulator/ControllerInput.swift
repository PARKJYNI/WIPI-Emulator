// 게임 컨트롤러(MFi/DualShock/Xbox)와 하드웨어 키보드 입력을 에뮬 키로 매핑.
// 키보드 배치는 Android hardwareKeyMap(wie_cli와 동일)을 그대로 따른다:
// 123/QWE/ASD/ZXC → 키패드 123/456/789/*0#, 방향키, Enter·Space→OK, Backspace→CLR, Shift→소프트키.

import GameController

final class ControllerInput {
    private var observers: [NSObjectProtocol] = []

    func start() {
        attachAll()

        observers.append(NotificationCenter.default.addObserver(
            forName: .GCControllerDidConnect, object: nil, queue: .main
        ) { [weak self] _ in self?.attachAll() })

        observers.append(NotificationCenter.default.addObserver(
            forName: .GCKeyboardDidConnect, object: nil, queue: .main
        ) { [weak self] _ in self?.attachAll() })
    }

    func stop() {
        observers.forEach(NotificationCenter.default.removeObserver)
        observers.removeAll()

        for controller in GCController.controllers() {
            controller.extendedGamepad?.valueChangedHandler = nil
        }
        GCKeyboard.coalesced?.keyboardInput?.keyChangedHandler = nil
    }

    private func attachAll() {
        for controller in GCController.controllers() {
            attach(controller)
        }
        attachKeyboard()
    }

    // MARK: - 게임패드

    /// 버튼별 이전 눌림 상태 (down/up 전이 감지용)
    private var buttonStates: [String: Bool] = [:]

    private func attach(_ controller: GCController) {
        guard let gamepad = controller.extendedGamepad else { return }

        gamepad.valueChangedHandler = { [weak self] gamepad, _ in
            guard let self else { return }
            // 방향: dpad + 왼쪽 스틱을 같은 키로
            let up = gamepad.dpad.up.isPressed || gamepad.leftThumbstick.up.value > 0.5
            let down = gamepad.dpad.down.isPressed || gamepad.leftThumbstick.down.value > 0.5
            let left = gamepad.dpad.left.isPressed || gamepad.leftThumbstick.left.value > 0.5
            let right = gamepad.dpad.right.isPressed || gamepad.leftThumbstick.right.value > 0.5

            self.transition("UP", pressed: up)
            self.transition("DOWN", pressed: down)
            self.transition("LEFT", pressed: left)
            self.transition("RIGHT", pressed: right)

            self.transition("OK", pressed: gamepad.buttonA.isPressed)
            self.transition("CLR", pressed: gamepad.buttonB.isPressed)
            self.transition("*", pressed: gamepad.buttonX.isPressed)
            self.transition("#", pressed: gamepad.buttonY.isPressed)
            self.transition("SOFT_L", pressed: gamepad.leftShoulder.isPressed)
            self.transition("SOFT_R", pressed: gamepad.rightShoulder.isPressed)
        }
    }

    private func transition(_ key: String, pressed: Bool) {
        let was = buttonStates[key] ?? false
        guard was != pressed else { return }
        buttonStates[key] = pressed

        if pressed {
            WipiCore.keyDown(key)
        } else {
            WipiCore.keyUp(key)
        }
    }

    // MARK: - 하드웨어 키보드

    private static let keyboardMap: [GCKeyCode: String] = [
        .upArrow: "UP", .downArrow: "DOWN", .leftArrow: "LEFT", .rightArrow: "RIGHT",
        .returnOrEnter: "OK", .spacebar: "OK", .keypadEnter: "OK",
        .one: "1", .two: "2", .three: "3",
        .keyQ: "4", .keyW: "5", .keyE: "6",
        .keyA: "7", .keyS: "8", .keyD: "9",
        .keyZ: "*", .keyX: "0", .keyC: "#",
        .four: "4", .five: "5", .six: "6",
        .seven: "7", .eight: "8", .nine: "9", .zero: "0",
        .deleteOrBackspace: "CLR",
        .leftShift: "SOFT_L", .rightShift: "SOFT_R",
        .F1: "CALL", .F2: "HANGUP",
    ]

    private func attachKeyboard() {
        GCKeyboard.coalesced?.keyboardInput?.keyChangedHandler = { _, _, keyCode, pressed in
            guard let key = Self.keyboardMap[keyCode] else { return }
            if pressed {
                WipiCore.keyDown(key)
            } else {
                WipiCore.keyUp(key)
            }
        }
    }
}
