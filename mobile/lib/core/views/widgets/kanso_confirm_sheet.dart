// Kanso — kanso_confirm_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_bottom_sheet_route.dart';
import 'kanso_button.dart';
import 'kanso_pressable.dart';

/// Asks a yes or no about something in the library that cannot be undone,
/// with the words the desktop's confirmation has: a title, what it does and
/// the button that does it.
class KansoConfirmSheet extends StatelessWidget {
  const KansoConfirmSheet({
    required this.title,
    required this.message,
    required this.confirmLabel,
    this.destructive = true,
    super.key,
  });

  final String title;
  final String message;
  final String confirmLabel;

  /// Whether the button that does it is drawn in the danger colour.
  final bool destructive;

  /// Shows the question over the whole app; true when the person confirmed.
  static Future<bool> show(
    BuildContext context, {
    required String title,
    required String message,
    required String confirmLabel,
    bool destructive = true,
  }) async {
    final done = await showKansoBottomSheet<bool>(
      context,
      builder: (_) => KansoConfirmSheet(
        title: title,
        message: message,
        confirmLabel: confirmLabel,
        destructive: destructive,
      ),
    );
    return done ?? false;
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    return Padding(
      padding: const EdgeInsetsDirectional.fromSTEB(
        KansoSpacing.md,
        KansoSpacing.xs,
        KansoSpacing.md,
        KansoSpacing.md,
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Row(
            children: <Widget>[
              Expanded(
                child: Text(title, style: context.typography.barTitle),
              ),
              KansoPressable(
                onTap: () => Navigator.of(context).pop(false),
                child: Padding(
                  padding: const EdgeInsets.all(KansoSpacing.sm),
                  child: Text(
                    l10n.searchCancel,
                    style: context.typography.body.copyWith(
                      color: colors.accent,
                    ),
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: KansoSpacing.xs),
          Text(message, style: context.typography.body),
          const SizedBox(height: KansoSpacing.md),
          KansoButton(
            label: confirmLabel,
            variant: destructive
                ? KansoButtonVariant.danger
                : KansoButtonVariant.primary,
            onPressed: () => Navigator.of(context).pop(true),
          ),
        ],
      ),
    );
  }
}
