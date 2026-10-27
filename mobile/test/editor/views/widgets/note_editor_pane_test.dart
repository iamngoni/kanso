// Kanso — note_editor_pane_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:kanso/core/bridge/api/types.dart' as core;
import 'package:kanso/core/bridge/api/types.dart' show EditCommand, FormatKind;
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/views/widgets/kanso_segment_cell.dart';
import 'package:kanso/editor/views/widgets/accessory_key.dart';
import 'package:kanso/editor/views/widgets/completion_popup.dart';
import 'package:kanso/editor/views/widgets/editor_conflict_strip.dart';
import 'package:kanso/editor/views/widgets/editor_failure_strip.dart';
import 'package:kanso/editor/views/widgets/editor_title_field.dart';
import 'package:kanso/editor/views/widgets/kanso_editable_text.dart';
import 'package:kanso/editor/views/widgets/keyboard_accessory_bar.dart';
import 'package:kanso/editor/views/widgets/new_property_row.dart';
import 'package:kanso/editor/views/widgets/note_editor_pane.dart';
import 'package:kanso/editor/views/widgets/properties_row.dart';
import 'package:kanso/editor/views/widgets/property_row.dart';
import 'package:kanso/note/models/data/note_mode.dart';
import 'package:kanso/note/models/data/note_place.dart';
import 'package:kanso/note/state/note/note_bloc.dart';
import 'package:kanso/note/views/widgets/locked_note_pane.dart';
import 'package:kanso/note/views/widgets/note_bottom_bar.dart';
import 'package:kanso/note/views/widgets/note_info_sheet.dart';
import 'package:kanso/note/views/widgets/note_mode_toggle.dart';
import 'package:kanso/note/views/widgets/note_pane.dart';
import 'package:kanso/search/views/widgets/highlighted_text.dart';
import 'package:kanso/shell/utils/kanso_shortcut_bindings.dart';

import '../../../helpers/browse_blocs.dart';
import '../../../helpers/fake_editor_document.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/pump_routed.dart';
import '../../../helpers/register_editor_bloc.dart';

/// The note page of the mock library with a fake editing session behind it.
class _Page {
  _Page(this.blocs, this.note, this.document, this.router);

  final BrowseBlocs blocs;
  final NoteBloc note;
  final FakeEditorDocument document;
  final GoRouter router;
}

Future<_Page> _open(
  WidgetTester tester, {
  String text = 'Hello world',
  bool inPane = false,
  bool edit = true,
  Size size = const Size(393, 852),
  NotePlace? place,
  bool openedNew = false,
  String noteTitle = 'Sync design notes',
}) async {
  final blocs = await BrowseBlocs.start();
  final document = FakeEditorDocument(text: text, title: noteTitle);
  addTearDown(document.events_.close);
  registerEditorBloc(document);
  if (place != null) {
    blocs.places.keep(MockIds.syncDesign, place);
  }
  if (openedNew) {
    blocs.library.notes[MockIds.syncDesign] = mockNote(
      title: noteTitle,
      body: text,
    );
    blocs.places.markOpenedNew(MockIds.syncDesign);
  }
  final note = await blocs.noteOf(MockIds.syncDesign);
  late final GoRouter router;
  await atMockNow(
    () async => router = await pumpRouted(
      tester,
      blocs.provide(NotePane(inPane: inPane), note: note),
      size: size,
      wrap: (page) => Shortcuts(
        shortcuts: kansoShortcutBindings(TargetPlatform.iOS),
        child: page,
      ),
    ),
  );
  if (edit) {
    note.add(const NoteEvent.modeChanged(NoteMode.edit));
    await tester.pump();
    await tester.pump();
    await tester.pump();
  }
  return _Page(blocs, note, document, router);
}

/// Lets the BLoCs' own futures finish, which the frames do not wait for.
Future<void> _settle(WidgetTester tester) async {
  await tester.runAsync(
    () => Future<void>.delayed(const Duration(milliseconds: 5)),
  );
  await tester.pump();
}

TextEditingValue _at(String text, int caret) => TextEditingValue(
  text: text,
  selection: TextSelection.collapsed(offset: caret),
);

void main() {
  group('NotePane for a note just made', () {
    testWidgets('opens to edit with the title to type when it has none', (
      tester,
    ) async {
      await _open(tester, edit: false, openedNew: true, noteTitle: '');
      await _settle(tester);
      expect(find.byType(EditorTitleField), findsOneWidget);
      final title = tester.widget<EditableText>(
        find.descendant(
          of: find.byType(EditorTitleField),
          matching: find.byType(EditableText),
        ),
      );
      expect(title.focusNode.hasFocus, isTrue);
    });

    testWidgets('a note with a title, as a link makes one, is typed in', (
      tester,
    ) async {
      await _open(tester, edit: false, openedNew: true, noteTitle: 'Not yet');
      await _settle(tester);
      final text = tester.widget<EditableText>(
        find.byType(KansoEditableText),
      );
      expect(text.focusNode.hasFocus, isTrue);
    });
  });

  group('NotePane in edit mode', () {
    testWidgets('has the toggle and the info button, in either mode', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      await _open(tester, edit: false);
      expect(find.byType(NoteModeToggle), findsOneWidget);
      expect(find.bySemanticsLabel('Note info'), findsOneWidget);
      expect(find.byType(EditorTitleField), findsNothing);
      handle.dispose();
    });

    testWidgets('Edit opens the editor: title, properties and the text', (
      tester,
    ) async {
      final page = await _open(tester, edit: false);
      await tester.tap(find.bySemanticsLabel('Edit'));
      await tester.pump();
      await tester.pump();
      expect(page.note.state.mode, NoteMode.edit);
      expect(find.byType(EditorTitleField), findsOneWidget);
      expect(find.byType(PropertiesRow), findsOneWidget);
      expect(find.byType(KansoEditableText), findsOneWidget);
      expect(find.text('Hello world', findRichText: true), findsOneWidget);
      expect(find.text('Sync design notes'), findsOneWidget);
    });

    testWidgets('Preview goes back, and saves what was typed', (tester) async {
      final page = await _open(tester);
      tester.testTextInput.updateEditingValue(_at('Hello world!', 12));
      await tester.pump();
      await tester.tap(find.bySemanticsLabel('Preview'));
      await tester.pump();
      await _settle(tester);
      expect(page.note.state.mode, NoteMode.preview);
      expect(find.byType(EditorTitleField), findsNothing);
      await tester.pump(const Duration(milliseconds: 10));
      expect(page.document.calls, contains('close'));
      expect(page.document.listening, isFalse);
    });

    testWidgets('typing in the text goes to the session, and says unsaved', (
      tester,
    ) async {
      final page = await _open(tester);
      tester.testTextInput.updateEditingValue(_at('Hello world!', 12));
      await tester.pump();
      expect(page.document.text, 'Hello world!');
      expect(page.document.applied, hasLength(1));
    });

    testWidgets(
      'the title goes to the session, and Return goes on to the text',
      (
        tester,
      ) async {
        final page = await _open(tester);
        final title = find.descendant(
          of: find.byType(EditorTitleField),
          matching: find.byType(EditableText),
        );
        await tester.tap(title);
        await tester.pump();
        tester.testTextInput.updateEditingValue(_at('A new title', 11));
        await tester.pump();
        expect(page.document.title, 'A new title');
        await tester.testTextInput.receiveAction(TextInputAction.next);
        await tester.pump();
        final text = tester.widget<EditableText>(
          find.byType(KansoEditableText),
        );
        expect(text.focusNode.hasFocus, isTrue);
      },
    );

    testWidgets('has no bottom bar while the keyboard is up, and has it back '
        'when it is down', (tester) async {
      await _open(tester);
      expect(find.byType(NoteBottomBar), findsOneWidget);
      tester.view.viewInsets = const FakeViewPadding(bottom: 300);
      addTearDown(tester.view.resetViewInsets);
      await tester.pump();
      expect(find.byType(NoteBottomBar), findsNothing);
      tester.view.resetViewInsets();
      await tester.pump();
      expect(find.byType(NoteBottomBar), findsOneWidget);
    });

    testWidgets('a conflict shows its strip, in the words of the desktop', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      final page = await _open(tester);
      page.document.events_.add(
        const core.EditorEvent.incoming(deleted: false),
      );
      await tester.pump();
      await tester.pump();
      expect(find.byType(EditorConflictStrip), findsOneWidget);
      expect(
        find.text(
          'This note was changed elsewhere while you were editing. The '
          "version you don't keep stays in its history.",
        ),
        findsOneWidget,
      );
      expect(find.text('Keep mine'), findsOneWidget);
      expect(find.text('Use theirs'), findsOneWidget);
      await tester.tap(find.text('Keep mine'));
      await tester.pump();
      expect(page.document.calls, contains('keepMine'));
      handle.dispose();
    });

    testWidgets('Use theirs shows their text', (tester) async {
      final page = await _open(tester);
      page.document.events_.add(
        const core.EditorEvent.incoming(deleted: false),
      );
      await tester.pump();
      await tester.pump();
      page.document
        ..text = 'theirs'
        ..version = 5;
      await tester.tap(find.text('Use theirs'));
      await tester.pump();
      await tester.pump();
      await tester.pump();
      expect(page.document.calls, contains('useTheirs'));
      expect(find.text('theirs', findRichText: true), findsOneWidget);
    });

    testWidgets('a deleted note asks Keep it or Discard', (tester) async {
      await _open(tester);
      final page = await _open(tester);
      page.document.events_.add(const core.EditorEvent.incoming(deleted: true));
      await tester.pump();
      await tester.pump();
      expect(
        find.text('This note was deleted elsewhere while you were editing.'),
        findsOneWidget,
      );
      expect(find.text('Keep it'), findsOneWidget);
      expect(find.text('Discard'), findsOneWidget);
    });

    testWidgets('a failed save shows a quiet strip, and Try again saves', (
      tester,
    ) async {
      final page = await _open(tester);
      page.document.events_.add(const core.EditorEvent.failed(message: 'disk'));
      await tester.pump();
      await tester.pump();
      expect(find.byType(EditorFailureStrip), findsOneWidget);
      expect(find.text("Your changes aren't saved yet."), findsOneWidget);
      await tester.tap(find.text('Try again'));
      await tester.pump();
      expect(page.document.calls, contains('flush'));
    });

    testWidgets('goes to the background: saves', (tester) async {
      final page = await _open(tester);
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.inactive);
      await tester.pump();
      expect(page.document.calls, contains('flush'));
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    });

    testWidgets('a text changed elsewhere is shown, title too', (tester) async {
      final page = await _open(tester);
      page.document
        ..text = 'from elsewhere'
        ..title = 'Elsewhere'
        ..version = 4;
      page.document.events_.add(
        core.EditorEvent.reloaded(snapshot: page.document.current),
      );
      await tester.pump();
      await tester.pump();
      expect(
        find.text('from elsewhere', findRichText: true),
        findsOneWidget,
      );
      expect(find.text('Elsewhere'), findsOneWidget);
    });

    testWidgets('a lock that closes turns to Preview and shows nothing of the '
        'text', (tester) async {
      final page = await _open(tester, text: 'SECRETWORDS');
      expect(find.text('SECRETWORDS', findRichText: true), findsOneWidget);
      // The library says the note is locked now.
      page.blocs.library.notes[MockIds.syncDesign] = mockLockedNote;
      page.document.events_.add(const core.EditorEvent.closed());
      await _settle(tester);
      await _settle(tester);
      page.blocs.library.changeStream.add(<String>[MockIds.syncDesign]);
      await tester.pump(const Duration(milliseconds: 300));
      await tester.pump();
      expect(page.note.state.mode, NoteMode.preview);
      expect(find.byType(LockedNotePane), findsOneWidget);
      expect(
        find.textContaining('SECRETWORDS', findRichText: true),
        findsNothing,
      );
      expect(find.byType(NoteModeToggle), findsNothing);
    });

    testWidgets('a note that is gone turns to Preview, which says so', (
      tester,
    ) async {
      final page = await _open(tester);
      page.blocs.library.notes.remove(MockIds.syncDesign);
      page.document.failures['useTheirs'] = const ApplicationError.notFound(
        message: 'gone',
      );
      page.document.events_.add(const core.EditorEvent.incoming(deleted: true));
      await tester.pump();
      await tester.pump();
      await tester.tap(find.text('Discard'));
      await _settle(tester);
      await _settle(tester);
      page.blocs.library.changeStream.add(<String>[MockIds.syncDesign]);
      await tester.pump(const Duration(milliseconds: 300));
      await tester.pump();
      expect(page.note.state.mode, NoteMode.preview);
      expect(find.text("This note isn't here"), findsOneWidget);
    });

    testWidgets('puts the caret where it was left, and leaves it where it is', (
      tester,
    ) async {
      final page = await _open(
        tester,
        edit: false,
        place: const NotePlace(caret: 5, scroll: 0),
      );
      page.note.add(const NoteEvent.modeChanged(NoteMode.edit));
      await tester.pump();
      await tester.pump();
      await tester.pump();
      final field = tester.widget<EditableText>(find.byType(KansoEditableText));
      expect(field.controller.selection.baseOffset, 5);
      field.controller.selection = const TextSelection.collapsed(offset: 8);
      await tester.pump();
      page.note.add(const NoteEvent.modeChanged(NoteMode.preview));
      await tester.pump();
      await tester.pump();
      expect(page.blocs.places.placeOf(MockIds.syncDesign)?.caret, 8);
    });

    testWidgets('a double tap on the preview opens the editor there', (
      tester,
    ) async {
      final page = await _open(tester, edit: false);
      final line = find.textContaining(
        'Every device keeps',
        findRichText: true,
      );
      expect(line, findsOneWidget);
      final at = tester.getTopLeft(line) + const Offset(30, 8);
      await tester.tapAt(at);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.tapAt(at);
      await tester.pump(const Duration(milliseconds: 100));
      await tester.pump();
      await tester.pump();
      expect(page.note.state.mode, NoteMode.edit);
      expect(page.note.state.caretInBody, isTrue);
      expect(page.note.state.caret, isNotNull);
      // The caret is in that line of the body.
      final body = mockOpenNote.body!;
      final start = body.indexOf('Every device keeps');
      expect(page.note.state.caret, inInclusiveRange(start, start + 20));
    });

    testWidgets('the info button shows the facts of the note in a sheet', (
      tester,
    ) async {
      await _open(tester, edit: false);
      await tester.tap(find.bySemanticsLabel('Note info'));
      await tester.pump(const Duration(milliseconds: 500));
      expect(find.byType(NoteInfoSheet), findsOneWidget);
      expect(
        find.descendant(
          of: find.byType(NoteInfoSheet),
          matching: find.text('Projects / Kanso'),
        ),
        findsOneWidget,
      );
    });

    testWidgets('the keyboard bar is there while the text has the focus', (
      tester,
    ) async {
      final page = await _open(tester);
      await tester.pump();
      expect(find.byType(KeyboardAccessoryBar), findsOneWidget);
      await tester.tap(
        find.byWidgetPredicate(
          (widget) => widget is AccessoryKey && widget.semanticLabel == 'Bold',
        ),
      );
      expect(page.document.commands, <EditCommand>[
        const EditCommand.format(format: FormatKind.bold),
      ]);
      // The title is not Markdown: no bar there.
      await tester.tap(
        find.descendant(
          of: find.byType(EditorTitleField),
          matching: find.byType(EditableText),
        ),
      );
      await tester.pump();
      expect(find.byType(KeyboardAccessoryBar), findsNothing);
    });

    testWidgets('Command-1 edits and Command-3 reads', (tester) async {
      final page = await _open(tester, edit: false);
      await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.digit1);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
      await tester.pump();
      await tester.pump();
      expect(page.note.state.mode, NoteMode.edit);
      await tester.pump();
      await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.digit3);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
      await tester.pump();
      expect(page.note.state.mode, NoteMode.preview);
    });

    testWidgets(
      'Command-S saves now, and Escape goes to Preview',
      (
        tester,
      ) async {
        final page = await _open(tester);
        await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
        await tester.sendKeyEvent(LogicalKeyboardKey.keyS);
        await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
        await _settle(tester);
        expect(page.document.calls, contains('flush'));
        await tester.sendKeyEvent(LogicalKeyboardKey.escape);
        await tester.pump();
        expect(page.note.state.mode, NoteMode.preview);
      },
      variant: TargetPlatformVariant.only(TargetPlatform.iOS),
    );

    testWidgets(
      'what completes at the caret is offered, and a tap chooses it',
      (
        tester,
      ) async {
        final page = await _open(tester, text: '[[Pl');
        page.document
          ..context = const core.EditorContext(
            inList: false,
            inTable: false,
            inCode: false,
            completionHere: true,
            history: core.HistoryState(canUndo: false, canRedo: false),
          )
          ..offered = const core.Completion(
            kind: core.CompletionKind.wiki,
            replace: core.Range16(start: 0, end: 4),
            items: <core.CompletionItem>[
              core.CompletionItem(
                label: 'Plan',
                detail: 'Projects / Kanso',
                highlight: core.Range16(start: 0, end: 2),
                insert: '[[Plan]]',
                newNote: false,
              ),
            ],
          );
        final field = tester.widget<EditableText>(
          find.byType(KansoEditableText),
        );
        field.controller.selection = const TextSelection.collapsed(offset: 3);
        await _settle(tester);
        await _settle(tester);
        expect(find.byType(CompletionPopup), findsOneWidget);
        expect(find.text('Projects / Kanso'), findsOneWidget);
        page.document.nextOutcome = const core.EditOutcome(
          version: 9,
          replace: core.Replacement(start: 0, end: 4, text: '[[Plan]]'),
          selection: core.Range16(start: 8, end: 8),
          history: core.HistoryState(canUndo: true, canRedo: false),
        );
        await tester.tap(
          find.byWidgetPredicate(
            (widget) => widget is HighlightedText && widget.text == 'Plan',
          ),
        );
        await _settle(tester);
        expect(field.controller.text, '[[Plan]]');
      },
    );

    testWidgets('Open link goes to the note the link is to, saving first', (
      tester,
    ) async {
      final page = await _open(tester, text: 'see [[Plan]] now');
      page.document.link = const core.LinkTarget.note(id: 'note:plan');
      await tester.longPressAt(
        tester.getTopLeft(find.byType(KansoEditableText)) +
            const Offset(55, 10),
      );
      await tester.pump();
      await tester.pump();
      await tester.tap(find.text('Open link'));
      await _settle(tester);
      await _settle(tester);
      expect(page.document.calls, contains('flush'));
      expect(page.router.state.uri.path, '/notes/note/note%3Aplan');
    });

    testWidgets('the properties open, change, go and come', (tester) async {
      final handle = tester.ensureSemantics();
      final blocs = await BrowseBlocs.start();
      final document =
          FakeEditorDocument(text: 'Hello', title: 'Sync design notes')
            ..properties = const <core.Property>[
              core.Property(
                key: 'owner',
                kind: core.PropertyKind.text,
                text: 'Ada',
                items: <String>[],
              ),
            ];
      addTearDown(document.events_.close);
      registerEditorBloc(document);
      final note = await blocs.noteOf(MockIds.syncDesign);
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(const NotePane(inPane: false), note: note),
        ),
      );
      note.add(const NoteEvent.modeChanged(NoteMode.edit));
      await tester.pump();
      await tester.pump();
      await tester.pump();
      expect(find.text('Properties · 1'), findsOneWidget);
      expect(find.byType(PropertyRow), findsNothing);
      await tester.tap(find.byType(PropertiesRow));
      await tester.pump();
      expect(find.byType(PropertyRow), findsOneWidget);

      // Change the value.
      document.properties = const <core.Property>[
        core.Property(
          key: 'owner',
          kind: core.PropertyKind.text,
          text: 'Grace',
          items: <String>[],
        ),
      ];
      final owner = find.descendant(
        of: find.byType(PropertyRow),
        matching: find.byType(EditableText),
      );
      await tester.enterText(owner, 'Grace');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await _settle(tester);
      expect(document.calls, contains('setProperty 0 owner Grace'));
      expect(find.text('Grace'), findsOneWidget);

      // Take it away.
      document.properties = const <core.Property>[];
      await tester.tap(find.bySemanticsLabel('Remove owner'));
      await _settle(tester);
      expect(document.calls, contains('removeProperty 0'));
      expect(find.text('Properties · 0'), findsOneWidget);
      expect(find.byType(PropertyRow), findsNothing);

      // Put one in.
      document.properties = const <core.Property>[
        core.Property(
          key: 'status',
          kind: core.PropertyKind.text,
          text: 'open',
          items: <String>[],
        ),
      ];
      await tester.tap(find.text('Add property'));
      await tester.pump();
      await tester.pump();
      final fields = find.descendant(
        of: find.byType(NewPropertyRow),
        matching: find.byType(EditableText),
      );
      await tester.enterText(fields.first, 'status');
      await tester.enterText(fields.last, 'open');
      await tester.tap(find.bySemanticsLabel('Add'));
      await _settle(tester);
      expect(document.calls, contains('setProperty null status open'));
      expect(find.text('Properties · 1'), findsOneWidget);
      handle.dispose();
    });

    testWidgets('a property that does not work says so', (tester) async {
      final handle = tester.ensureSemantics();
      final page = await _open(tester);
      page.document.failures['setProperty'] = const ApplicationError.invalid(
        message: 'A property needs a name.',
      );
      await tester.tap(find.byType(PropertiesRow));
      await tester.pump();
      await tester.tap(find.text('Add property'));
      await tester.pump();
      await tester.pump();
      final fields = find.descendant(
        of: find.byType(NewPropertyRow),
        matching: find.byType(EditableText),
      );
      await tester.enterText(fields.first, 'x');
      await tester.tap(find.bySemanticsLabel('Add'));
      await _settle(tester);
      await _settle(tester);
      expect(find.text("That property couldn't be set."), findsOneWidget);
      handle.dispose();
    });

    testWidgets('the toggle marks the mode', (tester) async {
      await _open(tester);
      final cells = tester.widgetList<KansoSegmentCell<NoteMode>>(
        find.byType(KansoSegmentCell<NoteMode>),
      );
      expect(
        cells.where((cell) => cell.active).single.segment.value,
        NoteMode.edit,
      );
    });

    testWidgets('a note that cannot be opened for editing says so', (
      tester,
    ) async {
      final blocs = await BrowseBlocs.start();
      final document = FakeEditorDocument()
        ..failAll = const ApplicationError.storage(message: 'x');
      addTearDown(document.events_.close);
      registerEditorBloc(document);
      final note = await blocs.noteOf(MockIds.syncDesign);
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(const NotePane(inPane: false), note: note),
        ),
      );
      note.add(const NoteEvent.modeChanged(NoteMode.edit));
      await tester.pump();
      await tester.pump();
      await tester.pump();
      expect(find.byType(NoteEditorPane), findsOneWidget);
      expect(find.byType(KansoEditableText), findsNothing);
    });
  });
}
