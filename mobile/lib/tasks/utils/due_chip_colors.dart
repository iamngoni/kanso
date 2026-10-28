// Kanso — due_chip_colors.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import '../../core/bridge/api/types.dart';
import '../../core/configs/kanso_colors.dart';

/// The text and fill of a due chip for [tone], as the desktop draws it
/// (`chip_tint`): a warm red once overdue, the link colour for today, quiet
/// otherwise and quieter once done. Pure.
(Color text, Color fill) dueChipColors(DueTone tone, KansoColors colors) =>
    switch (tone) {
      DueTone.overdue => (colors.danger, colors.danger.withValues(alpha: 0.16)),
      DueTone.today => (colors.link, colors.link.withValues(alpha: 0.16)),
      DueTone.plain => (colors.textSecondary, colors.fill),
      DueTone.done => (colors.textMuted, colors.fill.withValues(alpha: 0.6)),
    };
