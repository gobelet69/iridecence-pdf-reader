import AppKit
#if canImport(IridescenceCore)
import IridescenceCore
#endif

@MainActor
private func launchApplication() {
    let app = NSApplication.shared
    app.setActivationPolicy(.regular)
    let delegate = AppDelegate()
    app.delegate = delegate
    app.activate(ignoringOtherApps: true)
    app.run()
}

launchApplication()
