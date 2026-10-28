// Kanso — list_options_sheet.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/models/data/list_options.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/status_label.dart';
import '../../../core/views/widgets/kanso_check_row.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';
import '../../state/note_list/note_list_bloc.dart';

/// How the list is sorted and narrowed, as the desktop's list menu has it:
/// what to sort by and which way, a status to narrow to, and pinned only.
/// Recent has no sort, and Pinned no "Pinned Only".
///
/// What is chosen applies at once, and the sheet stays until it is done
/// with.
class ListOptionsSheet extends StatelessWidget {
  const ListOptionsSheet({super.key});

  /// The statuses a list can be narrowed to, as the library writes them.
  static const List<String> statuses = <String>[
    'active',
    'onHold',
    'completed',
    'dropped',
  ];

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final bloc = context.read<NoteListBloc>();
    return BlocBuilder<NoteListBloc, NoteListState>(
      builder: (context, state) {
        final options = state.options;
        void choose(ListOptions next) =>
            bloc.add(NoteListEvent.optionsChanged(next));
        final titles = options.sort == SortKey.title;
        return ListView(
          shrinkWrap: true,
          padding: const EdgeInsets.only(bottom: KansoSpacing.sm),
          children: <Widget>[
            Padding(
              padding: const EdgeInsetsDirectional.only(
                start: KansoSpacing.md,
                top: KansoSpacing.xs,
              ),
              child: Row(
                children: <Widget>[
                  Expanded(
                    child: Text(
                      l10n.listOptionsTitle,
                      style: context.typography.barTitle,
                    ),
                  ),
                  KansoPressable(
                    onTap: () => Navigator.of(context).maybePop(),
                    child: Padding(
                      padding: const EdgeInsets.all(KansoSpacing.sm),
                      child: Text(
                        l10n.noteListDone,
                        style: context.typography.body.copyWith(
                          color: colors.accent,
                        ),
                      ),
                    ),
                  ),
                ],
              ),
            ),
            // The choices sit in rounded groups, as Settings draws them.
            if (ListOptions.sorts(state.place)) ...<Widget>[
              KansoSheetGroup(
                title: l10n.listOptionsSortBy,
                children: <Widget>[
                  KansoCheckRow(
                    label: l10n.listOptionsDateUpdated,
                    checked: options.sort == SortKey.updated,
                    onTap: () => choose(options.sortedBy(SortKey.updated)),
                  ),
                  KansoCheckRow(
                    label: l10n.listOptionsDateCreated,
                    checked: options.sort == SortKey.created,
                    onTap: () => choose(options.sortedBy(SortKey.created)),
                  ),
                  KansoCheckRow(
                    label: l10n.listOptionsTitleSort,
                    checked: titles,
                    onTap: () => choose(options.sortedBy(SortKey.title)),
                  ),
                ],
              ),
              // For titles the first choice is A to Z, ascending; for
              // dates it is the newest first, descending.
              KansoSheetGroup(
                children: <Widget>[
                  KansoCheckRow(
                    label: titles
                        ? l10n.listOptionsAToZ
                        : l10n.listOptionsNewestFirst,
                    checked: options.descending != titles,
                    onTap: () => choose(options.copyWith(descending: !titles)),
                  ),
                  KansoCheckRow(
                    label: titles
                        ? l10n.listOptionsZToA
                        : l10n.listOptionsOldestFirst,
                    checked: options.descending == titles,
                    onTap: () => choose(options.copyWith(descending: titles)),
                  ),
                ],
              ),
            ],
            KansoSheetGroup(
              title: l10n.listOptionsStatus,
              children: <Widget>[
                KansoCheckRow(
                  label: l10n.listOptionsAnyStatus,
                  checked: options.statusOnly == null,
                  onTap: () => choose(options.copyWith(statusOnly: null)),
                ),
                for (final status in statuses)
                  KansoCheckRow(
                    label: l10n.statusName(noteStatusOf(status)),
                    checked: options.statusOnly == status,
                    onTap: () => choose(options.copyWith(statusOnly: status)),
                  ),
              ],
            ),
            if (ListOptions.narrowsToPinned(state.place))
              KansoSheetGroup(
                children: <Widget>[
                  KansoCheckRow(
                    label: l10n.listOptionsPinnedOnly,
                    checked: options.pinnedOnly,
                    onTap: () => choose(
                      options.copyWith(pinnedOnly: !options.pinnedOnly),
                    ),
                  ),
                ],
              ),
            if (state.place is! ListPlace_Trash)
              KansoSheetGroup(
                children: <Widget>[
                  KansoSettingsRow(
                    icon: KansoIcons.layoutTemplate,
                    label: l10n.listOptionsBoard,
                    chevron: true,
                    onTap: () {
                      final router = GoRouter.of(context);
                      final location = BrowseLocation.place(
                        state.place,
                        board: BoardKind.notes,
                      ).location;
                      Navigator.of(context).pop();
                      router.go(location);
                    },
                  ),
                ],
              ),
          ],
        );
      },
    );
  }
}
