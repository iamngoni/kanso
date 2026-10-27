// Kanso — kanso_print.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/services.dart';

import 'src/messages.g.dart';

/// Paper sizes, in PDF points (1/72 inch).
enum KansoPaper {
  a4(595.28, 841.89),
  letter(612, 792);

  const KansoPaper(this.widthPt, this.heightPt);

  /// The page width in points.
  final double widthPt;

  /// The page height in points.
  final double heightPt;
}

/// The countries whose paper is US Letter.
const Set<String> letterCountries = <String>{
  'US',
  'CA',
  'PH',
  'MX',
  'CL',
  'CO',
  'VE',
  'GT',
  'PR',
};

/// The paper for a locale's country code: US Letter in [letterCountries],
/// A4 everywhere else (and when the country is unknown). Case-insensitive.
KansoPaper paperFor(String? countryCode) {
  if (countryCode == null) return KansoPaper.a4;
  return letterCountries.contains(countryCode.trim().toUpperCase())
      ? KansoPaper.letter
      : KansoPaper.a4;
}

/// The page margin, in points (half an inch).
const double kansoPrintMarginPt = 36;

/// Makes backgrounds print: both web engines drop them from print output
/// unless the page asks for exact colours. Added inside `<head>` when there
/// is one (so the document's doctype and mode are untouched), else appended.
String withPrintStyles(String html) {
  const style =
      '<style>*{-webkit-print-color-adjust:exact;print-color-adjust:exact}'
      '</style>';
  final head = RegExp('</head>', caseSensitive: false).firstMatch(html);
  if (head == null) return '$html$style';
  return html.replaceRange(head.start, head.start, style);
}

/// Prints a self-contained HTML page to a paged PDF file.
abstract final class KansoPrint {
  static PrintHostApi _api = PrintHostApi();

  /// How long Dart waits in all; the native side stops its layout at 60 s.
  static const Duration _backstop = Duration(seconds: 75);

  /// Replaces the channel. For tests.
  // ignore: use_setters_to_change_properties
  static void debugApi(PrintHostApi api) => _api = api;

  /// Lays out [html] offscreen (no JavaScript is needed: pages have none) and
  /// writes a paged PDF to [outputPath], on [paper] with 36 pt margins and
  /// backgrounds printed. Returns the page count.
  ///
  /// Throws a [PlatformException] if the layout fails or takes over a minute.
  static Future<int> htmlToPdf({
    required String html,
    required String outputPath,
    required KansoPaper paper,
  }) async {
    try {
      return await _api
          .htmlToPdf(
            withPrintStyles(html),
            outputPath,
            switch (paper) {
              KansoPaper.a4 => PrintPaper.a4,
              KansoPaper.letter => PrintPaper.letter,
            },
            kansoPrintMarginPt,
          )
          .timeout(_backstop);
    } on TimeoutException {
      throw PlatformException(
        code: 'timeout',
        message: 'The page took too long to lay out.',
      );
    }
  }
}
