// Kanso — theme_files_section.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../state/theme_picker/theme_picker_bloc.dart';
import 'theme_import_outcome_row.dart';

/// The Theme files section of the theme picker: the Import theme row, which
/// opens the system file picker, how the last import ended, and one warning
/// row for each theme file that was left out. It needs a [ThemePickerBloc]
/// above it.
class ThemeFilesSection extends StatelessWidget {
  const ThemeFilesSection({super.key});

  /// How far a row's hairline is inset: past its icon.
  static const double iconInset = 48;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocBuilder<ThemePickerBloc, ThemePickerState>(
      buildWhen: (before, after) =>
          before.importing != after.importing ||
          before.outcome != after.outcome ||
          before.notices != after.notices,
      builder: (context, picker) {
        final outcome = picker.outcome;
        void import() => context.read<ThemePickerBloc>().add(
          const ThemePickerEvent.importRequested(),
        );
        return KansoGroupedSection(
          title: l10n.themesFiles,
          footer: l10n.themesImportDetail,
          dividerIndent: iconInset,
          children: <Widget>[
            KansoSettingsRow(
              key: const ValueKey<String>('import-theme'),
              icon: KansoIcons.download,
              label: l10n.themesImport,
              trailing: KansoButton(
                label: picker.importing
                    ? l10n.themesImporting
                    : l10n.themesImportButton,
                variant: KansoButtonVariant.secondary,
                expanded: false,
                compact: true,
                onPressed: picker.importing ? null : import,
              ),
              onTap: picker.importing ? null : import,
            ),
            if (outcome != null)
              ThemeImportOutcomeRow(
                outcome: outcome,
                importedLabel: (names, _) => l10n.themesImportedOne(names),
                failedLabel: l10n.themesImportFailed,
              ),
            for (final notice in picker.notices)
              KansoSettingsRow(
                key: ValueKey<String>('theme-notice-${notice.file}'),
                icon: KansoIcons.triangleAlert,
                label: '${notice.file}: ${notice.sentence}',
              ),
          ],
        );
      },
    );
  }
}
