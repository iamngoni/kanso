// Kanso — pdf_page_overlay.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/rendering.dart';

/// A layer drawn over a PDF page, in the page's own coordinates: the page's
/// picture, then its highlights, its selection and the search's hit, then
/// every overlay in turn.
///
/// It is the seam a later feature uses to draw on a page without the reader
/// knowing about it — the pen and its strokes (issue #43, step 10) — the way
/// the Rust reader keeps an ink hook open. An overlay never receives
/// gestures; a tool that needs them draws its own widget over the page and
/// passes a picture of it here.
// A one-method interface is the whole point here: a later tool adds one.
// ignore: one_member_abstracts
abstract class PdfPageOverlay {
  const PdfPageOverlay();

  /// Draws this layer over a page [size] logical pixels wide and tall.
  void paint(Canvas canvas, Size size);
}
