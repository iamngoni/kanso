// Kanso: import_summary_view.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../state/import_flow/import_flow_bloc.dart';
import 'import_lines.dart';

/// What the export holds, before anything is written: the library's own
/// summary, what it leaves out, and Import. An export with no notes in it
/// says so and goes back to the sources. It needs an [ImportFlowBloc]
/// above it.
class ImportSummaryView extends StatelessWidget {
  const ImportSummaryView({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bottom = MediaQuery.paddingOf(context).bottom;
    return BlocBuilder<ImportFlowBloc, ImportFlowState>(
      builder: (context, state) {
        final summary = state.summary;
        final bloc = context.read<ImportFlowBloc>();
        final choose = KansoButton(
          label: l10n.importChooseAnother,
          variant: KansoButtonVariant.secondary,
          onPressed: () => bloc.add(const ImportFlowEvent.restarted()),
        );
        if (summary == null || summary.empty) {
          return KansoEmptyState(
            title: l10n.importSummaryEmptyTitle,
            message: l10n.importSummaryEmptyMessage(state.pick?.name ?? ''),
            action: choose,
          );
        }
        return ListView(
          padding: EdgeInsets.fromLTRB(
            KansoSpacing.md,
            KansoSpacing.xs,
            KansoSpacing.md,
            bottom + KansoSpacing.lg,
          ),
          children: <Widget>[
            Text(l10n.importSummaryTitle, style: context.typography.heading2),
            const SizedBox(height: KansoSpacing.xxs),
            Text(
              l10n.importSummaryFrom(state.pick?.name ?? ''),
              style: context.typography.secondary,
            ),
            const SizedBox(height: KansoSpacing.md),
            ImportLines(lines: summary.lines),
            if (summary.skipped.isNotEmpty) ...<Widget>[
              const SizedBox(height: KansoSpacing.md),
              ImportLines(
                title: l10n.importSkippedTitle,
                lines: summary.skipped,
              ),
            ],
            const SizedBox(height: KansoSpacing.lg),
            KansoButton(
              label: l10n.importButton,
              icon: KansoIcons.download,
              onPressed: () => bloc.add(const ImportFlowEvent.confirmed()),
            ),
            const SizedBox(height: KansoSpacing.xs),
            choose,
          ],
        );
      },
    );
  }
}
