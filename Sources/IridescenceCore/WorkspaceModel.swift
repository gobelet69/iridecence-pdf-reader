import AppKit
import Foundation
import PDFKit

/// Explicit 7-state iCloud & file availability model (Section 5.2).
public enum ICloudStatus: UInt8, Sendable, Equatable {
    case local = 0
    case cloudOnly = 1
    case downloadRequested = 3
    case downloading = 2
    case downloadedVerified = 5
    case failed = 4
    case unavailable = 6

    public var isReadyToOpen: Bool {
        self == .local || self == .downloadedVerified
    }

    public var isInFlight: Bool {
        self == .downloadRequested || self == .downloading
    }

    public var isBadgeVisible: Bool {
        self != .local
    }

    public var badgeText: String {
        switch self {
        case .local: return ""
        case .cloudOnly: return "iCloud"
        case .downloadRequested: return "Requested…"
        case .downloading: return "Downloading…"
        case .downloadedVerified: return "Downloaded"
        case .failed: return "Retry"
        case .unavailable: return "Unavailable"
        }
    }
}

/// Pluggable backend protocol for iCloud file status inspection and download initiation,
/// allowing both real macOS `FileManager` ubiquity operations and deterministic unit/Visual-QA testing.
public protocol ICloudProviderBackend: Sendable {
    func inspectStatus(for url: URL) -> (status: ICloudStatus, progress: Double?, errorMessage: String?)
    func startUbiquitousDownload(for url: URL) throws
}

public struct SystemICloudProviderBackend: ICloudProviderBackend, Sendable {
    public init() {}

    public static func inspectSystemStatus(for url: URL) -> (status: ICloudStatus, progress: Double?, errorMessage: String?) {
        let rawName = url.lastPathComponent
        let isPlaceholder = rawName.hasPrefix(".") && rawName.lowercased().hasSuffix(".icloud")
        let logicalURL = ICloudCoordinator.logicalURL(for: url)
        let placeholderURL = ICloudCoordinator.placeholderURL(for: logicalURL)

        let fm = FileManager.default
        let logicalExists = fm.fileExists(atPath: logicalURL.path)
        let placeholderExists = fm.fileExists(atPath: placeholderURL.path)

        if !logicalExists && !placeholderExists && !fm.fileExists(atPath: url.path) {
            return (.unavailable, nil, "File is no longer available at \(logicalURL.lastPathComponent).")
        }

        let targetInspectionURL = placeholderExists ? placeholderURL : (logicalExists ? logicalURL : url)
        let keys: Set<URLResourceKey> = [
            .isUbiquitousItemKey,
            .ubiquitousItemDownloadingStatusKey,
            .ubiquitousItemIsDownloadingKey,
            .ubiquitousItemDownloadingErrorKey,
            .isReadableKey
        ]

        if let values = try? targetInspectionURL.resourceValues(forKeys: keys) {
            if let err = values.ubiquitousItemDownloadingError {
                return (.failed, nil, err.localizedDescription)
            }
            if values.ubiquitousItemIsDownloading == true {
                return (.downloading, nil, nil)
            }
            if let dlStatus = values.ubiquitousItemDownloadingStatus {
                if dlStatus == .notDownloaded {
                    return (.cloudOnly, nil, nil)
                }
                if dlStatus == .current || dlStatus == .downloaded {
                    if logicalExists {
                        return (.local, 1.0, nil)
                    }
                }
            }
        }

        if isPlaceholder || (placeholderExists && !logicalExists) {
            return (.cloudOnly, nil, nil)
        }

        if logicalExists && !fm.isReadableFile(atPath: logicalURL.path) {
            return (.unavailable, nil, "Access denied for “\(logicalURL.lastPathComponent)”.")
        }

        return (.local, 1.0, nil)
    }

    public func inspectStatus(for url: URL) -> (status: ICloudStatus, progress: Double?, errorMessage: String?) {
        Self.inspectSystemStatus(for: url)
    }

    public func startUbiquitousDownload(for url: URL) throws {
        let fm = FileManager.default
        let logical = ICloudCoordinator.logicalURL(for: url)
        let placeholder = ICloudCoordinator.placeholderURL(for: logical)

        if fm.fileExists(atPath: placeholder.path) {
            try fm.startDownloadingUbiquitousItem(at: placeholder)
        } else {
            try fm.startDownloadingUbiquitousItem(at: url)
        }
    }
}

@MainActor
public enum ICloudCoordinator {
    /// Converts a `.Filename.pdf.icloud` placeholder URL into its logical materialized `Filename.pdf` URL.
    nonisolated public static func logicalURL(for url: URL) -> URL {
        let name = url.lastPathComponent
        if name.hasPrefix(".") && name.lowercased().hasSuffix(".icloud") {
            let stripped = String(name.dropFirst().dropLast(".icloud".count))
            if !stripped.isEmpty {
                return url.deletingLastPathComponent().appendingPathComponent(stripped)
            }
        }
        return url
    }

    /// Returns the `.Filename.pdf.icloud` placeholder URL corresponding to a logical `Filename.pdf` URL.
    nonisolated public static func placeholderURL(for url: URL) -> URL {
        let logical = logicalURL(for: url)
        let name = logical.lastPathComponent
        return logical.deletingLastPathComponent().appendingPathComponent(".\(name).icloud")
    }

    /// Canonical key used to deduplicate and track an item across `.icloud` placeholder -> `.pdf` materialization transitions.
    nonisolated public static func canonicalLogicalKey(for url: URL) -> String {
        UIConstants.canonicalPath(logicalURL(for: url).path)
    }

    public static func status(for url: URL) -> ICloudStatus {
        ICloudDownloadManager.shared.effectiveStatus(for: url)
    }

    @discardableResult
    public static func startDownload(for url: URL) -> Bool {
        _ = ICloudDownloadManager.shared.requestDownload(for: url)
        return true
    }

    /// Verifies that a materialized PDF exists on disk, is non-empty, and can be parsed by `PDFDocument`.
    public static func verifyDownloadedPDF(at url: URL) -> URL? {
        let logical = logicalURL(for: url)
        guard FileManager.default.fileExists(atPath: logical.path),
              FileManager.default.isReadableFile(atPath: logical.path),
              let attrs = try? FileManager.default.attributesOfItem(atPath: logical.path),
              let size = attrs[.size] as? UInt64,
              size >= 32,
              PDFDocument(url: logical) != nil else {
            return nil
        }
        return logical
    }
}

/// Active download record managed by `ICloudDownloadManager`.
public struct ICloudDownloadSession: Sendable, Equatable {
    public let canonicalKey: String
    public let requestedURL: URL
    public let logicalURL: URL
    public var status: ICloudStatus
    public var progress: Double?
    public var errorMessage: String?
    public let requestedAt: Date
}

/// Coordinates iCloud PDF downloads, deduplication, verification, external file-system observation,
/// and automatic opening upon download completion without clobbering newer user navigation.
@MainActor
public final class ICloudDownloadManager {
    public static let shared = ICloudDownloadManager()

    public var backend: ICloudProviderBackend = SystemICloudProviderBackend()
    public private(set) var sessions: [String: ICloudDownloadSession] = [:]

    /// The canonical logical key of the file the user most recently clicked to auto-open once downloaded.
    /// Cleared or updated when the user intentionally opens another document in the meantime.
    public private(set) var pendingAutoOpenKey: String?

    /// Callbacks for UI synchronization
    public var onStateChanged: ((ICloudDownloadSession) -> Void)?
    public var onAutoOpenReady: ((URL) -> Void)?
    public var onBackgroundDownloadReady: ((URL) -> Void)?

    private var monitorTasks: [String: Task<Void, Never>] = [:]
    public var pollIntervalNanoseconds: UInt64 = 250_000_000 // 250ms while a download is actively in flight
    public var maxWaitLoops: Int = 120 // 30 seconds before surfacing a retryable timeout state

    public init() {}

    /// Notifies the download manager that the user intentionally opened a different document,
    /// preventing any older in-flight download from unexpectedly replacing the active document on completion.
    public func userDidNavigateToDocument(at url: URL) {
        let navKey = ICloudCoordinator.canonicalLogicalKey(for: url)
        if let pending = pendingAutoOpenKey, pending != navKey {
            AppLogger.shared.log(.icloud, "Cleared auto-open target \(pending) because user navigated to \(navKey)")
            pendingAutoOpenKey = nil
        }
    }

    public func session(for url: URL) -> ICloudDownloadSession? {
        let key = ICloudCoordinator.canonicalLogicalKey(for: url)
        return sessions[key]
    }

    public func effectiveStatus(for url: URL) -> ICloudStatus {
        let key = ICloudCoordinator.canonicalLogicalKey(for: url)
        if let active = sessions[key] {
            return active.status
        }
        return backend.inspectStatus(for: url).status
    }

    /// Initiates or resumes an iCloud download for `url`.
    /// Deduplicates in-flight requests and automatically opens the PDF when verified if `autoOpenWhenReady` is true.
    @discardableResult
    public func requestDownload(for url: URL, autoOpenWhenReady: Bool = true) -> ICloudDownloadSession {
        let logical = ICloudCoordinator.logicalURL(for: url)
        let key = ICloudCoordinator.canonicalLogicalKey(for: logical)

        if autoOpenWhenReady {
            pendingAutoOpenKey = key
        }

        // Deduplicate if already in flight
        if let existing = sessions[key], existing.status.isInFlight {
            AppLogger.shared.log(.icloud, "Deduplicated iCloud download request for \(logical.lastPathComponent)")
            onStateChanged?(existing)
            return existing
        }

        // Fast-path: check if the file was already downloaded externally (e.g. by Finder)
        if let verifiedURL = ICloudCoordinator.verifyDownloadedPDF(at: logical) {
            let completed = ICloudDownloadSession(
                canonicalKey: key,
                requestedURL: url,
                logicalURL: verifiedURL,
                status: .downloadedVerified,
                progress: 1.0,
                errorMessage: nil,
                requestedAt: Date()
            )
            sessions[key] = completed
            onStateChanged?(completed)
            if pendingAutoOpenKey == key {
                pendingAutoOpenKey = nil
                onAutoOpenReady?(verifiedURL)
            }
            return completed
        }

        var session = ICloudDownloadSession(
            canonicalKey: key,
            requestedURL: url,
            logicalURL: logical,
            status: .downloadRequested,
            progress: nil,
            errorMessage: nil,
            requestedAt: Date()
        )
        sessions[key] = session
        onStateChanged?(session)

        do {
            try backend.startUbiquitousDownload(for: url)
            session.status = .downloading
            sessions[key] = session
            onStateChanged?(session)
            AppLogger.shared.log(.icloud, "Started iCloud download for \(logical.lastPathComponent)")
        } catch {
            session.status = .failed
            session.errorMessage = error.localizedDescription
            sessions[key] = session
            onStateChanged?(session)
            AppLogger.shared.log(.error, "iCloud download request failed for \(logical.lastPathComponent): \(error.localizedDescription)")
            return session
        }

        startMonitoringDownload(forKey: key, initialURL: url, logicalURL: logical)
        return session
    }

    public func cancelDownloadMonitoring(for url: URL) {
        let key = ICloudCoordinator.canonicalLogicalKey(for: url)
        monitorTasks[key]?.cancel()
        monitorTasks.removeValue(forKey: key)
        if pendingAutoOpenKey == key {
            pendingAutoOpenKey = nil
        }
    }

    public func isTargetStillActive(for url: URL) -> Bool {
        let key = ICloudCoordinator.canonicalLogicalKey(for: url)
        return pendingAutoOpenKey == key
    }

    public func notifyFileSystemChanged(for url: URL) {
        let logical = ICloudCoordinator.logicalURL(for: url)
        let key = ICloudCoordinator.canonicalLogicalKey(for: logical)
        if let verified = ICloudCoordinator.verifyDownloadedPDF(at: logical) {
            completeDownloadSession(forKey: key, verifiedURL: verified)
        }
    }

    public func resetAllSessions() {
        for (_, task) in monitorTasks {
            task.cancel()
        }
        monitorTasks.removeAll()
        sessions.removeAll()
        pendingAutoOpenKey = nil
    }

    private func startMonitoringDownload(forKey key: String, initialURL: URL, logicalURL: URL) {
        monitorTasks[key]?.cancel()
        let interval = pollIntervalNanoseconds
        let maxLoops = maxWaitLoops

        let task = Task { @MainActor [weak self] in
            guard let self else { return }
            for _ in 0..<maxLoops {
                if Task.isCancelled { return }

                // Check if materialized & verified
                if let verifiedURL = ICloudCoordinator.verifyDownloadedPDF(at: logicalURL) {
                    self.completeDownloadSession(forKey: key, verifiedURL: verifiedURL)
                    return
                }

                let inspected = self.backend.inspectStatus(for: initialURL)
                if inspected.status == .failed || inspected.status == .unavailable {
                    if var current = self.sessions[key] {
                        current.status = inspected.status
                        current.errorMessage = inspected.errorMessage ?? "Unable to download from iCloud Drive."
                        self.sessions[key] = current
                        self.onStateChanged?(current)
                    }
                    self.monitorTasks.removeValue(forKey: key)
                    return
                } else if inspected.status == .local {
                    if let verifiedURL = ICloudCoordinator.verifyDownloadedPDF(at: logicalURL) {
                        self.completeDownloadSession(forKey: key, verifiedURL: verifiedURL)
                        return
                    }
                } else if var current = self.sessions[key] {
                    if current.status != .downloading || current.progress != inspected.progress {
                        current.status = .downloading
                        current.progress = inspected.progress
                        self.sessions[key] = current
                        self.onStateChanged?(current)
                    }
                }

                try? await Task.sleep(nanoseconds: interval)
            }

            if Task.isCancelled { return }
            if var timedOut = self.sessions[key], timedOut.status.isInFlight {
                timedOut.status = .failed
                timedOut.errorMessage = "iCloud download timed out for “\(logicalURL.lastPathComponent)”. Check your network connection and click Retry."
                self.sessions[key] = timedOut
                self.onStateChanged?(timedOut)
                AppLogger.shared.log(.error, "iCloud download timed out for \(logicalURL.lastPathComponent)")
            }
            self.monitorTasks.removeValue(forKey: key)
        }

        monitorTasks[key] = task
    }

    private func completeDownloadSession(forKey key: String, verifiedURL: URL) {
        monitorTasks[key]?.cancel()
        monitorTasks.removeValue(forKey: key)

        guard var current = sessions[key] else { return }
        current.status = .downloadedVerified
        current.progress = 1.0
        current.errorMessage = nil
        sessions[key] = current
        onStateChanged?(current)

        AppLogger.shared.log(.icloud, "Verified downloaded iCloud PDF at \(verifiedURL.path)")

        if pendingAutoOpenKey == key {
            pendingAutoOpenKey = nil
            onAutoOpenReady?(verifiedURL)
        } else {
            onBackgroundDownloadReady?(verifiedURL)
        }
    }
}

public final class WorkspaceNode: NSObject {
    public let name: String
    public var path: String
    public let isDirectory: Bool
    public var iCloudStatus: ICloudStatus
    public var children: [WorkspaceNode]
    public var isLoaded: Bool
    public var isLoading: Bool

    public static let maxRecursionDepth: Int = 25
    public static let maxTotalItems: Int = 50_000

    public init(
        name: String,
        path: String,
        isDirectory: Bool,
        iCloudStatus: ICloudStatus = .local,
        children: [WorkspaceNode] = [],
        isLoaded: Bool = false,
        isLoading: Bool = false
    ) {
        let cleanName: String
        if name.lowercased().hasSuffix(".icloud") && name.hasPrefix(".") {
            let trimmed = String(name.dropFirst().dropLast(".icloud".count))
            cleanName = trimmed.isEmpty ? name : trimmed
        } else {
            cleanName = name
        }
        self.name = cleanName
        self.path = path
        self.isDirectory = isDirectory
        self.iCloudStatus = iCloudStatus
        self.children = children
        self.isLoaded = isLoaded
        self.isLoading = isLoading
        super.init()
    }

    /// Logical path resolving `.filename.pdf.icloud` to `filename.pdf`.
    public var logicalPath: String {
        ICloudCoordinator.logicalURL(for: URL(fileURLWithPath: path)).path
    }

    /// Synchronously enumerates immediate children of a directory, deduplicating `.icloud` placeholders
    /// when a materialized file of the same logical name already exists.
    public static func scanImmediateChildren(
        at dirPath: String,
        showHiddenFiles: Bool = SettingsManager.shared.showHiddenFiles
    ) -> [WorkspaceNode] {
        let fm = FileManager.default
        let dirURL = URL(fileURLWithPath: dirPath)
        guard let contents = try? fm.contentsOfDirectory(
            at: dirURL,
            includingPropertiesForKeys: [
                .isDirectoryKey,
                .isSymbolicLinkKey,
                .isUbiquitousItemKey,
                .ubiquitousItemDownloadingStatusKey,
                .ubiquitousItemIsDownloadingKey
            ],
            options: []
        ) else {
            return []
        }

        var nodesByLogicalKey: [String: WorkspaceNode] = [:]

        for fileURL in contents {
            let rawName = fileURL.lastPathComponent
            let isICloudPlaceholder = rawName.hasPrefix(".") && rawName.lowercased().hasSuffix(".icloud")

            if !showHiddenFiles && rawName.hasPrefix(".") && !isICloudPlaceholder {
                continue
            }

            var isDirObj: ObjCBool = false
            let exists = fm.fileExists(atPath: fileURL.path, isDirectory: &isDirObj)
            let isDir = exists && isDirObj.boolValue

            if !isDir {
                let lower = rawName.lowercased()
                let isPDF = lower.hasSuffix(".pdf") || lower.hasSuffix(".pdf.icloud")
                if !isPDF {
                    continue
                }
            }

            let cloudState: ICloudStatus
            if isICloudPlaceholder {
                cloudState = .cloudOnly
            } else {
                cloudState = SystemICloudProviderBackend.inspectSystemStatus(for: fileURL).status
            }

            let logicalKey = ICloudCoordinator.canonicalLogicalKey(for: fileURL)
            if let existing = nodesByLogicalKey[logicalKey] {
                // Prefer the materialized `.pdf` entry over `.pdf.icloud` if both briefly coexist
                if existing.path.lowercased().hasSuffix(".icloud") && !isICloudPlaceholder {
                    nodesByLogicalKey[logicalKey] = WorkspaceNode(
                        name: rawName,
                        path: fileURL.path,
                        isDirectory: isDir,
                        iCloudStatus: cloudState
                    )
                }
            } else {
                nodesByLogicalKey[logicalKey] = WorkspaceNode(
                    name: rawName,
                    path: fileURL.path,
                    isDirectory: isDir,
                    iCloudStatus: cloudState
                )
            }

            if nodesByLogicalKey.count >= maxTotalItems {
                break
            }
        }

        var nodes = Array(nodesByLogicalKey.values)
        nodes.sort { a, b in
            if a.isDirectory != b.isDirectory {
                return a.isDirectory && !b.isDirectory
            }
            return a.name.localizedStandardCompare(b.name) == .orderedAscending
        }
        return nodes
    }

    public func loadChildrenSynchronously(showHiddenFiles: Bool = SettingsManager.shared.showHiddenFiles) {
        guard isDirectory else { return }
        children = Self.scanImmediateChildren(at: path, showHiddenFiles: showHiddenFiles)
        isLoaded = true
        isLoading = false
    }

    /// Recursively collects all PDF file paths in a workspace directory with symlink cycle detection and depth/item caps.
    /// When `onlyLocallyAvailable` is true, skips `.cloudOnly` / `.icloud` placeholder files so opening a workspace
    /// never triggers an unwanted cloud download error on mount.
    public static func collectAllPDFPaths(
        in rootPath: String,
        showHiddenFiles: Bool = SettingsManager.shared.showHiddenFiles,
        onlyLocallyAvailable: Bool = false
    ) -> [String] {
        var results: [String] = []
        var visitedDirs: Set<String> = []

        func walk(_ currentDir: String, depth: Int) {
            guard depth <= maxRecursionDepth, results.count < maxTotalItems else { return }
            let canon = UIConstants.canonicalPath(currentDir)
            guard !canon.isEmpty, !visitedDirs.contains(canon) else { return }
            visitedDirs.insert(canon)

            let children = scanImmediateChildren(at: currentDir, showHiddenFiles: showHiddenFiles)
            for child in children {
                if results.count >= maxTotalItems { break }
                if child.isDirectory {
                    walk(child.path, depth: depth + 1)
                } else {
                    if onlyLocallyAvailable && !child.iCloudStatus.isReadyToOpen {
                        continue
                    }
                    results.append(child.path)
                }
            }
        }

        walk(rootPath, depth: 0)
        return results
    }
}
