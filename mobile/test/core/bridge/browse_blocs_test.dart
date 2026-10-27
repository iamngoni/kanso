// Kanso — browse_blocs_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/search_scope_kind.dart';
import 'package:kanso/core/models/repos/impl/rust_library_repository.dart';
import 'package:kanso/core/models/repos/impl/rust_renderer_repository.dart';
import 'package:kanso/note/models/data/note_screen_status.dart';
import 'package:kanso/note/models/repos/impl/memory_note_place_repository.dart';
import 'package:kanso/note/state/note/note_bloc.dart';
import 'package:kanso/note_list/models/data/note_list_status.dart';
import 'package:kanso/note_list/state/note_list/note_list_bloc.dart';
import 'package:kanso/search/models/data/search_status.dart';
import 'package:kanso/search/state/search/search_bloc.dart';

import '../../helpers/browse_blocs.dart';
import '../../helpers/core_library.dart';
import '../../helpers/dump.dart';
import '../../helpers/seeded_appearance_bloc.dart';
import 'suites/open_library.dart';

/// Only the text of the locked note has this.
const String _secret = 'zxqsecret';

/// Only the text of the open note has this.
const String _plain = 'zxqplain';

/// The screens' BLoCs over the real library: what the Rust core makes of a
/// note's text is what they show, and a locked note's text, closed, is in
/// nothing they hold.
void main() {
  setUpAll(initCoreForTests);

  test('a note is highlighted by the core, in a built-in theme', () async {
    final library = await openTestLibrary();
    final id = await makeNote(
      library,
      'Plan',
      '# Heading\n\nSome `code` here.\n',
    );
    final bloc = NoteBloc(
      RustLibraryRepository(library),
      RustRendererRepository(library.renderer()),
      MemoryNotePlaceRepository(),
    )..add(NoteEvent.started(id, 'nord'));
    addTearDown(bloc.close);
    final state = await bloc.stream.firstWhere(
      (state) => state.status == NoteScreenStatus.ready,
    );
    expect(state.lines, hasLength(4));
    // The heading line is one run in the theme's heading colour.
    expect(state.lines.first.runs, isNotEmpty);
    expect(state.lines.first.runs.first.range.start, 0);
    expect(state.lines.first.runs.last.range.end, 9);
    expect(
      state.lines[2].runs.any((run) => run.color != null),
      isTrue,
      reason: 'the code span is styled',
    );
    // Another theme gives other colours for the same text.
    bloc.add(const NoteEvent.syntaxThemeChanged('kanso-light'));
    final nord = state.lines.first.runs.last.color;
    final other = await bloc.stream.firstWhere(
      (state) =>
          state.themeId == 'kanso-light' &&
          state.lines.first.runs.last.color != nord,
    );
    expect(other.lines.first.runs.last.color, isNotNull);
  });

  test('a closed locked note is in no state of the screens', () async {
    final library = await openTestLibrary();
    final repository = RustLibraryRepository(library);
    final renderer = RustRendererRepository(library.renderer());
    final open = await makeNote(library, 'Open note', 'body $_plain open');
    final locked = await makeNote(
      library,
      'Locked note',
      '---\nowner: $_secret\n---\nbody of $_secret locked',
    );
    await library.locks().lockNote(id: locked, password: lockPassword);
    await library.locks().lock();

    // The note screen.
    final note = NoteBloc(repository, renderer, MemoryNotePlaceRepository())
      ..add(NoteEvent.started(locked, 'nord'));
    addTearDown(note.close);
    final noteState = await note.stream.firstWhere(
      (state) => state.status == NoteScreenStatus.locked,
    );
    expect(noteState.note?.title, 'Locked note');
    expect(dump(noteState.note), isNot(contains(_secret)));
    expect(noteState.lines, isEmpty);

    // The list.
    final haptics = RecordingHaptics();
    final list = NoteListBloc(
      repository,
      ForgetfulPreferencesRepository(),
      haptics,
    )..add(const NoteListEvent.started(ListPlace.all()));
    addTearDown(list.close);
    final listState = await list.stream.firstWhere(
      (state) => state.status == NoteListStatus.ready,
    );
    expect(listState.rows.map((row) => row.title), contains('Locked note'));
    expect(dump(listState.rows), isNot(contains(_secret)));
    final lockedRow = listState.rows.firstWhere((row) => row.id == locked);
    expect(lockedRow.locked, isTrue);
    expect(lockedRow.snippet, isEmpty);
    expect(lockedRow.tags, isEmpty);

    // A locked note can still be pinned from the list, and stays sealed.
    list.add(
      NoteListEvent.changeRequested(const NoteChange.togglePin(), <String>[
        locked,
      ]),
    );
    final pinned = await list.stream.firstWhere(
      (state) =>
          state.rows.any((row) => row.id == locked && row.pinned) &&
          state.status == NoteListStatus.ready,
    );
    expect(dump(pinned.rows), isNot(contains(_secret)));
    expect(haptics.impacts, 1);

    // Search never reads a locked note's text: the words only it has find
    // nothing, and the words only the open note has find it.
    final search = SearchBloc(repository)
      ..add(const SearchEvent.started(_secret, SearchScopeKind.all, null));
    addTearDown(search.close);
    final nothing = await search.stream.firstWhere(
      (state) => state.status == SearchStatus.ready,
    );
    expect(nothing.hits, isEmpty);
    search.add(const SearchEvent.queryChanged(_plain));
    final found = await search.stream.firstWhere(
      (state) => state.query == _plain && state.status == SearchStatus.ready,
    );
    expect(found.hits.map((hit) => hit.row.id), <String>[open]);

    // By its title it is found, with nothing else of it.
    search.add(const SearchEvent.queryChanged('Locked note'));
    final byTitle = await search.stream.firstWhere(
      (state) =>
          state.query == 'Locked note' &&
          state.hits.any((hit) => hit.row.id == locked),
    );
    final hit = byTitle.hits.singleWhere((hit) => hit.row.id == locked);
    expect(hit.row.locked, isTrue);
    expect(hit.row.snippet, isEmpty);
    expect(dump(byTitle.hits), isNot(contains(_secret)));
  });
}
