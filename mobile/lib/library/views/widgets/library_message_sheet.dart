// Kanso — library_message_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';

/// Says one thing and goes, for what a change to the library could not do.
class LibraryMessageSheet extends StatelessWidget {
  const LibraryMessageSheet({required this.message, super.key});

  final String message;

  /// Shows [message] over the whole app.
  static Future<void> show(BuildContext context, String message) {
    return showKansoBottomSheet<void>(
      context,
      builder: (_) => LibraryMessageSheet(message: message),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return Padding(
      padding: const EdgeInsetsDirectional.fromSTEB(
        KansoSpacing.md,
        KansoSpacing.md,
        KansoSpacing.md,
        KansoSpacing.md,
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Semantics(
            liveRegion: true,
            child: Text(message, style: context.typography.body),
          ),
          const SizedBox(height: KansoSpacing.md),
          KansoButton(
            label: l10n.noteListDone,
            variant: KansoButtonVariant.secondary,
            onPressed: () => Navigator.of(context).pop(),
          ),
        ],
      ),
    );
  }
}
