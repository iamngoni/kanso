// Kanso — browse_blocs.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:clock/clock.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart' show addTearDown;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/library_index_status.dart';
import 'package:kanso/core/models/data/search_scope_kind.dart';
import 'package:kanso/core/models/repos/abs/haptics_repository.dart';
import 'package:kanso/core/state/library_index/library_index_bloc.dart';
import 'package:kanso/core/state/list_navigation/list_navigation_bloc.dart';
import 'package:kanso/core/state/navigation_history/navigation_history_bloc.dart';
import 'package:kanso/core/state/sync_status/sync_status_bloc.dart';
import 'package:kanso/note/models/data/note_screen_status.dart';
import 'package:kanso/note/models/repos/impl/memory_note_place_repository.dart';
import 'package:kanso/note/state/note/note_bloc.dart';
import 'package:kanso/note_list/models/data/note_list_status.dart';
import 'package:kanso/note_list/state/note_list/note_list_bloc.dart';
import 'package:kanso/search/models/data/search_status.dart';
import 'package:kanso/search/state/search/search_bloc.dart';
import 'package:kanso/settings/state/default_notebook/default_notebook_bloc.dart';
import 'package:kanso/shell/state/library_pane/library_pane_bloc.dart';
import 'package:kanso/shell/state/note_creation/note_creation_bloc.dart';

import 'close_bloc_later.dart';
import 'fixture_library_repository.dart';
import 'fixture_renderer_repository.dart';
import 'fixture_sync_repository.dart';
import 'library_fixtures.dart';
import 'seeded_appearance_bloc.dart';

/// Runs [body] with the clock at [mockNow], the time of the mock content.
Future<T> atMockNow<T>(Future<T> Function() body) =>
    withClock(Clock.fixed(mockNow), body);

/// A haptics repository that remembers what it was asked for.
class RecordingHaptics implements HapticsRepository {
  int selections = 0;
  int impacts = 0;

  @override
  Future<void> selection() async => selections++;

  @override
  Future<void> impact() async => impacts++;
}

/// The BLoCs the browsing screens read, over the mock library: the shared
/// ones, and a list of a place when a test makes one.
class BrowseBlocs {
  BrowseBlocs._(
    this.library,
    this.index,
    this.history,
    this.sync,
    this.syncRepository,
    this.listNavigation,
    this.libraryPane,
    this.creation,
    this.defaultNotebook,
  );

  /// The mock library (see [FixtureLibraryRepository]).
  final FixtureLibraryRepository library;
  final LibraryIndexBloc index;
  final NavigationHistoryBloc history;
  final SyncStatusBloc sync;
  final ListNavigationBloc listNavigation;
  final LibraryPaneBloc libraryPane;
  final NoteCreationBloc creation;
  final DefaultNotebookBloc defaultNotebook;
  final FixtureSyncRepository syncRepository;
  final FixtureRendererRepository renderer = FixtureRendererRepository();
  final RecordingHaptics haptics = RecordingHaptics();
  final MemoryNotePlaceRepository places = MemoryNotePlaceRepository();

  /// Makes them, with the library read, and closes them when the test ends.
  static Future<BrowseBlocs> start([
    FixtureLibraryRepository? library,
    FixtureSyncRepository? syncRepository,
  ]) async {
    final fixture = library ?? FixtureLibraryRepository();
    addTearDown(() => unawaited(fixture.close()));
    final index = LibraryIndexBloc(fixture)
      ..add(const LibraryIndexEvent.started());
    final history = NavigationHistoryBloc(fixture);
    closeBlocLater(index);
    closeBlocLater(history);
    final account = syncRepository ?? FixtureSyncRepository();
    final sync = SyncStatusBloc(account)..add(const SyncStatusEvent.started());
    closeBlocLater(sync);
    // A library held back (see `notebooksGate`) is still loading.
    if (fixture.notebooksGate == null) {
      await index.stream.firstWhere(
        (state) => state.status != LibraryIndexStatus.loading,
      );
    }
    if (sync.state.status == null) {
      await sync.stream.firstWhere((state) => state.status != null);
    }
    final listNavigation = ListNavigationBloc(fixture);
    final libraryPane = LibraryPaneBloc();
    closeBlocLater(listNavigation);
    closeBlocLater(libraryPane);
    final creation = NoteCreationBloc(
      fixture,
      MemoryNotePlaceRepository(),
      ForgetfulPreferencesRepository(),
    );
    closeBlocLater(creation);
    final defaultNotebook = DefaultNotebookBloc(
      ForgetfulPreferencesRepository(),
    );
    closeBlocLater(defaultNotebook);
    return BrowseBlocs._(
      fixture,
      index,
      history,
      sync,
      account,
      listNavigation,
      libraryPane,
      creation,
      defaultNotebook,
    );
  }

  /// A list of [place], read; or only made when [read] is false, so that it
  /// is still loading.
  Future<NoteListBloc> listOf(
    ListPlace place, {
    bool read = true,
  }) async {
    final bloc = NoteListBloc(
      library,
      ForgetfulPreferencesRepository(),
      haptics,
    );
    closeBlocLater(bloc);
    if (read) {
      bloc.add(NoteListEvent.started(place));
      await bloc.stream.firstWhere(
        (state) => state.status != NoteListStatus.loading,
      );
    }
    return bloc;
  }

  /// The note [id], read and highlighted in [themeId]; or only made when
  /// [read] is false, so that it is still loading.
  Future<NoteBloc> noteOf(
    String id, {
    String themeId = 'kanso-light',
    bool read = true,
  }) async {
    final bloc = NoteBloc(library, renderer, places);
    closeBlocLater(bloc);
    if (read) {
      bloc.add(NoteEvent.started(id, themeId));
      await bloc.stream.firstWhere(
        (state) => state.status != NoteScreenStatus.loading,
      );
    }
    return bloc;
  }

  /// A search, started with [query] in [scope]; and, unless [wait] is false,
  /// run.
  Future<SearchBloc> searchOf(
    String query, {
    SearchScopeKind scope = SearchScopeKind.all,
    String? notebookId,
    bool wait = true,
  }) async {
    final bloc = SearchBloc(library);
    closeBlocLater(bloc);
    bloc.add(SearchEvent.started(query, scope, notebookId));
    if (wait) {
      await bloc.stream.firstWhere(
        (state) =>
            state.status != SearchStatus.searching &&
            (!state.hasQuery || state.status != SearchStatus.idle),
      );
    }
    return bloc;
  }

  /// [child] with these BLoCs above it, and [list], [note] and [search] if
  /// given.
  Widget provide(
    Widget child, {
    NoteListBloc? list,
    NoteBloc? note,
    SearchBloc? search,
  }) {
    return MultiBlocProvider(
      providers: <BlocProvider<dynamic>>[
        BlocProvider<LibraryIndexBloc>.value(value: index),
        BlocProvider<NavigationHistoryBloc>.value(value: history),
        BlocProvider<SyncStatusBloc>.value(value: sync),
        BlocProvider<ListNavigationBloc>.value(value: listNavigation),
        BlocProvider<LibraryPaneBloc>.value(value: libraryPane),
        BlocProvider<NoteCreationBloc>.value(value: creation),
        BlocProvider<DefaultNotebookBloc>.value(value: defaultNotebook),
        if (list != null) BlocProvider<NoteListBloc>.value(value: list),
        if (note != null) BlocProvider<NoteBloc>.value(value: note),
        if (search != null) BlocProvider<SearchBloc>.value(value: search),
      ],
      child: child,
    );
  }
}
