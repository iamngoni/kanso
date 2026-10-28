// Kanso — library_index_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'library_index_bloc.dart';

@freezed
abstract class LibraryIndexState with _$LibraryIndexState {
  const factory LibraryIndexState({
    @Default(LibraryIndexStatus.loading) LibraryIndexStatus status,

    /// The notebooks in tree order, each with its depth and role.
    @Default(<NotebookRow>[]) List<NotebookRow> notebooks,
    @Default(<TagRow>[]) List<TagRow> tags,

    /// How many notes each place lists.
    @Default(LibraryOverview(all: 0, pinned: 0, recent: 0, trash: 0))
    LibraryOverview overview,

    /// How many open tasks are overdue: the badge of the Tasks row.
    @Default(0) int overdueTasks,

    /// Why the last reading failed, while [status] is failed.
    ApplicationError? error,
  }) = _LibraryIndexState;

  const LibraryIndexState._();

  /// The notebook [id], or null when there is none.
  NotebookRow? notebook(String id) {
    for (final notebook in notebooks) {
      if (notebook.id == id) {
        return notebook;
      }
    }
    return null;
  }

  /// The tag [id], or null when there is none.
  TagRow? tag(String id) {
    for (final tag in tags) {
      if (tag.id == id) {
        return tag;
      }
    }
    return null;
  }

  /// The name of the notebook [id], or null when there is none.
  String? notebookName(String id) => notebook(id)?.name;

  /// The names of the notebook [id] and the ones it is inside, outermost
  /// first, joined by " / " (`Projects / Kanso`); null when there is no such
  /// notebook.
  String? notebookPath(String id) {
    final names = <String>[];
    var current = notebook(id);
    // A notebook is inside at most as many others as there are notebooks;
    // the bound keeps a malformed tree from looping.
    for (var depth = 0; current != null && depth <= notebooks.length; depth++) {
      names.add(current.name);
      final parent = current.parentId;
      current = parent == null ? null : notebook(parent);
    }
    return names.isEmpty ? null : names.reversed.join(' / ');
  }
}
