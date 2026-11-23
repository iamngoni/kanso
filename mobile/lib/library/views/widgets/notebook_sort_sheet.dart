// Kanso — notebook_sort_sheet.dart
// Created 2026-11-23.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/book_sort_label.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_check_row.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';

/// The orders the library's notebooks can take, as the desktop's sort menu
/// lists them, the one in use checked: Default order, Name A to Z, Name Z
/// to A, Recently Updated and Recently Created.
///
/// A choice applies at once and is kept for next time; the sheet closes
/// with it. Notebooks are sorted inside each parent.
class NotebookSortSheet extends StatelessWidget {
  const NotebookSortSheet({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = context.read<LibraryIndexBloc>();
    return BlocBuilder<LibraryIndexBloc, LibraryIndexState>(
      buildWhen: (before, after) => before.bookSort != after.bookSort,
      builder: (context, state) {
        return ListView(
          shrinkWrap: true,
          padding: const EdgeInsets.only(bottom: KansoSpacing.sm),
          children: <Widget>[
            KansoSheetGroup(
              title: l10n.libraryNotebooksSortBy,
              children: <Widget>[
                for (final sort in BookSort.values)
                  KansoCheckRow(
                    label: l10n.bookSortName(sort),
                    checked: state.bookSort == sort,
                    onTap: () {
                      bloc.add(LibraryIndexEvent.bookSortChanged(sort));
                      Navigator.of(context).maybePop();
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
