// Kanso — notebook_tint.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

/// The colour a notebook keeps as stored (`#rrggbb`) as a [Color], or null
/// when it keeps none or the text isn't one.
Color? notebookColor(String? text) {
  final trimmed = text?.trim();
  if (trimmed == null || trimmed.isEmpty) {
    return null;
  }
  final digits = trimmed.startsWith('#') ? trimmed.substring(1) : trimmed;
  if (!RegExp(r'^[0-9a-fA-F]+$').hasMatch(digits)) {
    return null;
  }
  if (digits.length == 6) {
    return Color(0xFF000000 | int.parse(digits, radix: 16));
  }
  if (digits.length == 3) {
    final full = digits.split('').map((c) => '$c$c').join();
    return Color(0xFF000000 | int.parse(full, radix: 16));
  }
  return null;
}
