import Foundation

public enum LogCategory: String, Sendable, CaseIterable {
    case ui = "UI"
    case workspace = "WORKSPACE"
    case icloud = "ICLOUD"
    case pdf = "PDF"
    case error = "ERROR"
}

public struct LogEntry: Sendable, Equatable {
    public let timestamp: Date
    public let category: LogCategory
    public let message: String
    public let formattedLine: String

    public init(timestamp: Date = Date(), category: LogCategory, message: String) {
        self.timestamp = timestamp
        self.category = category
        self.message = message
        let secs = UInt64(max(0, timestamp.timeIntervalSince1970))
        self.formattedLine = "[\(secs)] [\(category.rawValue)] \(message)"
    }
}

/// Thread-safe bounded ring buffer logger (capped at `maxBufferSize = 2000` per Rule 3.2).
public final class AppLogger: @unchecked Sendable {
    public static let shared = AppLogger()
    public static let maxBufferSize: Int = 2000

    private let lock = NSLock()
    private var buffer: [LogEntry] = []
    private var isPausedState: Bool = false
    private var observerCallback: (@Sendable () -> Void)?

    public init() {}

    public var isPaused: Bool {
        get {
            lock.lock()
            defer { lock.unlock() }
            return isPausedState
        }
        set {
            lock.lock()
            isPausedState = newValue
            lock.unlock()
        }
    }

    public func setObserver(_ callback: (@Sendable () -> Void)?) {
        lock.lock()
        observerCallback = callback
        lock.unlock()
    }

    public func log(_ category: LogCategory, _ message: String) {
        let entry = LogEntry(category: category, message: message)
        var callbackToFire: (@Sendable () -> Void)?

        lock.lock()
        if !isPausedState {
            if buffer.count >= Self.maxBufferSize {
                let overflow = (buffer.count - Self.maxBufferSize) + 1
                buffer.removeFirst(overflow)
            }
            buffer.append(entry)
            callbackToFire = observerCallback
        }
        lock.unlock()

        callbackToFire?()
    }

    public func entries() -> [LogEntry] {
        lock.lock()
        defer { lock.unlock() }
        return buffer
    }

    public func allLogsString() -> String {
        lock.lock()
        let lines = buffer.map(\.formattedLine)
        lock.unlock()
        return lines.joined(separator: "\n")
    }

    public func clear() {
        var callbackToFire: (@Sendable () -> Void)?
        lock.lock()
        buffer.removeAll(keepingCapacity: true)
        callbackToFire = observerCallback
        lock.unlock()
        callbackToFire?()
    }

    public func count() -> Int {
        lock.lock()
        defer { lock.unlock() }
        return buffer.count
    }
}
