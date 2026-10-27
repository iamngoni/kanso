// Kanso — messages.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:pigeon/pigeon.dart';

/// The typed channel to the native side. Regenerate with
/// `dart run pigeon --input pigeons/messages.dart` in this folder; the
/// generated files are committed.
@ConfigurePigeon(
  PigeonOptions(
    dartOut: 'lib/src/messages.g.dart',
    kotlinOut: 'android/src/main/kotlin/app/kanso/kanso_ocr/Messages.g.kt',
    kotlinOptions: KotlinOptions(package: 'app.kanso.kanso_ocr'),
    swiftOut: 'ios/kanso_ocr/Sources/kanso_ocr/Messages.g.swift',
  ),
)
@HostApi()
abstract class OcrHostApi {
  /// The words in [image] (PNG, JPEG, or HEIC/HEIF bytes), one recognized
  /// line per line, top to bottom; empty when it has none. Bytes that are no
  /// image at all are an `unreadable` error.
  @async
  String read(Uint8List image);

  /// Whether the device has a recognizer that can run now.
  bool isAvailable();

  /// The name of the recognizer: `vision` on iOS, `mlkit` on Android.
  String engine();
}
