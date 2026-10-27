// Kanso — editor_suite.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';

import 'open_library.dart';

/// The editing session, through the bridge: every call of `KansoEditor`, and
/// `KansoLibrary.edit` that opens it.
void editorSuite(OpenLibrary open) {
  group('KansoEditor', () {
    test('a note opens with its text, and typing is kept in Rust', () async {
      final library = await open();
      final id = await makeNote(library, 'Plan', 'hello');
      final editor = await library.edit(id: id);

      final first = await editor.snapshot();
      expect(first.id, id);
      expect(first.title, 'Plan');
      expect(first.text, 'hello');
      expect(first.lock, isA<LockState_Plain>());
      expect(first.save, isA<SaveState_Saved>());
      expect(first.history.canUndo, isFalse);

      // Typing: the field tells Rust what it did, and hears nothing back.
      final context = editor.select(selection: _caret(5), composing: false);
      expect(context.inList, isFalse);
      expect(context.completionHere, isFalse);
      final typed = editor.apply(
        change: _type(first.version, 5, 5, ' 🙂 world'),
      );
      expect(typed.replace, isNull);
      expect(typed.version, greaterThan(first.version));
      expect(typed.history.canUndo, isTrue);
      expect((await editor.snapshot()).text, 'hello 🙂 world');

      // A change made against an old text is refused, and so is one outside
      // the text: neither changes it.
      expect(
        () => editor.apply(change: _type(first.version, 0, 0, 'x')),
        throwsA(isA<KansoError_Invalid>()),
      );
      expect(
        () => editor.apply(change: _type(typed.version, 3, 99, 'x')),
        throwsA(isA<KansoError_Invalid>()),
      );
      expect((await editor.snapshot()).text, 'hello 🙂 world');

      editor.setTitle(title: 'The plan');
      expect((await editor.snapshot()).title, 'The plan');

      // Saving writes the title and the text.
      final saved = await editor.flush();
      expect(saved, isA<SaveState_Saved>());
      final stored = await library.note(id: id);
      expect(stored.title, 'The plan');
      expect(stored.body, 'hello 🙂 world');

      final closed = await editor.close();
      expect(closed, isA<SaveState_Saved>());
      await expectLater(editor.snapshot(), throwsA(isA<KansoError_Invalid>()));
      await expectLater(
        library.edit(id: 'note:nope'),
        throwsA(isA<KansoError_NotFound>()),
      );
    });

    test('commands, undo and redo are decided in Rust', () async {
      final library = await open();
      final id = await makeNote(library, 'List', '- one');
      final editor = await library.edit(id: id);
      var version = (await editor.snapshot()).version;

      editor.select(selection: _caret(5), composing: false);
      // Enter on a list item starts the next one; Shift-Enter does not.
      expect(
        editor.command(command: const EditCommand.enter(plain: true)),
        isNull,
      );
      final enter = editor.command(
        command: const EditCommand.enter(plain: false),
      )!;
      expect(enter.replace, isNotNull);
      expect(enter.selection, _caret(8));
      version = enter.version;
      expect((await editor.snapshot()).text, '- one\n- ');
      expect(
        editor.select(selection: _caret(8), composing: false).inList,
        isTrue,
      );

      // A format, the heading cycle, wiki brackets and a completion chosen.
      final bold = editor.command(
        command: const EditCommand.format(format: FormatKind.bold),
      )!;
      expect(bold.replace, isNotNull);
      expect((await editor.snapshot()).text, '- one\n- ****');
      final heading = editor.command(command: const EditCommand.cycleHeading());
      expect(heading, isNotNull);
      expect((await editor.snapshot()).text, startsWith('- one\n'));
      final tab = editor.command(command: const EditCommand.tab(out: false));
      expect(tab, isNotNull);

      // Undo takes the steps back one at a time, and redo makes them again.
      final steps = <String>[];
      for (var undone = editor.undo(); undone != null; undone = editor.undo()) {
        steps.add((await editor.snapshot()).text);
      }
      expect(steps.last, '- one');
      expect((await editor.snapshot()).history.canUndo, isFalse);
      expect((await editor.snapshot()).history.canRedo, isTrue);
      var redone = 0;
      for (var again = editor.redo(); again != null; again = editor.redo()) {
        redone++;
      }
      expect(redone, steps.length);
      expect(editor.redo(), isNull);

      // A completion chosen is one step of its own.
      version = (await editor.snapshot()).version;
      final end = (await editor.snapshot()).text.length;
      final inserted = editor.command(
        command: EditCommand.insert(
          range: Range16(start: end, end: end),
          text: ' [[Plan]]',
        ),
      )!;
      expect(inserted.version, greaterThan(version));
      expect(editor.command(command: const EditCommand.wikiLink()), isNotNull);
    });

    test('colours, completions and links are asked for in Rust', () async {
      final library = await open();
      final target = await makeNote(library, 'Budget plan', 'SECRETWORDS');
      final id = await makeNote(library, 'Doc', '# Title\n\nsee ');
      final editor = await library.edit(id: id);
      final snapshot = await editor.snapshot();

      final runs = await editor.styledRuns(
        start: 0,
        end: snapshot.text.length,
        themeId: 'kanso-light',
      );
      expect(runs.version, snapshot.version);
      expect(runs.runs, isNotEmpty);
      expect(runs.runs.first.range, const Range16(start: 0, end: 1));
      expect(runs.runs.first.color, isNotNull);
      await expectLater(
        editor.styledRuns(start: 0, end: 5, themeId: 'no-such-theme'),
        throwsA(isA<KansoError_NotFound>()),
      );

      // Nothing is completing yet.
      expect(await editor.completion(), isNull);
      final end = snapshot.text.length;
      final typed = editor.apply(
        change: _type(snapshot.version, end, end, '[[Bud'),
      );
      editor.select(selection: _caret(end + 5), composing: false);
      final completion = (await editor.completion())!;
      expect(completion.kind, CompletionKind.wiki);
      expect(completion.replace, Range16(start: end, end: end + 5));
      expect(completion.items.first.label, 'Budget plan');
      expect(completion.items.first.highlight, const Range16(start: 0, end: 3));
      expect(completion.items.last.newNote, isTrue);
      // Titles only: not a word of the note.
      expect(
        completion.items.map((item) => item.insert).join(),
        isNot(contains('SECRETWORDS')),
      );
      final chosen = editor.command(
        command: EditCommand.insert(
          range: completion.replace,
          text: completion.items.first.insert,
        ),
      )!;
      expect(chosen.version, greaterThan(typed.version));

      final text = (await editor.snapshot()).text;
      expect(
        await editor.linkAt(offset: text.length - 2),
        LinkTarget.note(id: target),
      );
      expect(await editor.linkAt(offset: 0), isNull);
    });

    test('a photo is saved as an attachment and put in the text', () async {
      final library = await open();
      final id = await makeNote(library, 'Attach', 'hello');
      final editor = await library.edit(id: id);
      final snapshot = await editor.snapshot();
      editor.select(selection: _caret(5), composing: false);

      final attached = await editor.attach(
        file: ShareFile(
          name: 'photo.png',
          contentType: 'image/png',
          bytes: Uint8List.fromList(_pngSignature),
        ),
      );
      expect(attached.replace, isNotNull);
      expect(attached.version, greaterThan(snapshot.version));
      final text = (await editor.snapshot()).text;
      expect(text, contains('![](inkdrop://file:'));

      // Only an image or a PDF can be attached: the rest is refused, and
      // the text is as it was.
      await expectLater(
        editor.attach(
          file: ShareFile(
            name: 'notes.txt',
            bytes: Uint8List.fromList(<int>[1, 2, 3]),
          ),
        ),
        throwsA(isA<KansoError_Invalid>()),
      );
      expect((await editor.snapshot()).text, text);
    });

    test('properties are kept apart from the text', () async {
      final library = await open();
      final id = await makeNote(
        library,
        'Task',
        '---\nowner: Ada\nrank: 2\n---\nthe text',
      );
      final editor = await library.edit(id: id);
      final snapshot = await editor.snapshot();
      expect(snapshot.text, 'the text');
      expect(snapshot.properties.map((p) => p.key), <String>['owner', 'rank']);

      var properties = await editor.setProperty(
        index: 1,
        key: '',
        value: '3',
      );
      expect(properties[1].kind, PropertyKind.number);
      expect(properties[1].text, '3');
      properties = await editor.setProperty(key: 'status', value: 'open');
      expect(properties.map((p) => p.key), <String>['owner', 'rank', 'status']);
      properties = await editor.removeProperty(index: 0);
      expect(properties.map((p) => p.key), <String>['rank', 'status']);
      await expectLater(
        editor.removeProperty(index: 9),
        throwsA(isA<KansoError_NotFound>()),
      );
      expect((await editor.snapshot()).text, 'the text');

      await editor.flush();
      final stored = (await library.note(id: id)).body!;
      expect(stored, startsWith('---\n'));
      expect(stored, endsWith('---\nthe text'));
      expect(stored, isNot(contains('owner')));
      expect(
        (await library.properties(id: id)).map((p) => p.key),
        <String>['rank', 'status'],
      );
    });

    test('typing is saved by itself after a pause', () async {
      final library = await open();
      final id = await makeNote(library, 'Auto', 'one');
      final editor = await library.edit(id: id);
      final events = <EditorEvent>[];
      final subscription = editor.events().listen(events.add);
      addTearDown(() => unawaited(subscription.cancel()));
      await Future<void>.delayed(const Duration(milliseconds: 200));

      final version = (await editor.snapshot()).version;
      editor.apply(change: _type(version, 3, 3, ' two'));
      expect((await editor.snapshot()).save, isA<SaveState_Pending>());
      // Not yet.
      expect((await library.note(id: id)).body, 'one');
      await _until(() async => (await library.note(id: id)).body == 'one two');
      await _until(() async => events.any((e) => e is EditorEvent_Saved));
      expect((await editor.snapshot()).save, isA<SaveState_Saved>());
    });

    test('a note changed elsewhere asks which version to keep', () async {
      for (final keepMine in <bool>[true, false]) {
        final library = await open();
        final id = await makeNote(library, 'Clash', 'base');
        final editor = await library.edit(id: id);
        final events = <EditorEvent>[];
        final subscription = editor.events().listen(events.add);
        addTearDown(() => unawaited(subscription.cancel()));
        await Future<void>.delayed(const Duration(milliseconds: 200));

        final version = (await editor.snapshot()).version;
        editor.apply(change: _type(version, 4, 4, ' mine'));
        final rev = (await library.note(id: id)).rev;
        await library.save(id: id, baseRev: rev, body: 'theirs');
        await _until(
          () async =>
              events.any((e) => e is EditorEvent_Incoming && !e.deleted),
        );
        expect(
          (await editor.snapshot()).save,
          isA<SaveState_Conflict>(),
        );
        if (keepMine) {
          expect(await editor.keepMine(), isA<SaveState_Saved>());
          expect((await library.note(id: id)).body, 'base mine');
        } else {
          final shown = await editor.useTheirs();
          expect(shown.text, 'theirs');
          expect((await library.note(id: id)).body, 'theirs');
        }
      }

      // Nothing typed: the change is just taken.
      final library = await open();
      final id = await makeNote(library, 'Calm', 'base');
      final editor = await library.edit(id: id);
      final events = <EditorEvent>[];
      final subscription = editor.events().listen(events.add);
      addTearDown(() => unawaited(subscription.cancel()));
      await Future<void>.delayed(const Duration(milliseconds: 200));
      final rev = (await library.note(id: id)).rev;
      await library.save(id: id, baseRev: rev, body: 'elsewhere');
      await _until(() async => events.any((e) => e is EditorEvent_Reloaded));
      expect((await editor.snapshot()).text, 'elsewhere');
    });

    test('a locked note is saved sealed when its lock closes', () async {
      final library = await open();
      final locks = library.locks();
      final id = await makeNote(library, 'Secret', 'the SECRETWORDS');
      await locks.lockNote(id: id, password: lockPassword);
      final editor = await library.edit(id: id);
      final events = <EditorEvent>[];
      final subscription = editor.events().listen(events.add);
      addTearDown(() => unawaited(subscription.cancel()));
      await Future<void>.delayed(const Duration(milliseconds: 200));

      final snapshot = await editor.snapshot();
      expect(snapshot.lock, isA<LockState_Open>());
      editor.apply(change: _type(snapshot.version, 0, 0, 'TYPED '));

      await locks.lock();
      await expectLater(editor.snapshot(), throwsA(isA<KansoError_Locked>()));
      expect(
        () => editor.apply(change: _type(snapshot.version + 1, 0, 0, 'x')),
        throwsA(isA<KansoError_Locked>()),
      );
      await expectLater(
        editor.styledRuns(start: 0, end: 4, themeId: 'kanso-light'),
        throwsA(isA<KansoError_Locked>()),
      );
      await expectLater(editor.completion(), throwsA(isA<KansoError_Locked>()));
      await expectLater(
        library.edit(id: id),
        throwsA(isA<KansoError_Locked>()),
      );
      await _until(() async => events.any((e) => e is EditorEvent_Closed));
      expect(events.last, isA<EditorEvent_Closed>());
      expect((await library.note(id: id)).body, isNull);

      expect(await locks.unlock(password: lockPassword), isTrue);
      expect((await library.note(id: id)).body, 'TYPED the SECRETWORDS');
    });

    test('a note shared with nobody has no comments or other people in '
        'it, and nothing to comment on', () async {
      final library = await open();
      final id = await makeNote(library, 'Plan', 'hello world');
      final editor = await library.edit(id: id);

      expect(await editor.commentMarks(), isEmpty);
      expect(await editor.remoteCursors(), isEmpty);
      expect(await editor.selectComment(commentId: 'c'), isNull);
      editor.select(
        selection: const Range16(start: 0, end: 5),
        composing: false,
      );
      await expectLater(
        editor.commentOnSelection(body: 'Why?'),
        throwsA(isA<KansoError>()),
      );
      // The refusal leaves the text as it was.
      expect((await editor.snapshot()).text, 'hello world');
      await editor.close();
    });
  });
}

Range16 _caret(int at) => Range16(start: at, end: at);

/// The change a field makes when [text] replaces units [start] to [end].
TextChange _type(int version, int start, int end, String text) => TextChange(
  baseVersion: version,
  start: start,
  end: end,
  text: text,
  selection: _caret(start + text.length),
);

/// Waits until [ready], for at most ten seconds.
Future<void> _until(Future<bool> Function() ready) async {
  final deadline = DateTime.now().add(const Duration(seconds: 10));
  while (!await ready()) {
    if (DateTime.now().isAfter(deadline)) {
      fail('timed out waiting');
    }
    await Future<void>.delayed(const Duration(milliseconds: 25));
  }
}

/// The eight bytes every PNG starts with: enough for the core to know what
/// the file is.
const List<int> _pngSignature = <int>[
  0x89,
  0x50,
  0x4E,
  0x47,
  0x0D,
  0x0A,
  0x1A,
  0x0A,
];
