// Kanso — CaptureUITests.swift
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import XCTest

/// Shares a link, an image and a PDF from Safari, Photos and Files to Kanso's
/// share extension with Kanso not running, then opens Kanso and finds the new
/// notes in All Notes (#48 step 10).
///
/// What the tests expect is put in place by `scripts/mobile-capture-ios.sh`:
/// Kanso installed and launched once (that makes the library in the App
/// Group), `integration_test/media/Kanso-test-photo.png` in Photos,
/// `Quarterly-report.pdf` in Files under On My iPhone (or iPad), and
/// `capture-link.html` served on `localhost:8765`.
final class CaptureUITests: XCTestCase {
  private let kanso = XCUIApplication(bundleIdentifier: "app.kanso.Kanso")
  private let saved = "Saved to your Inbox."

  override func setUp() {
    continueAfterFailure = false
    kanso.terminate()
  }

  override func tearDown() {
    kanso.terminate()
  }

  // MARK: - Tests

  func testLinkFromSafari() {
    let safari = openInSafari("capture-link.html")
    XCTAssertTrue(safari.staticTexts["Kanso capture link test"].firstMatch.waitForExistence(timeout: 20))
    shareFromSafari(safari)
    tapKanso(in: safari)
    expectSaved(in: safari, shot: "link")
    expectNote("Kanso capture link test", shot: "link")
  }

  /// A PDF opened in Safari, shared from there. Files' own share sheet lists
  /// only Preview on the Simulator, share extensions included none (see
  /// `testPDFFromFiles`).
  func testPDFFromSafari() {
    let safari = openInSafari("Quarterly-report.pdf")
    sleep(6)
    shareFromSafari(safari)
    tapKanso(in: safari)
    expectSaved(in: safari, shot: "pdf")
    expectNote("Quarterly-report", shot: "pdf")
  }

  func testImageFromPhotos() {
    let photos = XCUIApplication(bundleIdentifier: "com.apple.mobileslideshow")
    photos.terminate()
    photos.launch()
    // The first launch after an update shows "What's New".
    let dismiss = photos.buttons["Continue"].firstMatch
    if dismiss.waitForExistence(timeout: 5) { dismiss.tap() }
    // The newest photo is the last in the grid: the one `simctl addmedia` added.
    let grid = photos.images.matching(identifier: "PXGGridLayout-Info")
    XCTAssertTrue(grid.firstMatch.waitForExistence(timeout: 20), photos.debugDescription)
    grid.element(boundBy: grid.count - 1).tap()
    let share = photos.buttons["Share"].firstMatch
    XCTAssertTrue(share.waitForExistence(timeout: 10), photos.debugDescription)
    share.tap()
    tapKanso(in: photos)
    expectSaved(in: photos, shot: "image")
    expectNote("Kanso-test-photo", shot: "image")
  }

  /// Shares the PDF from Files' own sheet. The Simulator's Files lists only
  /// Preview under Apps, whatever the extension's activation rule, so this
  /// skips there; on a device it is the real thing.
  func testPDFFromFiles() throws {
    let files = openLocalFiles()
    let pdf = files.cells.matching(NSPredicate(format: "label BEGINSWITH 'Quarterly-report'")).firstMatch
    pdf.press(forDuration: 1.2)
    let share = files.buttons["Share"].firstMatch
    XCTAssertTrue(share.waitForExistence(timeout: 5), files.debugDescription)
    share.tap()
    let item = files.cells["Kanso"].firstMatch
    guard item.waitForExistence(timeout: 8) else {
      let more = files.cells["More"].firstMatch
      if more.exists { more.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.3)).tap() }
      if !files.staticTexts["Kanso"].firstMatch.waitForExistence(timeout: 5) {
        throw XCTSkip("Files' share sheet doesn't offer Kanso on this Simulator.")
      }
      files.staticTexts["Kanso"].firstMatch.tap()
      expectSaved(in: files, shot: "pdf-files")
      expectNote("Quarterly-report", shot: "pdf-files")
      return
    }
    item.tap()
    expectSaved(in: files, shot: "pdf-files")
    expectNote("Quarterly-report", shot: "pdf-files")
  }

  // MARK: - Steps

  /// Files, in the "On My iPhone" (or iPad) folder, with the PDF in view.
  private func openLocalFiles() -> XCUIApplication {
    let files = XCUIApplication(bundleIdentifier: "com.apple.DocumentsApp")
    files.terminate()
    files.launch()
    // Both have a Browse button: a tab bar on the iPhone, the top bar on the iPad.
    let pdf = files.cells.matching(NSPredicate(format: "label BEGINSWITH 'Quarterly-report'")).firstMatch
    let browse = files.buttons["Browse"].firstMatch
    if browse.waitForExistence(timeout: 10) { browse.tap() }
    // Files reopens where it was left: the folder, or the list of places.
    if !pdf.waitForExistence(timeout: 5) {
      let local = files.descendants(matching: .any)
        .matching(NSPredicate(format: "label BEGINSWITH 'On My i' AND (elementType == %d OR elementType == %d)",
                              XCUIElement.ElementType.staticText.rawValue, XCUIElement.ElementType.cell.rawValue)).firstMatch
      XCTAssertTrue(local.waitForExistence(timeout: 10), files.debugDescription)
      local.tap()
      XCTAssertTrue(pdf.waitForExistence(timeout: 10), files.debugDescription)
    }
    return files
  }

  /// Opens a file of the script's local server in Safari.
  private func openInSafari(_ file: String) -> XCUIApplication {
    let safari = XCUIApplication(bundleIdentifier: "com.apple.mobilesafari")
    safari.terminate()
    safari.launch()
    XCUIDevice.shared.system.open(URL(string: "http://localhost:8765/\(file)")!)
    return safari
  }

  /// Opens Safari's share sheet: the toolbar's Share button on an iPad, the
  /// page menu's Share on an iPhone.
  private func shareFromSafari(_ safari: XCUIApplication) {
    let share = safari.buttons["ShareButton"].firstMatch
    if share.waitForExistence(timeout: 3) {
      share.tap()
    } else {
      safari.buttons["MoreMenuButton"].firstMatch.tap()
      let item = safari.buttons["Share"].firstMatch
      XCTAssertTrue(item.waitForExistence(timeout: 5), safari.debugDescription)
      item.tap()
    }
  }

  /// Taps Kanso in the open share sheet, swiping the app row when it isn't
  /// in view.
  private func tapKanso(in host: XCUIApplication) {
    let item = host.cells["Kanso"].firstMatch
    // The sheet fills in after it opens; if Kanso isn't in the first screen
    // of the app row, swipe the row.
    for _ in 0..<5 where !item.waitForExistence(timeout: 8) {
      let row = host.collectionViews["ActivityListView"].firstMatch
      let list = row.exists ? row : host.collectionViews.firstMatch
      if list.isHittable { list.swipeLeft() }
    }
    XCTAssertTrue(item.exists, host.debugDescription)
    item.tap()
  }

  private func expectSaved(in host: XCUIApplication, shot: String) {
    let text = host.staticTexts[saved].firstMatch
    XCTAssertTrue(text.waitForExistence(timeout: 30), host.debugDescription)
    screenshot("\(shot)-confirmation")
  }

  /// Launches Kanso and finds a note whose title contains `title` in All Notes.
  private func expectNote(_ title: String, shot: String) {
    kanso.launch()
    let match = NSPredicate(format: "label CONTAINS[c] %@", title)
    let note = kanso.staticTexts.matching(match).firstMatch
    if !note.waitForExistence(timeout: 20) {
      // A phone opens on the notebooks: go to All Notes first.
      let all = kanso.staticTexts["All Notes"].firstMatch
      if all.exists { all.tap() }
    }
    XCTAssertTrue(note.waitForExistence(timeout: 20), kanso.debugDescription)
    screenshot("\(shot)-list")
  }

  /// Keeps a screenshot as an attachment and, when the script names a
  /// folder (`CAPTURE_SHOTS`), as a PNG in it.
  private func screenshot(_ name: String) {
    let image = XCUIScreen.main.screenshot()
    let attachment = XCTAttachment(screenshot: image)
    attachment.name = name
    attachment.lifetime = .keepAlways
    add(attachment)
    if let folder = ProcessInfo.processInfo.environment["CAPTURE_SHOTS"] {
      try? image.pngRepresentation.write(to: URL(fileURLWithPath: folder).appendingPathComponent("\(name).png"))
    }
  }
}
