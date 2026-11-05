// Kanso — pdf_zoom.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/foundation.dart';

/// The reader's zoom, its own and apart from the app's text size: how wide a
/// page is drawn against the width of the screen. Fit width is the default.
@immutable
class PdfZoom {
  const PdfZoom._(this.factor, {required this.fitsWidth});

  /// The page as wide as the screen.
  static const PdfZoom fitWidth = PdfZoom._(1, fitsWidth: true);

  /// 50 % and 400 % of the fitted width.
  static const double min = 0.5;
  static const double max = 4;

  /// A page [factor] times the screen's width, held to [min]..[max].
  factory PdfZoom.of(double factor) => PdfZoom._(
    math.min(max, math.max(min, factor)),
    fitsWidth: factor == 1,
  );

  /// Pages' width as a multiple of the screen's.
  final double factor;

  /// Whether this is the fitted width (shown as "Fit width", not "100 %").
  final bool fitsWidth;

  /// Steps for the zoom's sheet: the levels a person picks.
  static const List<double> steps = <double>[0.5, 0.75, 1, 1.5, 2, 3, 4];

  @override
  bool operator ==(Object other) =>
      other is PdfZoom &&
      other.factor == factor &&
      other.fitsWidth == fitsWidth;

  @override
  int get hashCode => Object.hash(factor, fitsWidth);
}
