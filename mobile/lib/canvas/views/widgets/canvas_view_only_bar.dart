// Kanso — canvas_view_only_bar.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';

/// What takes the toolbar's place on a canvas the person may only look at:
/// that it is view only, and how to change that. The canvas still pans and
/// zooms.
class CanvasViewOnlyBar extends StatelessWidget {
  const CanvasViewOnlyBar({super.key});

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    return DecoratedBox(
      decoration: BoxDecoration(
        color: colors.elevated,
        border: Border(top: BorderSide(color: colors.divider)),
      ),
      child: SafeArea(
        top: false,
        child: Padding(
          padding: const EdgeInsets.symmetric(
            horizontal: KansoSpacing.md,
            vertical: KansoSpacing.sm,
          ),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: <Widget>[
              Padding(
                padding: const EdgeInsets.only(top: 2),
                child: KansoIcon(
                  KansoIcons.eye,
                  size: KansoSizes.iconSmall,
                  color: colors.textSecondary,
                ),
              ),
              const SizedBox(width: KansoSpacing.xs),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: <Widget>[
                    Text(
                      l10n.canvasViewOnlyLabel,
                      style: context.typography.label,
                    ),
                    const SizedBox(height: 2),
                    Text(
                      l10n.canvasViewOnlyMessage,
                      style: context.typography.meta,
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
