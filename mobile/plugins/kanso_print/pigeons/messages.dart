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
    kotlinOut: 'android/src/main/kotlin/app/kanso/kanso_print/Messages.g.kt',
    kotlinOptions: KotlinOptions(package: 'app.kanso.kanso_print'),
    swiftOut: 'ios/kanso_print/Sources/kanso_print/Messages.g.swift',
  ),
)
/// The paper sizes the native side knows.
enum PrintPaper { a4, letter }

@HostApi()
abstract class PrintHostApi {
  /// Lays `html` out offscreen and writes a paged PDF to `outputPath`, with
  /// `marginPt` margins on every side. Returns the page count.
  @async
  int htmlToPdf(
    String html,
    String outputPath,
    PrintPaper paper,
    double marginPt,
  );
}
