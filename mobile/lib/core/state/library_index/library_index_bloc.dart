// Kanso — library_index_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:dartz/dartz.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../bridge/api/types.dart';
import '../../models/data/application_error.dart';
import '../../models/data/library_index_status.dart';
import '../../models/repos/abs/library_repository.dart';
import '../../utils/debounce.dart';

part 'library_index_bloc.freezed.dart';
part 'library_index_event.dart';
part 'library_index_state.dart';

/// How long the library must be quiet before it is read again.
const Duration libraryIndexDebounce = Duration(milliseconds: 100);

/// The library's notebooks, tags and counts, read once and kept up to date,
/// for every screen that shows them: the library pane, the notes' lists, the
/// move sheet and a note's facts.
///
/// It reads again when the library reports a change, once the writes of a
/// burst have stopped, and keeps showing what it has meanwhile.
@injectable
class LibraryIndexBloc extends Bloc<LibraryIndexEvent, LibraryIndexState> {
  LibraryIndexBloc(this._library) : super(const LibraryIndexState()) {
    on<LibraryIndexStartedEvent>(_onStarted);
    on<LibraryIndexRetriedEvent>(
      (event, emit) => _read(emit),
      transformer: droppable(),
    );
    on<LibraryIndexChangesReceivedEvent>(
      (event, emit) => _read(emit),
      transformer: debounceRestartable<LibraryIndexChangesReceivedEvent>(
        libraryIndexDebounce,
      ),
    );
  }

  final LibraryRepository _library;
  StreamSubscription<List<String>>? _changes;
  int _reading = 0;

  Future<void> _onStarted(
    LibraryIndexStartedEvent event,
    Emitter<LibraryIndexState> emit,
  ) async {
    await _changes?.cancel();
    _changes = _library.changes().listen(
      (ids) => add(LibraryIndexEvent.changesReceived(ids)),
    );
    await _read(emit);
  }

  /// Reads the notebooks, tags and counts. A reading that a later one
  /// overtook is dropped, so the state never goes back to older data.
  Future<void> _read(Emitter<LibraryIndexState> emit) async {
    final reading = ++_reading;
    final results = await (
      _library.notebooks(),
      _library.tags(),
      _library.overview(),
      _library.overdueTasks(),
    ).wait;
    if (reading != _reading || emit.isDone) {
      return;
    }
    final (notebooks, tags, overview, overdue) = results;
    ApplicationError? failure;
    for (final result in <Either<ApplicationError, Object?>>[
      notebooks,
      tags,
      overview,
    ]) {
      result.fold((error) => failure ??= error, (_) {});
    }
    final error = failure;
    if (error != null) {
      emit(state.copyWith(status: LibraryIndexStatus.failed, error: error));
      return;
    }
    emit(
      LibraryIndexState(
        status: LibraryIndexStatus.ready,
        notebooks: notebooks.getOrElse(() => <NotebookRow>[]),
        tags: tags.getOrElse(() => <TagRow>[]),
        overview: overview.getOrElse(() => state.overview),
        // The badge is a nicety: a failure to count shows none.
        overdueTasks: overdue.getOrElse(() => 0),
      ),
    );
  }

  @override
  Future<void> close() async {
    await _changes?.cancel();
    return super.close();
  }
}
