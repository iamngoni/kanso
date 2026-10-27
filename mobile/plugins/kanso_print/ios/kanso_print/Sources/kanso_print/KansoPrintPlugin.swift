import Flutter
import UIKit

/// The Dart side of `KansoPrint.htmlToPdf`.
public class KansoPrintPlugin: NSObject, FlutterPlugin, PrintHostApi {
  public static func register(with registrar: FlutterPluginRegistrar) {
    let instance = KansoPrintPlugin()
    PrintHostApiSetup.setUp(binaryMessenger: registrar.messenger(), api: instance)
  }

  func htmlToPdf(html: String, outputPath: String, paper: PrintPaper, marginPt: Double) async throws -> Int64 {
    let job = await KansoPdfJob(html: html, outputPath: outputPath, paper: paper, margin: CGFloat(marginPt))
    return Int64(try await job.run())
  }
}
