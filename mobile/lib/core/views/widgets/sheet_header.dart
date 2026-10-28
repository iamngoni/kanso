// Kanso — sheet_header.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_pressable.dart';

/// The top line of a sheet: its [title] and a button at the end that closes
/// it ([actionLabel], "Done" or "Cancel").
class SheetHeader extends StatelessWidget {
  const SheetHeader({
    required this.title,
    required this.actionLabel,
    super.key,
  });

  final String title;
  final String actionLabel;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsetsDirectional.only(
        start: KansoSpacing.md,
        top: KansoSpacing.xs,
      ),
      child: Row(
        children: <Widget>[
          Expanded(child: Text(title, style: context.typography.barTitle)),
          KansoPressable(
            onTap: () => Navigator.of(context).maybePop(),
            child: Padding(
              padding: const EdgeInsets.all(KansoSpacing.sm),
              child: Text(
                actionLabel,
                style: context.typography.body.copyWith(
                  color: context.colors.accent,
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
