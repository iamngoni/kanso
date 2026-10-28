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
    on<LibraryIndexPlaceChangedEvent>(_onPlaceChanged);
    on<LibraryIndexNotebookFoldedEvent>(_onNotebookFolded);
    on<LibraryIndexTagToggledEvent>(_onTagToggled);
    on<LibraryIndexAllTagsToggledEvent>(_onAllTagsToggled);
    on<LibraryIndexStatusGroupToggledEvent>(
      (event, emit) => emit(state.copyWith(statusOpen: !state.statusOpen)),
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
      _library.statuses(),
    ).wait;
    if (reading != _reading || emit.isDone) {
      return;
    }
    final (notebooks, tags, overview, overdue, statuses) = results;
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
    final read = state.copyWith(
      status: LibraryIndexStatus.ready,
      notebooks: notebooks.getOrElse(() => <NotebookRow>[]),
      tags: tags.getOrElse(() => <TagRow>[]),
      overview: overview.getOrElse(() => state.overview),
      // The badge is a nicety: a failure to count shows none.
      overdueTasks: overdue.getOrElse(() => 0),
      statuses: statuses.getOrElse(() => state.statuses),
      error: null,
    );
    // The tree reads the state it will be drawn with, not the one before.
    final tree = await _tree(read);
    if (reading != _reading || emit.isDone) {
      return;
    }
    emit(read.copyWith(visibleNotebooks: tree.$1, tagPlaces: tree.$2));
  }

  Future<void> _onPlaceChanged(
    LibraryIndexPlaceChangedEvent event,
    Emitter<LibraryIndexState> emit,
  ) async {
    if (state.place == event.place) {
      return;
    }
    final next = state;
    emit(next.copyWith(place: event.place));
    await _readTree(emit, from: next.copyWith(place: event.place));
  }

  /// A tap on notebook [id]'s arrow: the folds change, then the rows.
  Future<void> _onNotebookFolded(
    LibraryIndexNotebookFoldedEvent event,
    Emitter<LibraryIndexState> emit,
  ) async {
    final folds = await _library.toggleFold(
      state.folds,
      event.id,
      showing: _showingNotebook,
    );
    final next = folds.fold(
      (error) => state,
      (folds) => state.copyWith(folds: folds),
    );
    emit(next);
    await _readTree(emit, from: next);
  }

  /// A tap on the tags' "…" or "Fewer".
  Future<void> _onAllTagsToggled(
    LibraryIndexAllTagsToggledEvent event,
    Emitter<LibraryIndexState> emit,
  ) async {
    final next = state.copyWith(allTags: !state.allTags);
    emit(next);
    await _readTree(emit, from: next);
  }

  /// A tap on the arrow of a tag level.
  Future<void> _onTagToggled(
    LibraryIndexTagToggledEvent event,
    Emitter<LibraryIndexState> emit,
  ) async {
    final path = event.path.toLowerCase();
    final open = List<String>.of(state.openTags);
    if (!open.remove(path)) {
      open.add(path);
    }
    final next = state.copyWith(openTags: open);
    emit(next);
    await _readTree(emit, from: next);
  }

  /// Reads the notebook rows and the tag chips and rows for [from]'s folds,
  /// open levels and place.
  Future<void> _readTree(
    Emitter<LibraryIndexState> emit, {
    LibraryIndexState? from,
  }) async {
    final base = from ?? state;
    final tree = await _tree(base);
    if (emit.isDone) {
      return;
    }
    emit(base.copyWith(visibleNotebooks: tree.$1, tagPlaces: tree.$2));
  }

  /// The notebook rows and tag places [base] shows. A failure keeps what was
  /// there: the tree is a view of the library, not the library itself.
  Future<(List<VisibleNotebook>, TagPlaces)> _tree(
    LibraryIndexState base,
  ) async {
    final place = base.place;
    final showingTag = switch (place) {
      ListPlace_Tag(:final id) => base.tag(id)?.name,
      ListPlace_TagPath(:final path) => path,
      _ => null,
    };
    final results = await (
      _library.visibleNotebooks(
        base.folds,
        showing: place is ListPlace_Notebook ? place.id : null,
      ),
      _library.tagPlaces(
        all: base.allTags,
        selected: place is ListPlace_Tag ? place.id : null,
        openPaths: base.openTags,
        showing: showingTag,
      ),
    ).wait;
    return (
      results.$1.getOrElse(() => base.visibleNotebooks),
      results.$2.getOrElse(() => base.tagPlaces),
    );
  }

  /// The notebook whose notes are listed, if one is.
  String? get _showingNotebook {
    final place = state.place;
    return place is ListPlace_Notebook ? place.id : null;
  }

  @override
  Future<void> close() async {
    await _changes?.cancel();
    return super.close();
  }
}
