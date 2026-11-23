// Kanso — book_sort_label.dart
// Created 2026-11-23.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../gen/l10n/app_localizations.dart';
import '../bridge/api/types.dart' show BookSort;

/// The words for the orders the library's notebooks can take, the same ones
/// the desktop's sidebar sort menu uses.
extension BookSortLabel on AppLocalizations {
  /// The name of [sort] in the notebooks' sort sheet.
  String bookSortName(BookSort sort) {
    return switch (sort) {
      BookSort.manual => libraryNotebooksSortManual,
      BookSort.nameAsc => libraryNotebooksSortNameAsc,
      BookSort.nameDesc => libraryNotebooksSortNameDesc,
      BookSort.updated => libraryNotebooksSortUpdated,
      BookSort.created => libraryNotebooksSortCreated,
    };
  }
}
