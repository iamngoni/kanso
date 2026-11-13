// Kanso — when_hint_chip.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../controllers/when_assist.dart';

/// The hint above the keyboard that shows the date found at the end of the
/// task being typed (`Fri 23 Oct, 15:00 · every week`). Tapping it writes
/// the canonical token in place of the phrase; typing on leaves the text
/// alone, and the hint goes with the phrase.
///
/// It takes no height while there is no hint.
class WhenHintBar extends StatelessWidget {
  const WhenHintBar({required this.assist, super.key});

  final WhenAssist assist;

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: assist,
      builder: (context, _) {
        final view = assist.hint;
        return AnimatedSize(
          duration: const Duration(milliseconds: 140),
          curve: Curves.easeOutCubic,
          alignment: Alignment.bottomCenter,
          child: view == null
              ? const SizedBox(width: double.infinity)
              : WhenHintChip(hint: view.hint, onTap: assist.apply),
        );
      },
    );
  }
}

/// The chip itself: the date as it reads, a repeat mark when it repeats,
/// and "Set" at the end.
class WhenHintChip extends StatelessWidget {
  const WhenHintChip({required this.hint, required this.onTap, super.key});

  final WhenHint hint;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    final l10n = context.l10n;
    // Taps on the chip are in the field, as far as its focus goes.
    return TextFieldTapRegion(
      child: Semantics(
        button: true,
        label: l10n.whenHintSemantics(hint.label),
        excludeSemantics: true,
        onTap: onTap,
        child: ColoredBox(
          color: colors.sidebar,
          child: Padding(
            padding: const EdgeInsets.symmetric(
              horizontal: KansoSpacing.sm,
              vertical: KansoSpacing.xs,
            ),
            child: Align(
              alignment: Alignment.centerLeft,
              child: KansoPressable(
                onTap: onTap,
                child: ConstrainedBox(
                  constraints: const BoxConstraints(
                    minHeight: KansoSizes.chip + KansoSpacing.xs,
                  ),
                  child: DecoratedBox(
                    decoration: BoxDecoration(
                      color: colors.accent.withValues(alpha: 0.12),
                      borderRadius: KansoRadii.pillBorder,
                      border: Border.all(
                        color: colors.accent.withValues(alpha: 0.28),
                      ),
                    ),
                    child: Padding(
                      padding: const EdgeInsets.only(
                        left: KansoSpacing.sm,
                        right: KansoSpacing.xs,
                      ),
                      child: Row(
                        mainAxisSize: MainAxisSize.min,
                        children: <Widget>[
                          KansoIcon(
                            hint.repeats
                                ? KansoIcons.refreshCw
                                : KansoIcons.calendar,
                            size: KansoSizes.iconSmall,
                            color: colors.accent,
                          ),
                          const SizedBox(width: KansoSpacing.xs),
                          Flexible(
                            child: Text(
                              hint.label,
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                              style: typography.label.copyWith(
                                color: colors.text,
                              ),
                            ),
                          ),
                          const SizedBox(width: KansoSpacing.sm),
                          DecoratedBox(
                            decoration: BoxDecoration(
                              color: colors.button,
                              borderRadius: KansoRadii.pillBorder,
                            ),
                            child: Padding(
                              padding: const EdgeInsets.symmetric(
                                horizontal: KansoSpacing.sm,
                                vertical: 3,
                              ),
                              child: Text(
                                l10n.whenHintAction,
                                style: typography.meta.copyWith(
                                  color: colors.buttonText,
                                  fontWeight: FontWeight.w600,
                                ),
                              ),
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
