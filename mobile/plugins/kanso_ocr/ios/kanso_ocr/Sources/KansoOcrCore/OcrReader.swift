// Kanso — OcrReader.swift
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import Foundation
import ImageIO
import Vision

/// What `OcrReader` can fail with.
public enum OcrError: Error, Equatable {
  /// The bytes are no image the system can decode.
  case unreadable
}

/// The words in a picture, read by Apple Vision on the device. Nothing is
/// sent anywhere.
public enum OcrReader {
  /// The name written beside the text; the desktop's macOS Vision reader
  /// writes the same.
  public static let engine = "vision"

  /// The words in the picture ([data]: PNG, JPEG, HEIC/HEIF), one recognized
  /// line per line, top to bottom; empty when there are none. Blocking:
  /// call it from a background queue.
  public static func read(_ data: Data) throws -> String {
    guard let source = CGImageSourceCreateWithData(data as CFData, nil),
      CGImageSourceGetCount(source) > 0,
      CGImageSourceGetStatusAtIndex(source, 0) == .statusComplete,
      let properties = CGImageSourceCopyPropertiesAtIndex(source, 0, nil) as? [CFString: Any],
      properties[kCGImagePropertyPixelWidth] != nil
    else { throw OcrError.unreadable }

    let raw = (properties[kCGImagePropertyOrientation] as? NSNumber)?.uint32Value
    let orientation = raw.flatMap(CGImagePropertyOrientation.init(rawValue:)) ?? .up

    let request = VNRecognizeTextRequest()
    request.recognitionLevel = .accurate
    request.usesLanguageCorrection = true
    let handler = VNImageRequestHandler(data: data, orientation: orientation, options: [:])
    do {
      try handler.perform([request])
    } catch {
      throw OcrError.unreadable
    }
    return text(from: request.results ?? [])
  }

  /// The observations' best candidates in reading order.
  static func text(from observations: [VNRecognizedTextObservation]) -> String {
    let lines = observations.compactMap { observation -> Line? in
      guard let string = observation.topCandidates(1).first?.string else { return nil }
      return Line(string: string, top: observation.boundingBox.maxY, left: observation.boundingBox.minX)
    }
    return join(lines)
  }

  struct Line: Equatable {
    var string: String
    /// Vision's origin is the bottom left, so a larger `top` is higher up.
    var top: CGFloat
    var left: CGFloat
  }

  /// Top to bottom, then left to right; blank lines dropped.
  static func join(_ lines: [Line]) -> String {
    lines
      .sorted { a, b in a.top != b.top ? a.top > b.top : a.left < b.left }
      .map { $0.string.trimmingCharacters(in: .whitespacesAndNewlines) }
      .filter { !$0.isEmpty }
      .joined(separator: "\n")
  }
}
