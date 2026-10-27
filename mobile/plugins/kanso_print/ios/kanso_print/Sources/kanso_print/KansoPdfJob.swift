import Flutter
import UIKit
import WebKit

/// One page printed to one PDF: a `WKWebView` that is never on screen loads
/// the HTML, then a `UIPrintPageRenderer` paginates it into PDF pages.
@MainActor
final class KansoPdfJob: NSObject, WKNavigationDelegate {
  /// How long the layout may take.
  private static let timeout: TimeInterval = 60

  private let html: String
  private let outputPath: String
  private let paperRect: CGRect
  private let printableRect: CGRect
  private let webView: WKWebView
  private var continuation: CheckedContinuation<Int, Error>?

  init(html: String, outputPath: String, paper: PrintPaper, margin: CGFloat) {
    self.html = html
    self.outputPath = outputPath
    let size: CGSize
    switch paper {
    case .a4: size = CGSize(width: 595.28, height: 841.89)
    case .letter: size = CGSize(width: 612, height: 792)
    }
    paperRect = CGRect(origin: .zero, size: size)
    printableRect = paperRect.insetBy(dx: margin, dy: margin)
    let configuration = WKWebViewConfiguration()
    configuration.defaultWebpagePreferences.allowsContentJavaScript = false
    // The frame is the printable width, so the page lays out as it prints.
    webView = WKWebView(
      frame: CGRect(origin: .zero, size: printableRect.size), configuration: configuration)
    super.init()
    webView.navigationDelegate = self
  }

  /// Loads, paginates and writes the PDF; returns the page count.
  func run() async throws -> Int {
    try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Int, Error>) in
      self.continuation = continuation
      Task { @MainActor [weak self] in
        try? await Task.sleep(nanoseconds: UInt64(KansoPdfJob.timeout * 1_000_000_000))
        self?.finish(.failure(Self.failure("timeout", "The page took too long to lay out.")))
      }
      webView.loadHTMLString(html, baseURL: nil)
    }
  }

  private static func failure(_ code: String, _ message: String) -> PigeonError {
    PigeonError(code: code, message: message, details: nil)
  }

  /// Resumes once; later calls (the timeout after a success) do nothing.
  private func finish(_ result: Result<Int, Error>) {
    guard let continuation else { return }
    self.continuation = nil
    webView.stopLoading()
    continuation.resume(with: result)
  }

  func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
    // Give layout one more turn of the run loop before paginating.
    DispatchQueue.main.async { [weak self] in
      guard let self else { return }
      do {
        finish(.success(try render()))
      } catch {
        finish(.failure(error))
      }
    }
  }

  func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) {
    finish(.failure(Self.failure("load-failed", error.localizedDescription)))
  }

  func webView(
    _ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!,
    withError error: Error
  ) {
    finish(.failure(Self.failure("load-failed", error.localizedDescription)))
  }

  func webViewWebContentProcessDidTerminate(_ webView: WKWebView) {
    finish(.failure(Self.failure("web-process-ended", "The web view's process ended.")))
  }

  /// Paginates the loaded page into PDF pages and writes the file.
  private func render() throws -> Int {
    let renderer = UIPrintPageRenderer()
    renderer.addPrintFormatter(webView.viewPrintFormatter(), startingAtPageAt: 0)
    renderer.setValue(NSValue(cgRect: paperRect), forKey: "paperRect")
    renderer.setValue(NSValue(cgRect: printableRect), forKey: "printableRect")

    let pages = max(renderer.numberOfPages, 1)
    renderer.prepare(forDrawingPages: NSRange(location: 0, length: pages))
    let pdf = UIGraphicsPDFRenderer(bounds: paperRect)
    let data = pdf.pdfData { context in
      for index in 0..<pages {
        context.beginPage()
        renderer.drawPage(at: index, in: context.pdfContextBounds)
      }
    }
    do {
      try data.write(to: URL(fileURLWithPath: outputPath), options: .atomic)
    } catch {
      throw Self.failure("write-failed", error.localizedDescription)
    }
    return pages
  }
}
