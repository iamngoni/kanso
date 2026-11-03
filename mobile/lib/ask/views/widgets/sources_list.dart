// Kanso — sources_list.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/ask_receipt.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../utils/superscript.dart';

/// "Sources": one row for each receipt of an answer, its number, the note,
/// the lines and the day the note last changed. A tap shows the lines.
class SourcesList extends StatelessWidget {
  const SourcesList({
    required this.receipts,
    required this.onOpen,
    super.key,
  });

  final List<AskReceipt> receipts;
  final ValueChanged<AskReceipt> onOpen;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Padding(
          padding: const EdgeInsets.only(top: KansoSpacing.xs),
          child: Text(
            context.l10n.askSources,
            style: typography.sectionLabel,
          ),
        ),
        for (final receipt in receipts)
          KansoPressable(
            onTap: () => onOpen(receipt),
            semanticLabel: receipt.label,
            child: SizedBox(
              height: KansoSizes.touchTarget,
              child: Row(
                children: <Widget>[
                  SizedBox(
                    width: KansoSpacing.lg,
                    child: Text(
                      superscript(receipt.n),
                      style: typography.body.copyWith(color: colors.accent),
                    ),
                  ),
                  KansoIcon(
                    KansoIcons.fileText,
                    size: KansoSizes.iconSmall,
                    color: colors.textSecondary,
                  ),
                  const SizedBox(width: KansoSpacing.xs),
                  Expanded(
                    child: Text(
                      receipt.label,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: typography.body.copyWith(color: colors.accent),
                    ),
                  ),
                ],
              ),
            ),
          ),
      ],
    );
  }
}
