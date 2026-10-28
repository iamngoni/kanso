// Kanso — new_note_button.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_effects.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_shadows.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';

/// The capsule that makes a new note, afloat at the foot of a phone's list:
/// the plus and "New note" on the button colour, centred, with a soft
/// shadow under it. Held, it offers the camera, photos and PDFs
/// ([onLongPress]).
class NewNoteButton extends StatelessWidget {
  const NewNoteButton({required this.onPressed, this.onLongPress, super.key});

  final VoidCallback onPressed;
  final VoidCallback? onLongPress;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final label = context.l10n.noteListNewNote;
    return KansoPressable(
      onTap: onPressed,
      onLongPress: onLongPress,
      semanticLabel: label,
      scale: KansoEffects.pressScale,
      child: Container(
        height: KansoSizes.floatingButton,
        padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.lg),
        decoration: BoxDecoration(
          color: colors.button,
          borderRadius: KansoRadii.pillBorder,
          boxShadow: KansoShadows.raised(Theme.brightnessOf(context)),
        ),
        child: ExcludeSemantics(
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: <Widget>[
              KansoIcon(KansoIcons.plus, color: colors.buttonText),
              const SizedBox(width: KansoSpacing.xs),
              Text(
                label,
                style: context.typography.listTitle.copyWith(
                  color: colors.buttonText,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
