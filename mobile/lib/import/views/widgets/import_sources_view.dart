// Kanso: import_sources_view.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../state/import_flow/import_flow_bloc.dart';
import '../../utils/import_labels.dart';
import 'import_kind_sheet.dart';
import 'import_source_row.dart';

/// The places notes can be imported from. Choosing one asks the system for
/// the file or folder (the sources that read either ask which first), then
/// the flow reads it. When the flow was opened with a file, the list is
/// narrowed to what that file could be and nothing is asked for. It needs an
/// [ImportFlowBloc] above it.
class ImportSourcesView extends StatelessWidget {
  const ImportSourcesView({super.key});

  Future<void> choose(BuildContext context, ImportSource source) async {
    final bloc = context.read<ImportFlowBloc>();
    final given = bloc.state.pick != null && !bloc.state.picked;
    if (given || !(source.picksFiles && source.picksFolders)) {
      bloc.add(
        ImportFlowEvent.sourceChosen(source.kind, folder: source.picksFolders),
      );
      return;
    }
    final folder = await ImportKindSheet.open(context, source);
    if (folder != null) {
      bloc.add(ImportFlowEvent.sourceChosen(source.kind, folder: folder));
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final bottom = MediaQuery.paddingOf(context).bottom;
    return BlocBuilder<ImportFlowBloc, ImportFlowState>(
      builder: (context, state) {
        final problem = state.problem;
        return ListView(
          padding: EdgeInsets.fromLTRB(
            KansoSpacing.md,
            KansoSpacing.xs,
            KansoSpacing.md,
            bottom + KansoSpacing.lg,
          ),
          children: <Widget>[
            Text(l10n.importIntro, style: context.typography.body),
            const SizedBox(height: KansoSpacing.md),
            if (problem != null) ...<Widget>[
              Semantics(
                liveRegion: true,
                child: Text(
                  l10n.importProblemText(problem),
                  style: context.typography.secondary.copyWith(
                    color: colors.danger,
                  ),
                ),
              ),
              const SizedBox(height: KansoSpacing.md),
            ],
            KansoGroupedSection(
              title: l10n.importSourcesLabel,
              children: <Widget>[
                for (final source in state.listed)
                  ImportSourceRow(
                    source: source,
                    onTap: state.picking ? null : () => choose(context, source),
                  ),
              ],
            ),
            if (state.picking) ...<Widget>[
              const SizedBox(height: KansoSpacing.md),
              Text(l10n.importPicking, style: context.typography.meta),
            ],
          ],
        );
      },
    );
  }
}
