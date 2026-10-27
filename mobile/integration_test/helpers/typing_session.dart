// Kanso — typing_session.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

// Shared by the typing test and the bench app: no `flutter_test` here.

import 'package:dartz/dartz.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/browse_location.dart';
import 'package:kanso/core/models/repos/abs/library_repository.dart';
import 'package:kanso/core/router/kanso_router.dart';
import 'package:kanso/editor/views/widgets/kanso_editable_text.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/note/models/data/note_mode.dart';
import 'package:kanso/note/state/note/note_bloc.dart';
import 'package:kanso/note/views/widgets/note_pane.dart';

import 'long_note.dart';
import 'typing_script.dart';

/// Makes a note of [words] words in a notebook of its own, in the library
/// the app is running on, and returns its id.
Future<String> createLongNote(int words) async {
  final library = getIt<LibraryRepository>();
  T made<T>(Either<ApplicationError, T> result) => result.fold(
    (error) => throw StateError('Making the long note failed: $error'),
    (value) => value,
  );
  final book = made(await library.createNotebook('Long notes'));
  final saved = made(
    await library.create(
      NewNote(
        notebookId: book.id,
        title: 'A note of $words words',
        body: longNote(words),
      ),
    ),
  );
  return saved.id;
}

/// Opens the note [id] in the app's router, turns it to Edit, and waits
/// until the field is on screen.
Future<void> openForTyping(String id) async {
  getIt<KansoRouter>().config.go(
    BrowseLocation.place(const ListPlace.all(), noteId: id).location,
  );
  final pane = await _waitFor<Element>(() {
    Element? found;
    void visit(Element element) {
      if (found != null) {
        return;
      }
      if (element.widget is NotePane) {
        found = element;
        return;
      }
      element.visitChildren(visit);
    }

    WidgetsBinding.instance.rootElement?.visitChildren(visit);
    return found;
  }, 'the note to open');
  // The note must have been read before it can be edited.
  final note = pane.read<NoteBloc>();
  if (note.state.note == null) {
    await note.stream
        .firstWhere((state) => state.note != null)
        .timeout(const Duration(minutes: 5));
  }
  note.add(const NoteEvent.modeChanged(NoteMode.edit));
  final field = await _waitFor<KansoEditableTextState>(
    findEditorState,
    'the editor to open',
  );
  await _waitFor<KansoEditableTextState>(
    () => field.widget.focusNode.hasFocus ? field : null,
    'the editor to have the focus',
  );
}

Future<T> _waitFor<T extends Object>(
  T? Function() look,
  String what, {
  Duration timeout = const Duration(minutes: 2),
}) async {
  final deadline = DateTime.now().add(timeout);
  while (DateTime.now().isBefore(deadline)) {
    final found = look();
    if (found != null) {
      return found;
    }
    await Future<void>.delayed(const Duration(milliseconds: 100));
  }
  throw StateError('Gave up waiting for $what.');
}
