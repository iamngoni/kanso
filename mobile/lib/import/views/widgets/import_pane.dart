// Kanso: import_pane.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../models/data/import_stage.dart';
import '../../state/import_flow/import_flow_bloc.dart';
import 'import_failed_view.dart';
import 'import_progress_view.dart';
import 'import_report_view.dart';
import 'import_sources_view.dart';
import 'import_summary_view.dart';

/// Import notes: a back button and the step the flow is at (the sources, a
/// progress view while it reads or writes, what the export holds, what the
/// import did, or what went wrong). It needs an [ImportFlowBloc] above it.
///
/// While the notes are being written the page can't be left, so the person
/// is not shown a half-written import as finished.
class ImportPane extends StatelessWidget {
  const ImportPane({super.key});

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return BlocBuilder<ImportFlowBloc, ImportFlowState>(
      buildWhen: (before, now) => before.stage != now.stage,
      builder: (context, state) {
        return PopScope(
          canPop: state.stage != ImportStage.importing,
          child: ColoredBox(
            color: colors.app,
            child: Column(
              children: <Widget>[
                KansoTopBar(
                  backgroundColor: colors.app,
                  bordered: false,
                  leading: state.stage == ImportStage.importing
                      ? null
                      : const ShellBackButton(),
                  title: context.l10n.importTitle,
                ),
                Expanded(
                  child: switch (state.stage) {
                    ImportStage.sources => const ImportSourcesView(),
                    ImportStage.reading ||
                    ImportStage.importing => const ImportProgressView(),
                    ImportStage.summary => const ImportSummaryView(),
                    ImportStage.done => const ImportReportView(),
                    ImportStage.failed => const ImportFailedView(),
                  },
                ),
              ],
            ),
          ),
        );
      },
    );
  }
}
