// Draws Kanso's app icon: an ink ensō with a vermilion seal on a warm
// off-white macOS tile. Usage: swift make-icon.swift <out.png> [size]
import AppKit

let out = CommandLine.arguments[1]
let size = CommandLine.arguments.count > 2 ? Double(CommandLine.arguments[2])! : 1024
let s = size / 1024  // everything below is laid out on the 1024 grid

let rep = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: Int(size), pixelsHigh: Int(size),
                           bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
                           colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
let cg = NSGraphicsContext.current!.cgContext
cg.scaleBy(x: s, y: s)

func rgb(_ hex: UInt32, _ a: Double = 1) -> CGColor {
    CGColor(red: Double((hex >> 16) & 0xff) / 255, green: Double((hex >> 8) & 0xff) / 255,
            blue: Double(hex & 0xff) / 255, alpha: a)
}

// The tile: 824 px centred, corner radius 185, with a soft drop shadow.
let tile = CGPath(roundedRect: CGRect(x: 100, y: 100, width: 824, height: 824),
                  cornerWidth: 185, cornerHeight: 185, transform: nil)
cg.saveGState()
cg.setShadow(offset: CGSize(width: 0, height: -10), blur: 28, color: rgb(0x000000, 0.28))
cg.addPath(tile); cg.setFillColor(rgb(0xF4EFE5)); cg.fillPath()
cg.restoreGState()
cg.saveGState()
cg.addPath(tile); cg.clip()
let gradient = CGGradient(colorsSpace: CGColorSpaceCreateDeviceRGB(),
                          colors: [rgb(0xFCFAF4), rgb(0xECE5D7)] as CFArray, locations: [0, 1])!
cg.drawLinearGradient(gradient, start: CGPoint(x: 512, y: 924), end: CGPoint(x: 512, y: 100), options: [])
cg.restoreGState()
cg.addPath(tile); cg.setStrokeColor(rgb(0x000000, 0.07)); cg.setLineWidth(2); cg.strokePath()

// The ensō: an open circle, thick where the brush lands and thinning toward
// the gap at the upper right, drawn as many short round-capped segments.
let centre = CGPoint(x: 505, y: 505), radius = 228.0
let start = 58.0, sweep = 318.0, steps = 240
cg.setStrokeColor(rgb(0x2B2722)); cg.setLineCap(.round)
for i in 0..<steps {
    let t0 = Double(i) / Double(steps), t1 = Double(i + 1) / Double(steps)
    let a0 = (start + sweep * t0) * .pi / 180, a1 = (start + sweep * t1) * .pi / 180
    let width = 30 + 34 * sin(.pi * pow(t0, 0.8))  // swells, then thins at both ends
    cg.setLineWidth(width)
    cg.move(to: CGPoint(x: centre.x + radius * cos(a0), y: centre.y + radius * sin(a0)))
    cg.addLine(to: CGPoint(x: centre.x + radius * cos(a1), y: centre.y + radius * sin(a1)))
    cg.strokePath()
}

// The seal, in the gap.
cg.setFillColor(rgb(0xC2492E))
cg.fillEllipse(in: CGRect(x: 700 - 24, y: 690 - 24, width: 48, height: 48))

// Kanso Dev: an amber band across the foot of the tile, saying DEV, so the
// two apps are never mistaken for each other in the Dock or Finder.
if ProcessInfo.processInfo.environment["KANSO_ICON_DEV"] != nil {
    cg.saveGState()
    cg.addPath(tile); cg.clip()
    cg.setFillColor(rgb(0xD9922E)); cg.fill(CGRect(x: 100, y: 100, width: 824, height: 200))
    cg.restoreGState()
    let label = NSAttributedString(string: "DEV", attributes: [
        .font: NSFont.systemFont(ofSize: 132, weight: .heavy),
        .foregroundColor: NSColor.white,
        .kern: 14,
    ])
    let box = label.size()
    label.draw(at: NSPoint(x: 512 - box.width / 2, y: 200 - box.height / 2))
}

// Kanso Nightly: a blue band across the foot of the tile, saying NIGHTLY,
// so the three apps are never mistaken for each other in the Dock or Finder.
if ProcessInfo.processInfo.environment["KANSO_ICON_NIGHTLY"] != nil {
    cg.saveGState()
    cg.addPath(tile); cg.clip()
    cg.setFillColor(rgb(0x3B5BDB)); cg.fill(CGRect(x: 100, y: 100, width: 824, height: 200))
    cg.restoreGState()
    let label = NSAttributedString(string: "NIGHTLY", attributes: [
        .font: NSFont.systemFont(ofSize: 64, weight: .heavy),
        .foregroundColor: NSColor.white,
        .kern: 6,
    ])
    let box = label.size()
    label.draw(at: NSPoint(x: 512 - box.width / 2, y: 200 - box.height / 2))
}

NSGraphicsContext.restoreGraphicsState()
try! rep.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: out))
