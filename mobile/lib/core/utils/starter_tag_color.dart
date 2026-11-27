// Kanso — starter_tag_color.dart
// Created 2026-11-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import '../bridge/api/types.dart';

/// A starter tag's colour as a [Color].
extension StarterTagColor on StarterTagInfo {
  /// The colour the tag will be made with (`0xRRGGBB`), opaque; drawn calmed.
  Color get tint => Color(0xFF000000 | color);
}
