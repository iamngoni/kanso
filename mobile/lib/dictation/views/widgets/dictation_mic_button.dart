// Kanso — dictation_mic_button.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_motion.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';

/// The large microphone button of the voice note: on while it listens, with
/// a halo that swells with the voice.
class DictationMicButton extends StatelessWidget {
  const DictationMicButton({
    required this.listening,
    required this.level,
    required this.onPressed,
    super.key,
  });

  final bool listening;

  /// How loud the voice is, 0 to 1.
  final double level;

  /// Null while the button cannot be used.
  final VoidCallback? onPressed;

  /// The button's diameter.
  static const double diameter = 88;

  /// How far the halo reaches past the button at full voice.
  static const double halo = 20;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final label = listening
        ? context.l10n.dictationMicStop
        : context.l10n.dictationMicStart;
    final reach = listening ? halo * level.clamp(0, 1) : 0.0;
    return KansoPressable(
      onTap: onPressed,
      semanticLabel: label,
      child: SizedBox.square(
        dimension: diameter + 2 * halo + KansoSpacing.xs,
        child: Center(
          child: AnimatedContainer(
            duration: KansoMotion.resolve(context, KansoMotion.fast),
            curve: KansoMotion.standard,
            width: diameter + 2 * reach,
            height: diameter + 2 * reach,
            alignment: Alignment.center,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              color: colors.accent.withValues(alpha: listening ? 0.22 : 0),
            ),
            child: Container(
              width: diameter,
              height: diameter,
              alignment: Alignment.center,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: listening ? colors.accent : colors.button,
              ),
              child: KansoIcon(
                KansoIcons.mic,
                size: 36,
                color: colors.buttonText,
              ),
            ),
          ),
        ),
      ),
    );
  }
}
