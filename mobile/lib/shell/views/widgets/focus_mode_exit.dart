// Kanso — focus_mode_exit.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../../../core/configs/kanso_colors.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';

/// The way out of focus mode, at the top right and above the note.
///
/// The note's own bar is hidden in focus mode, so this is the one control
/// the mode leaves on screen. It sits in the safe area and has the raised
/// colour, so it stays readable over text.
class FocusModeExit extends StatelessWidget {
  const FocusModeExit({required this.onExit, super.key});

  final VoidCallback onExit;

  @override
  Widget build(BuildContext context) {
    final KansoColors colors = context.colors;
    return SafeArea(
      child: Align(
        alignment: Alignment.topRight,
        child: Padding(
          padding: const EdgeInsets.all(KansoSpacing.xs),
          child: DecoratedBox(
            decoration: BoxDecoration(
              color: colors.elevated,
              borderRadius: KansoRadii.mediumBorder,
              border: Border.all(color: colors.divider),
            ),
            child: KansoIconButton(
              icon: KansoIcons.close,
              semanticLabel: context.l10n.focusModeExit,
              onPressed: onExit,
            ),
          ),
        ),
      ),
    );
  }
}
