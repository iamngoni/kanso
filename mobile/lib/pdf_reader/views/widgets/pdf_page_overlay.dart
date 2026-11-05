// Kanso — pdf_page_overlay.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/rendering.dart';

/// A layer drawn over a PDF page, in the page's own coordinates: the page's
/// picture, then its highlights, its selection and the search's hit, then
/// every overlay in turn.
///
/// It is the seam for non-interactive page decorations. Ink owns its pointer
/// events in the page view so it can track each touch and stylus sample.
// A one-method interface is the whole point here: a later tool adds one.
// ignore: one_member_abstracts
abstract class PdfPageOverlay {
  const PdfPageOverlay();

  /// Draws this layer over a page [size] logical pixels wide and tall.
  void paint(Canvas canvas, Size size);
}
