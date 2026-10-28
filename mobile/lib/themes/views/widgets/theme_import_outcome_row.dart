// Kanso — theme_import_outcome_row.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../models/data/theme_import_outcome.dart';

/// The row that says how the last import ended: a check mark and what was
/// imported, or a warning and why it failed. It serves themes and icon
/// themes alike: the caller words the two sentences.
class ThemeImportOutcomeRow extends StatelessWidget {
  const ThemeImportOutcomeRow({
    required this.outcome,
    required this.importedLabel,
    required this.failedLabel,
    super.key,
  });

  final ThemeImportOutcome outcome;

  /// The sentence for a successful import of the given names and summary.
  final String Function(String names, String summary) importedLabel;

  /// The sentence for a failure for the given message.
  final String Function(String message) failedLabel;

  @override
  Widget build(BuildContext context) {
    return switch (outcome) {
      ThemeImportedOutcome(:final names, :final summary) => KansoSettingsRow(
        key: const ValueKey<String>('theme-import-outcome'),
        icon: KansoIcons.circleCheck,
        label: importedLabel(names.join(', '), summary).trim(),
      ),
      ThemeImportFailedOutcome(:final message) => KansoSettingsRow(
        key: const ValueKey<String>('theme-import-outcome'),
        icon: KansoIcons.triangleAlert,
        label: failedLabel(message),
        danger: true,
      ),
    };
  }
}
