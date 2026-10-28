// Kanso — search_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/data/search_scope_kind.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../../core/utils/debounce.dart';
import '../../../discovery/models/repos/abs/discovery_repository.dart';
import '../../models/data/search_status.dart';

part 'search_bloc.freezed.dart';
part 'search_event.dart';
part 'search_state.dart';

/// How long the person may pause typing before the notes are searched.
const Duration searchDebounce = Duration(milliseconds: 150);

/// How long the writes of a burst may be apart for the search to be run
/// again once, after the last.
const Duration searchChangesDebounce = Duration(milliseconds: 100);

/// How many hits a search asks for.
const int searchLimit = 50;

/// A search of the notes: for the words typed, once the person pauses, in
/// titles, tags or a notebook or all of them, and again when the library
/// changes. A locked note is found by its title and nothing else.
@injectable
class SearchBloc extends Bloc<SearchEvent, SearchState> {
  SearchBloc(this._library, this._discovery) : super(const SearchState()) {
    on<SearchStartedEvent>(_onStarted, transformer: restartable());
    on<SearchQueryChangedEvent>(
      (event, emit) {
        // Words typed and then taken back are not looked for.
        if (event.text != _typed) {
          return Future<void>.value();
        }
        return _run(emit, query: event.text);
      },
      transformer: debounceRestartable<SearchQueryChangedEvent>(searchDebounce),
    );
    on<SearchScopeChangedEvent>(
      (event, emit) => _run(emit, scope: event.scope),
      transformer: restartable(),
    );
    on<SearchClearedEvent>(
      (event, emit) => _run(emit, query: ''),
      transformer: restartable(),
    );
    on<SearchChangesReceivedEvent>(
      (event, emit) => _run(emit),
      transformer: debounceRestartable<SearchChangesReceivedEvent>(
        searchChangesDebounce,
      ),
    );
    on<SearchRetriedEvent>((event, emit) => _run(emit));
  }

  final LibraryRepository _library;
  final DiscoveryRepository _discovery;
  StreamSubscription<List<String>>? _changes;
  int _searching = 0;

  /// The words in the field as of the last event: what a search that
  /// waited out the pause must still be for.
  String _typed = '';

  @override
  void onEvent(SearchEvent event) {
    super.onEvent(event);
    switch (event) {
      case SearchStartedEvent(:final query):
        _typed = query;
      case SearchQueryChangedEvent(:final text):
        _typed = text;
      case SearchClearedEvent():
        _typed = '';
      case SearchScopeChangedEvent() ||
          SearchChangesReceivedEvent() ||
          SearchRetriedEvent():
    }
  }

  Future<void> _onStarted(
    SearchStartedEvent event,
    Emitter<SearchState> emit,
  ) async {
    await _changes?.cancel();
    _changes = _library.changes().listen(
      (ids) => add(SearchEvent.changesReceived(ids)),
    );
    emit(
      SearchState(
        query: event.query,
        scope: event.scope,
        notebookId: event.notebookId,
      ),
    );
    await _run(emit);
  }

  /// Searches for the state's words, with [query] and [scope] if given. A
  /// search that a later one overtook is dropped, so what is shown is never
  /// older than what was asked for.
  Future<void> _run(
    Emitter<SearchState> emit, {
    String? query,
    SearchScopeKind? scope,
  }) async {
    final searching = ++_searching;
    var next = state.copyWith(
      query: query ?? state.query,
      scope: scope ?? state.scope,
      meaningHits: query != null && query != state.query
          ? const <RelatedRow>[]
          : state.meaningHits,
    );
    if (!next.hasQuery) {
      emit(
        next.copyWith(
          status: SearchStatus.idle,
          hits: const <Hit>[],
          meaningHits: const <RelatedRow>[],
          error: null,
        ),
      );
      return;
    }
    // What is shown stays while the new search runs; with nothing shown yet
    // there is a search to wait for.
    next = next.copyWith(
      status: next.hits.isEmpty ? SearchStatus.searching : next.status,
    );
    emit(next);
    final results = await (
      _library.search(next.query, scope: next.libraryScope),
      _meaningHits(next.query),
    ).wait;
    if (searching != _searching || emit.isDone) {
      return;
    }
    results.$1.fold(
      (error) => emit(
        state.copyWith(
          status: SearchStatus.failed,
          error: error,
          meaningHits: results.$2,
        ),
      ),
      (hits) => emit(
        state.copyWith(
          status: SearchStatus.ready,
          hits: hits,
          meaningHits: results.$2,
          error: null,
        ),
      ),
    );
  }

  Future<List<RelatedRow>> _meaningHits(String text) async {
    final query = _discovery.meaningQuery(text);
    if (query == null) return const <RelatedRow>[];
    final enabled = await _discovery.relatedEnabled();
    if (enabled.fold((_) => false, (value) => value) != true) {
      return const <RelatedRow>[];
    }
    final result = await _discovery.searchByMeaning(query, k: 20);
    return result.fold(
      (_) => const <RelatedRow>[],
      (value) => switch (value) {
        MeaningResult_Found(:final rows) => rows,
        MeaningResult_Off() || MeaningResult_NoModel() => const <RelatedRow>[],
      },
    );
  }

  @override
  Future<void> close() async {
    await _changes?.cancel();
    return super.close();
  }
}
