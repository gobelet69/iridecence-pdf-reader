import AppKit
import Foundation
import PDFKit
import UniformTypeIdentifiers

public enum AnnotationType: Sendable, Equatable {
    case highlight
    case underline
    case strikeout
    case textNote
}

public struct AnnotationSummary: Sendable, Equatable {
    public let pageIndex: Int
    public let subtype: String
    public let textPreview: String
    public let bounds: NSRect

    public init(pageIndex: Int, subtype: String, textPreview: String, bounds: NSRect) {
        self.pageIndex = pageIndex
        self.subtype = subtype
        self.textPreview = textPreview
        self.bounds = bounds
    }
}

public enum AnnotationSaveError: LocalizedError, Equatable {
    case masterDirectoryProtected
    case writeFailed(String)

    public var errorDescription: String? {
        switch self {
        case .masterDirectoryProtected:
            return "Protected directory: Saving directly into /Documents/MASTER is strictly forbidden. Please use 'Save As…' to save a copy to another folder."
        case .writeFailed(let detail):
            return detail
        }
    }
}

/// Manages PDFKit annotations (Highlight, Underline, Strikeout, TextNote) with multi-line
/// `quadrilateralPoints` support and atomic safe saving with strict `/Documents/MASTER` protection.
public enum AnnotationsManager {
    /// Enforces Rule 2.1: Never touch, modify, write to, or delete files in `/Users/theodeville/Documents/MASTER`.
    public static func isMasterProtectedPath(_ filePath: String) -> Bool {
        let rawCheck = filePath.contains("/Documents/MASTER/") || filePath.hasSuffix("/Documents/MASTER")
        let canonical = UIConstants.canonicalPath(filePath)
        let canonCheck = canonical.contains("/Documents/MASTER/") || canonical.hasSuffix("/Documents/MASTER")
        return rawCheck || canonCheck
    }

    @MainActor
    @discardableResult
    public static func addAnnotationToSelection(
        in pdfView: PDFView,
        type: AnnotationType,
        noteText: String? = nil,
        settings: SettingsManager = .shared
    ) -> Bool {
        guard let selection = pdfView.currentSelection else {
            AppLogger.shared.log(.pdf, "Cannot add annotation: No text currently selected")
            return false
        }

        let pages = selection.pages
        guard !pages.isEmpty else { return false }

        let lineSelections = selection.selectionsByLine()
        var addedCount = 0

        for page in pages {
            let unionBounds = selection.bounds(for: page)
            guard unionBounds.width > 0.0, unionBounds.height > 0.0 else { continue }

            let (subtype, color): (PDFAnnotationSubtype, NSColor) = {
                switch type {
                case .highlight:
                    return (.highlight, settings.configuredHighlightColor())
                case .underline:
                    return (.underline, NSColor(srgbRed: 0.15, green: 0.65, blue: 0.95, alpha: 0.85))
                case .strikeout:
                    return (.strikeOut, NSColor(srgbRed: 0.95, green: 0.30, blue: 0.30, alpha: 0.85))
                case .textNote:
                    return (.text, NSColor(srgbRed: 1.0, green: 0.80, blue: 0.20, alpha: 0.95))
                }
            }()

            let targetBounds: NSRect
            if type == .textNote {
                targetBounds = NSRect(
                    x: max(12.0, unionBounds.origin.x - 22.0),
                    y: unionBounds.origin.y + max(0.0, unionBounds.height - 20.0),
                    width: 20.0,
                    height: 20.0
                )
            } else {
                targetBounds = unionBounds
            }

            let annotation = PDFAnnotation(bounds: targetBounds, forType: subtype, withProperties: nil)
            annotation.color = color

            if type != .textNote && !lineSelections.isEmpty {
                var quadPoints: [NSValue] = []
                for lineSel in lineSelections {
                    let linePages = lineSel.pages
                    guard linePages.contains(where: { $0 === page }) else { continue }
                    let lb = lineSel.bounds(for: page)
                    guard lb.width > 0.0, lb.height > 0.0 else { continue }

                    let xMin = lb.origin.x - unionBounds.origin.x
                    let yMin = lb.origin.y - unionBounds.origin.y
                    let xMax = xMin + lb.width
                    let yMax = yMin + lb.height

                    // PDFKit quadrilateral point order: top-left, top-right, bottom-left, bottom-right
                    quadPoints.append(NSValue(point: NSPoint(x: xMin, y: yMax)))
                    quadPoints.append(NSValue(point: NSPoint(x: xMax, y: yMax)))
                    quadPoints.append(NSValue(point: NSPoint(x: xMin, y: yMin)))
                    quadPoints.append(NSValue(point: NSPoint(x: xMax, y: yMin)))
                }
                if quadPoints.count >= 4 {
                    annotation.quadrilateralPoints = quadPoints
                }
            }

            let selectedSnippet = (selection.string ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
            if let note = noteText, !note.isEmpty {
                annotation.contents = note
            } else if type == .textNote, !selectedSnippet.isEmpty {
                let preview = String(selectedSnippet.prefix(120))
                annotation.contents = preview
            }

            page.addAnnotation(annotation)
            addedCount += 1
        }

        if addedCount > 0 {
            pdfView.clearSelection()
            pdfView.needsDisplay = true
            AppLogger.shared.log(.pdf, "Added \(addedCount) annotation(s) (\(type))")
            return true
        }
        return false
    }

    public static func getDocumentAnnotations(in doc: PDFDocument) -> [AnnotationSummary] {
        var results: [AnnotationSummary] = []
        let pageCount = min(doc.pageCount, 25_000)

        for pageIdx in 0..<pageCount {
            guard let page = doc.page(at: pageIdx) else { continue }
            for ann in page.annotations {
                let subtype = ann.type ?? ""
                guard subtype == "Highlight"
                    || subtype == "Underline"
                    || subtype == "StrikeOut"
                    || subtype == "Text"
                    || subtype == "FreeText"
                else {
                    continue
                }

                let bounds = ann.bounds
                var preview = (ann.contents ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
                if preview.isEmpty, let sel = page.selection(for: bounds), let str = sel.string {
                    preview = str.trimmingCharacters(in: .whitespacesAndNewlines)
                }
                preview = preview.replacingOccurrences(of: "\n", with: " ").replacingOccurrences(of: "\r", with: " ")
                if preview.count > 80 {
                    preview = String(preview.prefix(80)) + "…"
                }

                results.append(AnnotationSummary(
                    pageIndex: pageIdx,
                    subtype: subtype,
                    textPreview: preview,
                    bounds: bounds
                ))
            }
        }
        return results
    }

    @MainActor
    @discardableResult
    public static func removeAnnotation(
        in doc: PDFDocument,
        pageIndex: Int,
        targetBounds: NSRect,
        targetSubtype: String
    ) -> Bool {
        guard let page = doc.page(at: pageIndex) else { return false }
        for ann in page.annotations {
            let subtype = ann.type ?? ""
            let b = ann.bounds
            let dx = abs(b.origin.x - targetBounds.origin.x)
            let dy = abs(b.origin.y - targetBounds.origin.y)
            if subtype == targetSubtype && dx < 1.5 && dy < 1.5 {
                page.removeAnnotation(ann)
                return true
            }
        }
        return false
    }

    /// Safely writes a `PDFDocument` to disk using an atomic temporary file swap,
    /// strictly blocking any path inside `/Documents/MASTER`.
    public static func saveDocumentSafely(_ doc: PDFDocument, to filePath: String) throws {
        if isMasterProtectedPath(filePath) {
            AppLogger.shared.log(.error, "Blocked attempt to save inside protected MASTER directory: \(filePath)")
            throw AnnotationSaveError.masterDirectoryProtected
        }

        let targetURL = URL(fileURLWithPath: filePath)
        let parentDir = targetURL.deletingLastPathComponent()
        let fileName = targetURL.lastPathComponent
        let tempURL = parentDir.appendingPathComponent(".\(fileName).iridescence_tmp.pdf")

        // Double-check tempURL is also outside MASTER
        if isMasterProtectedPath(tempURL.path) {
            throw AnnotationSaveError.masterDirectoryProtected
        }

        guard doc.write(to: tempURL) else {
            let _ = try? FileManager.default.removeItem(at: tempURL)
            throw AnnotationSaveError.writeFailed("PDFKit failed to write temporary PDF at \(tempURL.path)")
        }

        guard let attrs = try? FileManager.default.attributesOfItem(atPath: tempURL.path),
              let fileSize = attrs[.size] as? UInt64,
              fileSize >= 64 else {
            let _ = try? FileManager.default.removeItem(at: tempURL)
            throw AnnotationSaveError.writeFailed("Temporary PDF file is unexpectedly empty or invalid.")
        }

        do {
            if FileManager.default.fileExists(atPath: targetURL.path) {
                _ = try FileManager.default.replaceItemAt(targetURL, withItemAt: tempURL)
            } else {
                try FileManager.default.moveItem(at: tempURL, to: targetURL)
            }
            AppLogger.shared.log(.pdf, "Safely saved PDF document to \(filePath) (\(fileSize) bytes)")
        } catch {
            let _ = try? FileManager.default.removeItem(at: tempURL)
            throw AnnotationSaveError.writeFailed("Failed to finalize saved PDF at \(filePath): \(error.localizedDescription)")
        }
    }

    @MainActor
    public static func runSavePanel(defaultFilename: String) -> String? {
        let panel = NSSavePanel()
        panel.canCreateDirectories = true
        panel.nameFieldStringValue = defaultFilename
        panel.allowedContentTypes = [.pdf]
        let response = panel.runModal()
        guard response == .OK, let url = panel.url else { return nil }
        return url.path
    }
}
