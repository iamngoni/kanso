// Kanso — run_style.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/painting.dart';

/// The style a styled run asks for: only what it names, over the plain
/// text.
///
/// [color] is 24-bit RGB (the high byte is not read), [weight] is a CSS
/// weight (100 to 900) and [italic] is null for "not named".
TextStyle runStyle({int? color, bool? italic, int? weight}) {
  return TextStyle(
    color: color == null ? null : Color(0xFF000000 | color),
    fontStyle: italic ?? false ? FontStyle.italic : null,
    fontWeight: weight == null
        ? null
        : FontWeight.values[((weight / 100).round() - 1).clamp(0, 8)],
  );
}
