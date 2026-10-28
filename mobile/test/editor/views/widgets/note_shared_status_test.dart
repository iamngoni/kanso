// Kanso — note_shared_status_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/editor/views/controllers/kanso_markdown_controller.dart';
import 'package:kanso/editor/views/widgets/editor_title_field.dart';
import 'package:kanso/editor/views/widgets/kanso_editable_text.dart';
import 'package:kanso/editor/views/widgets/keyboard_accessory_bar.dart';
import 'package:kanso/editor/views/widgets/markdown_editor.dart';
import 'package:kanso/editor/views/widgets/remote_carets.dart';
import 'package:kanso/note/models/data/note_mode.dart';
import 'package:kanso/note/state/note/note_bloc.dart';
import 'package:kanso/note/views/widgets/note_pane.dart';
import 'package:kanso/sharing/state/shared_note_status/shared_note_status_bloc.dart';

import '../../../helpers/browse_blocs.dart';
import '../../../helpers/fake_editor_document.dart';
import '../../../helpers/fixture_shared_notes_repository.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/pump_routed.dart';
import '../../../helpers/register_editor_bloc.dart';

/// A shared note's page, its status, and a fake editing session.
class _Page {
  _Page(this.note, this.document, this.shared);

  final NoteBloc note;
  final FakeEditorDocument document;
  final FixtureSharedNotesRepository shared;

  KansoMarkdownController get controller =>
      (find.byType(MarkdownEditor).evaluate().single.widget as MarkdownEditor)
          .controller;
}

LiveStatus _status(
  ShareRole role, {
  bool ended = false,
  List<String> problems = const <String>[],
}) => LiveStatus(
  role: role,
  connected: true,
  ended: ended,
  nearLimit: false,
  problems: problems,
);

const CursorMark _ana = CursorMark(
  name: 'ana',
  caret: 5,
  start: 0,
  end: 5,
  light: 0x2F6AAE,
  dark: 0x7FB0EC,
);

Future<_Page> _open(
  WidgetTester tester,
  LiveStatus? status, {
  List<CursorMark> cursors = const <CursorMark>[],
}) async {
  final shared = FixtureSharedNotesRepository();
  if (status != null) {
    shared.liveStatuses[MockIds.syncDesign] = status;
  }
  final blocs = await BrowseBlocs.start(null, null, shared);
  final document = FakeEditorDocument(
    text: 'Alpha beta gamma',
    title: 'Sync design notes',
  )..cursors = cursors;
  addTearDown(document.events_.close);
  registerEditorBloc(document);
  final note = await blocs.noteOf(MockIds.syncDesign);
  final bloc = SharedNoteStatusBloc(shared)
    ..add(const SharedNoteStatusEvent.started(MockIds.syncDesign));
  addTearDown(bloc.close);
  await atMockNow(
    () => pumpRouted(
      tester,
      blocs.provide(const NotePane(inPane: false), note: note, status: bloc),
    ),
  );
  await _settle(tester);
  note.add(const NoteEvent.modeChanged(NoteMode.edit));
  await tester.pump();
  await tester.pump();
  await _settle(tester);
  return _Page(note, document, shared);
}

/// Lets the BLoCs' own futures finish, which the frames do not wait for.
Future<void> _settle(WidgetTester tester) async {
  for (var i = 0; i < 3; i++) {
    await tester.runAsync(
      () => Future<void>.delayed(const Duration(milliseconds: 5)),
    );
    await tester.pump();
  }
}

bool _textReadOnly(WidgetTester tester) =>
    tester.widget<KansoEditableText>(find.byType(KansoEditableText)).readOnly;

bool _titleReadOnly(WidgetTester tester) => tester
    .widget<EditableText>(
      find.descendant(
        of: find.byType(EditorTitleField),
        matching: find.byType(EditableText),
      ),
    )
    .readOnly;

void main() {
  group('A shared note’s page', () {
    testWidgets('to view: says so, and its title and text are for reading', (
      tester,
    ) async {
      await _open(tester, _status(ShareRole.viewer));
      expect(
        find.text('Shared · Can view · Read only · Live'),
        findsOneWidget,
      );
      expect(_textReadOnly(tester), isTrue);
      expect(_titleReadOnly(tester), isTrue);
      // Nothing to type: no bar of keys over the keyboard.
      await tester.tap(find.byType(MarkdownEditor));
      await tester.pump();
      expect(find.byType(KeyboardAccessoryBar), findsNothing);
    });

    testWidgets('to comment on: read only as well', (tester) async {
      await _open(tester, _status(ShareRole.commenter));
      expect(find.textContaining('Can comment · Read only'), findsOneWidget);
      expect(_textReadOnly(tester), isTrue);
    });

    testWidgets('to edit: no read only, and the text takes typing', (
      tester,
    ) async {
      await _open(tester, _status(ShareRole.editor));
      expect(find.text('Shared · Can edit · Live'), findsOneWidget);
      expect(_textReadOnly(tester), isFalse);
      expect(_titleReadOnly(tester), isFalse);
    });

    testWidgets('sharing that ended leaves a private copy to edit', (
      tester,
    ) async {
      await _open(tester, _status(ShareRole.viewer, ended: true));
      expect(find.text('Private copy · No longer shared'), findsOneWidget);
      expect(_textReadOnly(tester), isFalse);
    });

    testWidgets('a note that isn’t shared says nothing of sharing', (
      tester,
    ) async {
      await _open(tester, null);
      expect(find.textContaining('Shared ·'), findsNothing);
      expect(find.byType(RemoteCarets), findsOneWidget);
      expect(_textReadOnly(tester), isFalse);
    });

    testWidgets('what couldn’t be done is shown, and Dismiss clears it', (
      tester,
    ) async {
      final page = await _open(
        tester,
        _status(
          ShareRole.editor,
          problems: <String>['This note is past its size limit.', 'Older.'],
        ),
      );
      expect(
        find.text('This note is past its size limit. (1 more)'),
        findsOneWidget,
      );
      await tester.tap(find.text('Dismiss'));
      await _settle(tester);
      expect(
        page.shared.calls,
        contains('dismissProblems ${MockIds.syncDesign}'),
      );
      expect(find.textContaining('size limit'), findsNothing);
    });

    testWidgets('others’ cursors are asked for and drawn in the editor', (
      tester,
    ) async {
      final page = await _open(
        tester,
        _status(ShareRole.editor),
        cursors: const <CursorMark>[_ana],
      );
      expect(page.document.calls, contains('remoteCursors'));
      expect(page.controller.remoteCursors, const <CursorMark>[_ana]);
      // The room says someone moved: they are asked for again.
      final before = page.document.calls
          .where((call) => call == 'remoteCursors')
          .length;
      page.shared.eventsController.add(
        const SharedEvent(
          noteIds: <String>[],
          invitations: 0,
          commentNotes: <String>[],
          liveNotes: <String>[MockIds.syncDesign],
        ),
      );
      await _settle(tester);
      expect(
        page.document.calls.where((call) => call == 'remoteCursors').length,
        greaterThan(before),
      );
    });
  });
}
