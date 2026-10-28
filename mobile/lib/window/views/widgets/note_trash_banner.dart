// Kanso — note_trash_banner.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';

/// The strip under a note window's bar when its note is in the trash, with a
/// way to take it out (the desktop editor's trash banner).
class NoteTrashBanner extends StatelessWidget {
  const NoteTrashBanner({required this.onRestore, super.key});

  final VoidCallback onRestore;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    return Semantics(
      container: true,
      liveRegion: true,
      child: ColoredBox(
        color: colors.fill,
        child: Padding(
          padding: const EdgeInsets.symmetric(
            horizontal: KansoSpacing.md,
            vertical: KansoSpacing.xs,
          ),
          child: Row(
            children: <Widget>[
              KansoIcon(KansoIcons.trash, color: colors.textSecondary),
              const SizedBox(width: KansoSpacing.xs),
              Expanded(
                child: Text(
                  l10n.noteWindowInTrash,
                  style: context.typography.body.copyWith(
                    color: colors.textSecondary,
                  ),
                ),
              ),
              KansoPressable(
                onTap: onRestore,
                child: Padding(
                  padding: const EdgeInsets.all(KansoSpacing.xs),
                  child: Text(
                    l10n.noteWindowRestore,
                    style: context.typography.body.copyWith(
                      color: colors.accent,
                    ),
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
