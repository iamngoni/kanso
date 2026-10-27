// Kanso — new_note_button.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_shadows.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';

/// The round button afloat in the corner of a phone's note list that starts
/// a new note. Its shadow is warm and soft so it lifts off the list
/// without a grey smudge.
class NewNoteButton extends StatelessWidget {
  const NewNoteButton({required this.onPressed, super.key});

  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return KansoPressable(
      onTap: onPressed,
      semanticLabel: context.l10n.noteListNewNote,
      child: Container(
        width: KansoSizes.floatingButton,
        height: KansoSizes.floatingButton,
        alignment: Alignment.center,
        decoration: BoxDecoration(
          color: colors.button,
          shape: BoxShape.circle,
          boxShadow: KansoShadows.raised(Theme.brightnessOf(context)),
        ),
        child: KansoIcon(
          KansoIcons.plus,
          size: KansoSizes.icon + 4,
          color: colors.buttonText,
        ),
      ),
    );
  }
}
