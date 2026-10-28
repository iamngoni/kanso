// Kanso — kanso_status_glyph.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_colors.dart';
import '../../configs/kanso_icons.dart';
import '../../configs/kanso_sizes.dart';
import '../../models/data/note_status.dart';
import '../../utils/context_extensions.dart';
import 'kanso_icon.dart';

/// The glyph of a note's status, as on the desktop: Active is a dot in a
/// circle in the accent colour, On hold a pause in the warning colour,
/// Completed a check in the success colour, Dropped a cross in the quiet
/// text colour. A note with no status has none.
class KansoStatusGlyph extends StatelessWidget {
  const KansoStatusGlyph({
    required this.status,
    this.size = KansoSizes.glyph,
    super.key,
  });

  final NoteStatus status;
  final double size;

  /// The icon of [status], or null when it has no glyph.
  static KansoIcons? iconFor(NoteStatus status) {
    return switch (status) {
      NoteStatus.none => null,
      NoteStatus.active => KansoIcons.circleDot,
      NoteStatus.onHold => KansoIcons.circlePause,
      NoteStatus.completed => KansoIcons.circleCheck,
      NoteStatus.dropped => KansoIcons.circleX,
    };
  }

  /// The colour of [status] in [colors].
  static Color colorFor(NoteStatus status, KansoColors colors) {
    return switch (status) {
      NoteStatus.none => colors.textMuted,
      NoteStatus.active => colors.accent,
      NoteStatus.onHold => colors.warning,
      NoteStatus.completed => colors.success,
      NoteStatus.dropped => colors.textMuted,
    };
  }

  @override
  Widget build(BuildContext context) {
    final icon = iconFor(status);
    if (icon == null) {
      return const SizedBox.shrink();
    }
    return KansoIcon(icon, size: size, color: colorFor(status, context.colors));
  }
}
