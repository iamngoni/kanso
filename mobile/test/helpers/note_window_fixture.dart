// Kanso — note_window_fixture.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:kanso/note/state/note/note_bloc.dart';
import 'package:kanso/window/models/data/note_window_status.dart';
import 'package:kanso/window/state/note_window/note_window_bloc.dart';
import 'package:kanso/window/views/pages/note_window_page.dart';
import 'package:kanso/window/views/widgets/note_window_scope.dart';

import 'browse_blocs.dart';
import 'close_bloc_later.dart';
import 'fake_note_window_repository.dart';
import 'fixture_library_repository.dart';
import 'fixture_session_repository.dart';

/// A note's own window over the mock library: the BLoCs it reads, a window
/// repository that remembers what it was asked, and the page.
class NoteWindowFixture {
  NoteWindowFixture._(
    this.blocs,
    this.note,
    this.window,
    this.windows,
    this.session,
  );

  final BrowseBlocs blocs;
  final NoteBloc note;
  final NoteWindowBloc window;
  final FakeNoteWindowRepository windows;
  final FixtureSessionRepository session;

  FixtureLibraryRepository get library => blocs.library;

  /// Opens the note [id] in a window, read.
  static Future<NoteWindowFixture> open(
    String id, {
    Brightness brightness = Brightness.light,
    FixtureLibraryRepository? library,
    bool supported = true,
  }) async {
    final blocs = await BrowseBlocs.start(library);
    final note = await blocs.noteOf(id, themeId: 'kanso-${brightness.name}');
    final session = FixtureSessionRepository();
    final window = NoteWindowBloc(blocs.library, session);
    // Listen before the read starts so its state is never missed; the wait
    // is over microtasks alone, so a widget test's clock need not move.
    final read = window.stream.firstWhere(
      (state) => state.status != NoteWindowStatus.loading,
    );
    window.add(NoteWindowEvent.started(id));
    closeBlocLater(window);
    await read;
    return NoteWindowFixture._(
      blocs,
      note,
      window,
      FakeNoteWindowRepository(supported: supported),
      session,
    );
  }

  /// The window's page, with the BLoCs above it.
  Widget get page {
    return blocs.provide(
      NoteWindowScope(
        repository: windows,
        child: BlocProvider<NoteWindowBloc>.value(
          value: window,
          child: NoteWindowPage(windows: windows),
        ),
      ),
      note: note,
    );
  }
}
