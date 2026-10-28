// Kanso — KansoOcrPlugin.swift
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import Flutter
import KansoOcrCore
import UIKit

/// The platform's text recognizer, Apple Vision, on the device. The picture
/// never leaves it and no permission is needed.
public class KansoOcrPlugin: NSObject, FlutterPlugin, OcrHostApi {
  public static func register(with registrar: FlutterPluginRegistrar) {
    OcrHostApiSetup.setUp(binaryMessenger: registrar.messenger(), api: KansoOcrPlugin())
  }

  func read(image: FlutterStandardTypedData) async throws -> String {
    let data = image.data
    // Vision blocks; keep it off the main thread and the Swift cooperative pool.
    return try await withCheckedThrowingContinuation { continuation in
      DispatchQueue.global(qos: .userInitiated).async {
        do {
          continuation.resume(returning: try OcrReader.read(data))
        } catch {
          continuation.resume(
            throwing: PigeonError(
              code: "unreadable", message: "The bytes are not an image Vision can read.",
              details: nil))
        }
      }
    }
  }

  func isAvailable() throws -> Bool { true }

  func engine() throws -> String { OcrReader.engine }
}
