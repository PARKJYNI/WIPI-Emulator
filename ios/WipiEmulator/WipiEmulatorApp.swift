import SwiftUI

@main
struct WipiEmulatorApp: App {
    init() {
        WipiCore.initialize()
        AudioSession.configure()
    }

    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}
