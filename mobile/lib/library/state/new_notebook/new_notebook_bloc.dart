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

/// Makes a notebook of the name a person typed, at the top of the library.
///
/// The name is trimmed, and an empty one is refused without asking the
/// library. A second request while one is being made is ignored.
@injectable
class NewNotebookBloc extends Bloc<NewNotebookEvent, NewNotebookState> {
  NewNotebookBloc(this._library) : super(const NewNotebookState()) {
    on<NewNotebookSubmittedEvent>(_onSubmitted, transformer: droppable());
  }

  final LibraryRepository _library;

  Future<void> _onSubmitted(
    NewNotebookSubmittedEvent event,
    Emitter<NewNotebookState> emit,
  ) async {
    final name = event.name.trim();
    if (name.isEmpty) {
      emit(const NewNotebookState(failure: NewNotebookFailure.invalidName));
      return;
    }
    emit(const NewNotebookState(creating: true));
    final made = await _library.createNotebook(name);
    emit(
      made.fold(
        (error) => NewNotebookState(
          failure: error is InvalidError
              ? NewNotebookFailure.invalidName
              : NewNotebookFailure.failed,
        ),
        (notebook) => NewNotebookState(created: notebook),
      ),
    );
  }
}
