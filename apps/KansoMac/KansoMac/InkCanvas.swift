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

    func makeNSView(context: Context) -> InkCaptureView {
        let view = InkCaptureView()
        view.onStrokesChanged = { strokes.wrappedValue = $0 }
        return view
    }

    func updateNSView(_ nsView: InkCaptureView, context: Context) {}
}

/// An `NSView` that turns mouse/trackpad drags into strokes. `NSEvent.pressure`
/// supplies force on pressure-sensitive trackpads; otherwise it defaults to 1.0.
final class InkCaptureView: NSView {
    var onStrokesChanged: (([InkStroke]) -> Void)?

    private var strokes: [InkStroke] = []
    private var currentPoints: [InkPoint] = []

    override var isFlipped: Bool { true } // top-left origin, matching the canvas model

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
        strokes.append(
            InkStroke(
                points: currentPoints,
                color: ColorRgba(r: 20, g: 20, b: 20, a: 255),
                width: 2.5
            )
        )
        currentPoints = []
        onStrokesChanged?(strokes)
        needsDisplay = true
    }

    // Lightweight wet-ink preview. The committed render goes through the Rust
    // core (tiny-skia / wgpu); this is only an immediate on-screen trace.
    override func draw(_ dirtyRect: NSRect) {
        NSColor.textBackgroundColor.setFill()
        dirtyRect.fill()
        NSColor.labelColor.setStroke()

        func trace(_ points: [InkPoint]) {
            guard let first = points.first else { return }
            let path = NSBezierPath()
            path.lineWidth = 2.5
            path.lineCapStyle = .round
            path.lineJoinStyle = .round
            path.move(to: CGPoint(x: CGFloat(first.x), y: CGFloat(first.y)))
            for p in points.dropFirst() {
                path.line(to: CGPoint(x: CGFloat(p.x), y: CGFloat(p.y)))
            }
            path.stroke()
        }

        for stroke in strokes { trace(stroke.points) }
        trace(currentPoints)
    }
}
