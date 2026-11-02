// Kanso — receipt_sheet.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/ask_receipt.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';

/// Opens the lines a receipt points at over the answer: which note, which
/// lines, the lines themselves, and a button that opens the note there.
Future<void> showReceiptSheet(
  BuildContext context,
  AskReceipt receipt, {
  required VoidCallback onOpen,
}) {
  return showKansoBottomSheet<void>(
    context,
    builder: (sheet) => ReceiptSheet(
      receipt: receipt,
      onOpen: () {
        Navigator.of(sheet).maybePop();
        onOpen();
      },
    ),
  );
}

/// The body of [showReceiptSheet].
class ReceiptSheet extends StatelessWidget {
  const ReceiptSheet({required this.receipt, required this.onOpen, super.key});

  final AskReceipt receipt;
  final VoidCallback onOpen;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    return Padding(
      padding: const EdgeInsets.all(KansoSpacing.md),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Text(receipt.label, style: typography.heading2),
          const SizedBox(height: KansoSpacing.sm),
          Flexible(
            child: SingleChildScrollView(
              child: Container(
                padding: const EdgeInsets.all(KansoSpacing.sm),
                decoration: BoxDecoration(
                  color: colors.fill,
                  borderRadius: KansoRadii.mediumBorder,
                ),
                child: Text(receipt.quote, style: typography.body),
              ),
            ),
          ),
          const SizedBox(height: KansoSpacing.md),
          KansoButton(label: context.l10n.askOpenNote, onPressed: onOpen),
        ],
      ),
    );
  }
}
