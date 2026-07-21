import SwiftUI
import AppKit

// Native ink capture for macOS. Captures raw pointer/stylus input into the FFI's
// `InkStroke` type and hands completed strokes back via `onCommit`, which the app
// forwards to `engine.createSketch(...)`. This is the thin, platform-native
// capture layer; the geometry/format/render all live in the shared Rust ink core.
//
// On iPad the equivalent view uses `UITouch` coalesced/predicted samples + force
// for low-latency Apple Pencil capture, normalizing into the same `InkStroke`.

struct InkCanvas: NSViewRepresentable {
    /// Called with the full set of strokes when the user finishes (e.g. taps Save).
    var strokes: Binding<[InkStroke]>
    var color: ColorRgba = ColorRgba(r: 20, g: 20, b: 20, a: 255)
    var width: Float = 2.5
    var isErasing: Bool = false
    var onCommittedEdit: (([InkStroke], [InkStroke]) -> Void)?

    func makeNSView(context: Context) -> InkCaptureView {
        let view = InkCaptureView()
        view.setStrokes(strokes.wrappedValue)
        view.setDrawingStyle(color: color, width: width, isErasing: isErasing)
        view.onStrokesChanged = { strokes.wrappedValue = $0 }
        view.onCommittedEdit = onCommittedEdit
        return view
    }

    func updateNSView(_ nsView: InkCaptureView, context: Context) {
        nsView.setStrokes(strokes.wrappedValue)
        nsView.setDrawingStyle(color: color, width: width, isErasing: isErasing)
        nsView.onCommittedEdit = onCommittedEdit
    }
}

/// An `NSView` that turns mouse/trackpad drags into strokes. `NSEvent.pressure`
/// supplies force on pressure-sensitive trackpads; otherwise it defaults to 1.0.
final class InkCaptureView: NSView {
    var onStrokesChanged: (([InkStroke]) -> Void)?
    var onCommittedEdit: (([InkStroke], [InkStroke]) -> Void)?

    private var strokes: [InkStroke] = []
    private var currentPoints: [InkPoint] = []
    private var strokeColor = ColorRgba(r: 20, g: 20, b: 20, a: 255)
    private var strokeWidth: Float = 2.5
    private var isErasing = false

    override var isFlipped: Bool { true } // top-left origin, matching the canvas model

    func setDrawingStyle(color: ColorRgba, width: Float, isErasing: Bool) {
        strokeColor = color
        strokeWidth = width
        self.isErasing = isErasing
    }

    func setStrokes(_ nextStrokes: [InkStroke]) {
        guard nextStrokes != strokes else { return }
        strokes = nextStrokes
        currentPoints = []
        needsDisplay = true
    }

    private func sample(_ event: NSEvent) -> InkPoint {
        let p = convert(event.locationInWindow, from: nil)
        // `pressure` is 0…1; trackpads without force report 0 on move, so floor it.
        let pressure = max(Float(event.pressure), 0.15)
        return InkPoint(x: Float(p.x), y: Float(p.y), pressure: pressure)
    }

    override func mouseDown(with event: NSEvent) {
        currentPoints = [sample(event)]
    }

    override func mouseDragged(with event: NSEvent) {
        currentPoints.append(sample(event))
        needsDisplay = true
    }

    override func mouseUp(with event: NSEvent) {
        currentPoints.append(sample(event))
        guard currentPoints.count >= 2 else { currentPoints = []; return }
        let previousStrokes = strokes
        if isErasing {
            eraseStrokes(touchedBy: currentPoints)
        } else {
            strokes.append(
                InkStroke(
                    points: currentPoints,
                    color: strokeColor,
                    width: strokeWidth
                )
            )
        }
        currentPoints = []
        if previousStrokes != strokes {
            onCommittedEdit?(previousStrokes, strokes)
            onStrokesChanged?(strokes)
        }
        needsDisplay = true
    }

    private func eraseStrokes(touchedBy eraserPoints: [InkPoint]) {
        let radius = max(CGFloat(strokeWidth) * 5, 18)
        strokes.removeAll { stroke in
            stroke.points.contains { strokePoint in
                eraserPoints.contains { eraserPoint in
                    let dx = CGFloat(strokePoint.x - eraserPoint.x)
                    let dy = CGFloat(strokePoint.y - eraserPoint.y)
                    return sqrt(dx * dx + dy * dy) <= radius
                }
            }
        }
    }

    // Lightweight wet-ink preview. The committed render goes through the Rust
    // core (tiny-skia / wgpu); this is only an immediate on-screen trace.
    override func draw(_ dirtyRect: NSRect) {
        NSColor.textBackgroundColor.setFill()
        dirtyRect.fill()
        func trace(_ points: [InkPoint], color: ColorRgba = ColorRgba(r: 20, g: 20, b: 20, a: 255), width: CGFloat = 2.5) {
            guard let first = points.first else { return }
            NSColor(
                srgbRed: CGFloat(color.r) / 255,
                green: CGFloat(color.g) / 255,
                blue: CGFloat(color.b) / 255,
                alpha: CGFloat(color.a) / 255
            ).setStroke()
            let path = NSBezierPath()
            path.lineWidth = width
            path.lineCapStyle = .round
            path.lineJoinStyle = .round
            path.move(to: CGPoint(x: CGFloat(first.x), y: CGFloat(first.y)))
            for p in points.dropFirst() {
                path.line(to: CGPoint(x: CGFloat(p.x), y: CGFloat(p.y)))
            }
            path.stroke()
        }

        for stroke in strokes {
            trace(stroke.points, color: stroke.color, width: CGFloat(stroke.width))
        }
        if isErasing {
            trace(currentPoints, color: ColorRgba(r: 201, g: 110, b: 99, a: 210), width: max(CGFloat(strokeWidth) * 5, 18))
        } else {
            trace(currentPoints, color: strokeColor, width: CGFloat(strokeWidth))
        }
    }
}
