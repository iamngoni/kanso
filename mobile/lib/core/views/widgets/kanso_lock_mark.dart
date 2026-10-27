// Kanso — kanso_lock_mark.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_icons.dart';
import '../../configs/kanso_sizes.dart';
import '../../utils/context_extensions.dart';
import 'kanso_icon.dart';

/// The mark of a locked note or notebook in a list: a closed lock, or an
/// open one while the locked notes are open. A screen reader says
/// "Locked" or "Unlocked".
class KansoLockMark extends StatelessWidget {
  const KansoLockMark({
    this.unlocked = false,
    this.size = KansoSizes.glyph,
    super.key,
  });

  final bool unlocked;
  final double size;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return Semantics(
      label: unlocked ? l10n.noteRowUnlocked : l10n.noteRowLocked,
      child: ExcludeSemantics(
        child: KansoIcon(
          unlocked ? KansoIcons.lockOpen : KansoIcons.lock,
          size: size,
          color: context.colors.textMuted,
        ),
      ),
    );
  }
}
