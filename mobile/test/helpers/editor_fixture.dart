// Kanso — editor_fixture.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/library.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/core/models/data/layout_class.dart';
import 'package:kanso/editor/models/repos/impl/rust_editor_repository.dart';
import 'package:kanso/editor/views/widgets/kanso_editable_text.dart';
import 'package:kanso/note/models/data/note_mode.dart';
import 'package:kanso/note/state/note/note_bloc.dart';
import 'package:kanso/note/views/widgets/note_pane.dart';
import 'package:kanso/sharing/state/shared_note_status/shared_note_status_bloc.dart';
import 'package:kanso/shell/views/widgets/list_note_layout.dart';
import 'package:kanso/shell/views/widgets/tablet_layout.dart';

import 'browse_blocs.dart';
import 'core_library.dart';
import 'kanso_viewport.dart';
import 'library_fixtures.dart';
import 'register_editor_bloc.dart';

/// The text of the note the editor goldens show, after its properties.
const String goldenText =
    '## Open questions\n'
    '- How long a device may stay offline\n'
    '- What a conflict looks like to a person\n'
    '\n'
    'See also [[Sync wishlist]]';

/// The properties of that note, as front matter.
const String goldenFront = '---\nowner: Ada\nstatus: active\nrank: 2\n---\n';

/// [note] as the note pane of a window of [viewport]: beside empty panes for
/// the list, and for the library when there is room for it.
Widget inPanes(KansoViewport viewport, Brightness brightness, Widget note) {
  final palette = brightness == Brightness.dark
      ? KansoPalette.kansoDark
      : KansoPalette.kansoLight;
  final list = ColoredBox(color: palette.colors.list);
  return switch (viewport.layout) {
    LayoutClass.single => note,
    LayoutClass.double => ListNoteLayout(list: list, note: note),
    LayoutClass.triple => TabletLayout(
      library: ColoredBox(color: palette.colors.sidebar),
      list: list,
      note: note,
    ),
  };
}

/// The note page in edit mode over a real library: the Rust core's colours,
/// completions and sessions behind the screens, and the fixture library
/// behind the rest, so a golden shows what a phone would.
class EditorFixture {
  EditorFixture._(this.library, this.noteId, this.blocs, this.note);

  final KansoLibrary library;
  final String noteId;
  final BrowseBlocs blocs;
  final NoteBloc note;

  /// What the page says of the note's sharing, for a note pictured as
  /// shared.
  SharedNoteStatusBloc? status;

  /// Makes a library with some notebooks and notes to link to, the note to
  /// edit with [body], and opens it for editing.
  static Future<EditorFixture> open(
    WidgetTester tester, {
    required Brightness brightness,
    String body = '$goldenFront$goldenText',
    String title = 'Sync design notes',
  }) async {
    final made = (await tester.runAsync(() async {
      await initCoreForTests();
      final library = await openTestLibrary();
      final projects = await library.createNotebook(name: 'Projects');
      final kanso = await library.createNotebook(
        name: 'Kanso',
        parentId: projects.id,
      );
      final reading = await library.createNotebook(name: 'Reading');
      final inbox = await library.createNotebook(name: 'Inbox');
      // Apart in time, so that what ties for a completion is always in the
      // same order.
      Future<void> note(
        String name,
        String notebook, [
        String text = '',
      ]) async {
        await library.create(
          note: NewNote(notebookId: notebook, title: name, body: text),
        );
        await Future<void>.delayed(const Duration(milliseconds: 5));
      }

      await note('Sync overview', kanso.id);
      await note('Syncing across devices', reading.id);
      await note('Sync wishlist', inbox.id, 'A list. #todo #tools');
      final saved = await library.create(
        note: NewNote(notebookId: kanso.id, title: title, body: body),
      );
      return (library, saved.id);
    }))!;
    final (library, noteId) = made;
    registerEditorBlocOn(RustEditorRepository(library));
    final blocs = await BrowseBlocs.start();
    blocs.library.notes[noteId] = mockNote(
      id: noteId,
      title: title,
      body: body,
      tags: <String>[MockIds.tagIdea, MockIds.tagDraft],
      status: 'active',
    );
    final note = await blocs.noteOf(
      noteId,
      themeId: 'kanso-${brightness.name}',
    );
    note.add(const NoteEvent.modeChanged(NoteMode.edit));
    return EditorFixture._(library, noteId, blocs, note);
  }

  /// The note page on a phone.
  Widget get phone => blocs.provide(
    const Scaffold(
      resizeToAvoidBottomInset: false,
      body: NotePane(inPane: false),
    ),
    note: note,
    status: status,
  );

  /// The page as a window of [viewport] has it: the phone's page, or the
  /// note pane beside the panes the window has room for, empty.
  Widget inViewport(KansoViewport viewport, Brightness brightness) {
    if (viewport.layout == LayoutClass.single) {
      return phone;
    }
    return blocs.provide(
      Scaffold(
        resizeToAvoidBottomInset: false,
        body: inPanes(viewport, brightness, const NotePane(inPane: true)),
      ),
      note: note,
      status: status,
    );
  }

  /// Lets what the core answers (the session, colours, completions) arrive
  /// and be drawn.
  Future<void> settle(WidgetTester tester) async {
    for (var i = 0; i < 5; i++) {
      await tester.runAsync(
        () => Future<void>.delayed(const Duration(milliseconds: 40)),
      );
      await tester.pump(const Duration(milliseconds: 20));
    }
  }

  /// The field of the text.
  EditableText field(WidgetTester tester) =>
      tester.widget<EditableText>(find.byType(KansoEditableText));
}
