// Kanso: import_report_view.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../state/import_flow/import_flow_bloc.dart';
import 'import_lines.dart';

/// What the import did: the library's own report, what it skipped or could
/// not read, a way to the first note it made and Done. It needs an
/// [ImportFlowBloc] above it.
class ImportReportView extends StatelessWidget {
  const ImportReportView({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bottom = MediaQuery.paddingOf(context).bottom;
    return BlocBuilder<ImportFlowBloc, ImportFlowState>(
      builder: (context, state) {
        final report = state.summary;
        if (report == null) {
          return const SizedBox.shrink();
        }
        final first = report.firstNote;
        return ListView(
          padding: EdgeInsets.fromLTRB(
            KansoSpacing.md,
            KansoSpacing.xs,
            KansoSpacing.md,
            bottom + KansoSpacing.lg,
          ),
          children: <Widget>[
            Semantics(
              liveRegion: true,
              child: Text(
                l10n.importDoneTitle,
                style: context.typography.heading2,
              ),
            ),
            const SizedBox(height: KansoSpacing.md),
            ImportLines(lines: report.lines),
            if (report.skipped.isNotEmpty) ...<Widget>[
              const SizedBox(height: KansoSpacing.md),
              ImportLines(
                title: l10n.importSkippedTitle,
                lines: report.skipped,
              ),
            ],
            if (report.rejected.isNotEmpty) ...<Widget>[
              const SizedBox(height: KansoSpacing.md),
              ImportLines(
                title: l10n.importRejectedTitle,
                lines: report.rejected,
              ),
            ],
            const SizedBox(height: KansoSpacing.lg),
            if (first != null) ...<Widget>[
              KansoButton(
                label: l10n.importOpenFirst,
                onPressed: () => context.go(
                  const BrowseLocation.place(
                    ListPlace.all(),
                  ).withNote(first).location,
                ),
              ),
              const SizedBox(height: KansoSpacing.xs),
            ],
            KansoButton(
              label: l10n.importDoneButton,
              variant: first == null
                  ? KansoButtonVariant.primary
                  : KansoButtonVariant.secondary,
              onPressed: () => context.pop(),
            ),
          ],
        );
      },
    );
  }
}
