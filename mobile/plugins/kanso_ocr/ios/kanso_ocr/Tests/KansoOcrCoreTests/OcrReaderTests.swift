// Kanso — OcrReaderTests.swift
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import UIKit
import XCTest

@testable import KansoOcrCore

final class OcrReaderTests: XCTestCase {
  /// `text` in large black type on white.
  private func render(_ text: String, size: CGSize = CGSize(width: 800, height: 300)) -> UIImage {
    let format = UIGraphicsImageRendererFormat()
    format.scale = 1
    return UIGraphicsImageRenderer(size: size, format: format).image { context in
      UIColor.white.setFill()
      context.fill(CGRect(origin: .zero, size: size))
      let attributes: [NSAttributedString.Key: Any] = [
        .font: UIFont.boldSystemFont(ofSize: 160), .foregroundColor: UIColor.black,
      ]
      let box = (text as NSString).size(withAttributes: attributes)
      (text as NSString).draw(
        at: CGPoint(x: (size.width - box.width) / 2, y: (size.height - box.height) / 2),
        withAttributes: attributes)
    }
  }

  func testReadsKansoFromAPng() throws {
    let png = try XCTUnwrap(render("KANSO").pngData())
    XCTAssertEqual(try OcrReader.read(png).lowercased(), "kanso")
  }

  func testReadsKansoFromAJpeg() throws {
    let jpeg = try XCTUnwrap(render("KANSO").jpegData(compressionQuality: 0.9))
    XCTAssertEqual(try OcrReader.read(jpeg).lowercased(), "kanso")
  }

  func testABlankPictureHasNoWords() throws {
    let png = try XCTUnwrap(render("").pngData())
    XCTAssertEqual(try OcrReader.read(png), "")
  }

  func testBytesThatAreNoImageAreUnreadable() {
    XCTAssertThrowsError(try OcrReader.read(Data([1, 2, 3, 4]))) {
      XCTAssertEqual($0 as? OcrError, .unreadable)
    }
    XCTAssertThrowsError(try OcrReader.read(Data())) {
      XCTAssertEqual($0 as? OcrError, .unreadable)
    }
  }

  func testLinesReadTopToBottomThenLeftToRight() {
    let lines = [
      OcrReader.Line(string: "third", top: 0.2, left: 0.1),
      OcrReader.Line(string: "second right", top: 0.5, left: 0.6),
      OcrReader.Line(string: "first", top: 0.9, left: 0.1),
      OcrReader.Line(string: "  ", top: 0.7, left: 0.1),
      OcrReader.Line(string: "second left", top: 0.5, left: 0.1),
    ]
    XCTAssertEqual(OcrReader.join(lines), "first\nsecond left\nsecond right\nthird")
  }

  func testTheEngineIsNamedLikeTheDesktops() {
    XCTAssertEqual(OcrReader.engine, "vision")
  }
}
