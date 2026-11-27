// Kanso — new_notebook_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../models/data/new_notebook_failure.dart';

part 'new_notebook_bloc.freezed.dart';
part 'new_notebook_event.dart';
part 'new_notebook_state.dart';

/// Makes a notebook of the name a person typed, at the top of the library or
/// inside the notebook the sheet was opened from, and offers the starters to
/// make it from.
///
/// The name is trimmed, and an empty one is refused without asking the
/// library. A starter that makes notes is refused inside a locked notebook,
/// as the library would. A second request while one is being made is
/// ignored.
@injectable
class NewNotebookBloc extends Bloc<NewNotebookEvent, NewNotebookState> {
  NewNotebookBloc(this._library) : super(const NewNotebookState()) {
    on<NewNotebookOpenedEvent>(_onOpened);
    on<NewNotebookSubmittedEvent>(_onSubmitted, transformer: droppable());
  }

  final LibraryRepository _library;

  Future<void> _onOpened(
    NewNotebookOpenedEvent event,
    Emitter<NewNotebookState> emit,
  ) async {
    emit(
      state.copyWith(
        starters: _library.notebookStarters(),
        parentId: event.parentId,
      ),
    );
    final parent = event.parentId;
    if (parent == null) {
      return;
    }
    final books = await _library.notebooks();
    emit(
      state.copyWith(
        parentLocked: books.fold(
          (_) => false,
          (rows) => _lockedWithin(rows, parent),
        ),
      ),
    );
  }

  Future<void> _onSubmitted(
    NewNotebookSubmittedEvent event,
    Emitter<NewNotebookState> emit,
  ) async {
    final name = event.name.trim();
    if (name.isEmpty) {
      emit(state.copyWith(failure: NewNotebookFailure.invalidName));
      return;
    }
    final starter = state.starters
        .where((starter) => starter.kind == event.kind)
        .firstOrNull;
    if (state.parentLocked && (starter?.makesNotes ?? false)) {
      emit(state.copyWith(failure: NewNotebookFailure.lockedParent));
      return;
    }
    emit(state.copyWith(creating: true, failure: null));
    final made = await _library.createNotebook(
      name,
      parentId: state.parentId,
      kind: event.kind,
    );
    emit(
      made.fold(
        (error) => state.copyWith(
          creating: false,
          failure: error is InvalidError
              ? NewNotebookFailure.invalidName
              : NewNotebookFailure.failed,
        ),
        (notebook) => state.copyWith(creating: false, created: notebook),
      ),
    );
  }

  /// Whether the notebook [id], or one it is inside, has a lock of its own.
  static bool _lockedWithin(List<NotebookRow> rows, String id) {
    final byId = <String, NotebookRow>{for (final row in rows) row.id: row};
    for (
      var row = byId[id];
      row != null;
      row = row.parentId == null ? null : byId[row.parentId]
    ) {
      if (row.locked) {
        return true;
      }
    }
    return false;
  }
}
