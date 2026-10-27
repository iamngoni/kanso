// Kanso — kanso_ocr.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';

import 'src/messages.g.dart';

export 'src/messages.g.dart' show OcrHostApi;

/// Reads the words in a picture.
abstract interface class TextReader {
  /// The name of the recognizer, as stored beside the text it read:
  /// `vision` (Apple Vision) or `mlkit` (Google ML Kit).
  String get engine;

  /// The words in [image] (PNG, JPEG or HEIC bytes), one recognized line per
  /// line, top to bottom; an empty string when it has none.
  ///
  /// Bytes that are no image throw a `PlatformException` whose code is
  /// [KansoOcr.unreadable]; the caller treats that as "no words".
  Future<String> read(Uint8List image);

  /// Whether this device can read text now.
  Future<bool> get available;
}

/// The platform's on-device text recognizer: Vision on iOS, ML Kit on
/// Android (its bundled Latin model, so no download and no network).
///
/// The picture's bytes never leave the device, and no permission is needed:
/// they come from the app.
class KansoOcr implements TextReader {
  KansoOcr({OcrHostApi? api}) : api = api ?? OcrHostApi();

  /// The `PlatformException` code for bytes that cannot be decoded.
  static const unreadable = 'unreadable';

  /// The channel to the native side; replaced in tests.
  final OcrHostApi api;

  /// The recognizer's name, known without asking the native side (which
  /// answers the same through [OcrHostApi.engine]).
  @override
  String get engine => defaultTargetPlatform == TargetPlatform.iOS
      ? 'vision'
      : 'mlkit';

  @override
  Future<String> read(Uint8List image) => api.read(image);

  @override
  Future<bool> get available async {
    try {
      return await api.isAvailable();
    } on Object {
      return false;
    }
  }
}
