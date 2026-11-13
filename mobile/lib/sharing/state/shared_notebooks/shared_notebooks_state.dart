// Kanso — shared_notebooks_state.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'shared_notebooks_bloc.dart';

@freezed
abstract class SharedNotebooksState with _$SharedNotebooksState {
  const factory SharedNotebooksState({
    @Default(LoadStatus.loading) LoadStatus status,
    @Default(<SharedBookRow>[]) List<SharedBookRow> books,
    @Default(<BookInvitationRow>[]) List<BookInvitationRow> invitations,

    /// Invitations being accepted or declined, by share ID.
    @Default(<String>{}) Set<String> working,

    /// What to tell of notebooks that ended here, oldest first.
    @Default(<String>[]) List<String> notices,

    /// "Update Kanso to open this shared notebook.": the sync server said
    /// this Kanso is too old for shared notebooks. Null while it is happy.
    String? updateLine,

    /// The notebook just joined, to open.
    String? joined,
    ShareProblem? problem,
  }) = _SharedNotebooksState;
}
