// Kanso — note_comments_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/views/widgets/kanso_text_input.dart';
import 'package:kanso/editor/views/controllers/kanso_markdown_controller.dart';
import 'package:kanso/editor/views/widgets/markdown_editor.dart';
import 'package:kanso/note/models/data/note_mode.dart';
import 'package:kanso/note/state/note/note_bloc.dart';
import 'package:kanso/note/views/widgets/note_pane.dart';
import 'package:kanso/sharing/state/comments/comments_bloc.dart';
import 'package:kanso/sharing/views/widgets/comments_sheet.dart';

import '../../../helpers/browse_blocs.dart';
import '../../../helpers/fake_editor_document.dart';
import '../../../helpers/fixture_shared_notes_repository.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/pump_routed.dart';
import '../../../helpers/register_editor_bloc.dart';
import '../../../helpers/sharing_fixtures.dart';

/// A shared note's page, with its comments, and a fake editing session.
class _Page {
  _Page(this.note, this.comments, this.document, this.shared);

  final NoteBloc note;
  final CommentsBloc comments;
  final FakeEditorDocument document;
  final FixtureSharedNotesRepository shared;

  KansoMarkdownController get controller =>
      (find.byType(MarkdownEditor).evaluate().single.widget as MarkdownEditor)
          .controller;
}

const String _text = 'Alpha beta gamma';

Future<_Page> _open(
  WidgetTester tester, {
  ShareRole role = ShareRole.owner,
  bool edit = true,
}) async {
  final shared = FixtureSharedNotesRepository();
  shared.shares[MockIds.syncDesign] = shareInfo(
    MockIds.syncDesign,
    myRole: role,
    owner: role == ShareRole.owner ? 'ngoni' : 'ana',
  );
  shared.commentLists[MockIds.syncDesign] = <CommentRow>[
    commentRow('c1', quote: 'beta', start: 6, end: 10),
    commentRow('c2', quote: 'gamma', start: 11, end: 16, resolved: true),
  ];
  final blocs = await BrowseBlocs.start(null, null, shared);
  final document = FakeEditorDocument(text: _text, title: 'Sync design notes')
    ..marks = const <Range16>[Range16(start: 6, end: 10)]
    ..commentWords = <String, Range16>{
      'c1': const Range16(start: 6, end: 10),
    };
  addTearDown(document.events_.close);
  registerEditorBloc(document);
  final note = await blocs.noteOf(MockIds.syncDesign);
  final comments = CommentsBloc(shared)
    ..add(const CommentsEvent.started(MockIds.syncDesign));
  addTearDown(comments.close);
  await atMockNow(
    () => pumpRouted(
      tester,
      blocs.provide(
        const NotePane(inPane: false),
        note: note,
        comments: comments,
      ),
    ),
  );
  await _settle(tester);
  if (edit) {
    note.add(const NoteEvent.modeChanged(NoteMode.edit));
    await tester.pump();
    await tester.pump();
    await _settle(tester);
  }
  return _Page(note, comments, document, shared);
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

void main() {
  group('A shared note and its comments', () {
    testWidgets('the button says how many are open, and opens them', (
      tester,
    ) async {
      await _open(tester, edit: false);
      final button = find.bySemanticsLabel('Comments, 1 open');
      expect(button, findsOneWidget);
      await tester.tap(button);
      await tester.pumpAndSettle();
      expect(find.byType(CommentsSheet), findsOneWidget);
      expect(find.text('beta'), findsOneWidget);
    });

    testWidgets('the editor tints the words the open comments are on', (
      tester,
    ) async {
      final page = await _open(tester);
      expect(page.document.calls, contains('commentMarks'));
      expect(page.controller.commentMarks, const <TextRange>[
        TextRange(start: 6, end: 10),
      ]);
    });

    testWidgets('a comment chosen in the preview opens the editor on its '
        'words', (tester) async {
      final page = await _open(tester, edit: false);
      await tester.tap(find.bySemanticsLabel('Comments, 1 open'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('beta'));
      await tester.pumpAndSettle();
      await _settle(tester);
      expect(page.note.state.mode, NoteMode.edit);
      expect(page.document.calls, contains('selectComment c1'));
      expect(
        page.controller.selection,
        const TextSelection(baseOffset: 6, extentOffset: 10),
      );
      expect(page.comments.state.selecting, isNull);
    });

    testWidgets('Comment in the text menu puts a comment on the selection', (
      tester,
    ) async {
      final page = await _open(tester);
      await tester.longPressAt(
        tester.getTopLeft(find.byType(MarkdownEditor)) + const Offset(20, 10),
      );
      await tester.pump();
      await tester.pump();
      await tester.tap(find.text('Comment'));
      await tester.pumpAndSettle();
      expect(find.text('Comment on the selection'), findsOneWidget);
      await tester.enterText(find.byType(KansoTextInput), 'Why alpha?');
      await tester.pump();
      await tester.tap(find.text('Comment').last);
      await tester.pumpAndSettle();
      await _settle(tester);
      expect(page.document.calls, contains('commentOnSelection Why alpha?'));
      expect(find.text('Comment on the selection'), findsNothing);
      // The list and the tint are asked for again.
      expect(
        page.shared.calls.where((call) => call.startsWith('comments ')),
        hasLength(greaterThanOrEqualTo(2)),
      );
    });

    testWidgets('a reader has no Comment in the text menu', (tester) async {
      await _open(tester, role: ShareRole.viewer);
      await tester.longPressAt(
        tester.getTopLeft(find.byType(MarkdownEditor)) + const Offset(20, 10),
      );
      await tester.pump();
      await tester.pump();
      expect(find.text('Select All'), findsOneWidget);
      expect(find.text('Comment'), findsNothing);
    });
  });
}
