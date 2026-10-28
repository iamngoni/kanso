// Kanso — palette_sheet.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/linking_types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_inline_message.dart';
import '../../../core/views/widgets/kanso_search_input.dart';
import '../../../core/views/widgets/kanso_section_header.dart';
import '../../../injection.dart';
import '../../models/data/palette_status.dart';
import '../../state/palette/palette_bloc.dart';
import 'palette_row.dart';

/// Shows the command palette over the whole app, for what [paletteContext]
/// says the app can do now, and calls [onSelected] with the thing chosen.
///
/// The sheet is above the pages' own providers: it makes its own
/// [PaletteBloc] and closes it when it goes.
Future<void> showKansoPalette(
  BuildContext context, {
  required PaletteContext paletteContext,
  required ValueChanged<PaletteTarget> onSelected,
}) async {
  final bloc = getIt<PaletteBloc>()..add(PaletteEvent.started(paletteContext));
  try {
    await showKansoBottomSheet<void>(
      context,
      builder: (_) => BlocProvider<PaletteBloc>.value(
        value: bloc,
        child: PaletteSheet(onSelected: onSelected),
      ),
    );
  } finally {
    unawaited(bloc.close());
  }
}

/// The palette: a field to type in, then the commands the app can run, the
/// places of the library and the notes that match, in sections.
class PaletteSheet extends StatelessWidget {
  const PaletteSheet({required this.onSelected, super.key});

  final ValueChanged<PaletteTarget> onSelected;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final blocs = context.read<PaletteBloc>();
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Padding(
          padding: const EdgeInsets.fromLTRB(
            KansoSpacing.md,
            KansoSpacing.sm,
            KansoSpacing.md,
            KansoSpacing.sm,
          ),
          child: KansoSearchInput(
            hint: l10n.paletteHint,
            autofocus: true,
            clearLabel: l10n.searchClear,
            onChanged: (query) => blocs.add(PaletteEvent.queryChanged(query)),
          ),
        ),
        Flexible(
          child: BlocBuilder<PaletteBloc, PaletteState>(
            builder: (context, state) {
              final problem = state.problem;
              if (problem != null) {
                return Padding(
                  padding: const EdgeInsets.all(KansoSpacing.md),
                  child: KansoInlineMessage(message: problem),
                );
              }
              if (state.isEmpty) {
                return Padding(
                  padding: const EdgeInsets.all(KansoSpacing.md),
                  child: KansoEmptyState(
                    title: l10n.paletteEmptyTitle,
                    message: l10n.paletteEmptyMessage,
                  ),
                );
              }
              return _PaletteList(
                sections: state.sections,
                loading: state.status == PaletteStatus.loading,
                onSelected: (target) {
                  Navigator.of(context).pop();
                  onSelected(target);
                },
              );
            },
          ),
        ),
      ],
    );
  }
}

/// The sections and their rows, in one list.
class _PaletteList extends StatelessWidget {
  const _PaletteList({
    required this.sections,
    required this.loading,
    required this.onSelected,
  });

  final List<PaletteSection> sections;
  final bool loading;
  final ValueChanged<PaletteTarget> onSelected;

  @override
  Widget build(BuildContext context) {
    final rows = <Widget>[];
    for (final section in sections) {
      if (section.entries.isEmpty) {
        continue;
      }
      rows.add(KansoSectionHeader(label: section.label));
      for (final entry in section.entries) {
        rows.add(
          PaletteRow(
            entry: entry,
            onTap: () => onSelected(entry.target),
          ),
        );
      }
    }
    return ListView(
      shrinkWrap: true,
      padding: const EdgeInsets.only(bottom: KansoSpacing.sm),
      children: rows,
    );
  }
}
