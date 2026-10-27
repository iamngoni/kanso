// Kanso — library_suite.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/import.dart';
import 'package:kanso/core/bridge/api/library.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:path/path.dart' as p;

import 'open_library.dart';

/// The notes of a library, through the bridge: every call of `KansoLibrary`.
/// It runs on this computer (`test/core/bridge`) and on a simulator or an
/// emulator (`integration_test`).
void librarySuite(OpenLibrary open) {
  group('KansoLibrary', () {
    test('imports an export and opens a Markdown document', () async {
      final library = await open();
      final folder = await Directory.systemTemp.createTemp('kanso_import_');
      addTearDown(() => folder.delete(recursive: true));
      final export = File('${folder.path}/export.md');
      await export.writeAsString('# Export\n\nFrom another app.');
      final document = File('${folder.path}/opened.md');
      await document.writeAsString('# Opened\n\nFrom another app.');

      final sources = importSources();
      expect(sources.map((source) => source.kind), contains('markdown'));
      expect(importKindsFor(path: export.path), contains('markdown'));

      final preview = await library
          .importPlan(kind: 'markdown', path: export.path)
          .toList();
      final planned = preview.whereType<ImportEvent_Done>().single.field0;
      expect(planned.dryRun, isTrue);
      expect(planned.found.notes, 1);

      final imported = await library
          .import_(kind: 'markdown', path: export.path)
          .toList();
      expect(
        imported.whereType<ImportEvent_Done>().single.field0.dryRun,
        isFalse,
      );

      final opened = await library.openDocuments(
        files: <OpenedFile>[
          OpenedFile(path: document.path, name: document.uri.pathSegments.last),
        ],
        staging: '${folder.path}/staging',
      );
      expect(opened.noteId, isNotNull);
      expect(opened.problems, isEmpty);
    });

    test('history, Replace and backups work through the bridge', () async {
      final library = await open();
      expect(backupsSummary(), isNotEmpty);
      expect(replaceApplyLabel(matches: 2, notes: 1), isNotEmpty);

      final made = await library.create(
        note: const NewNote(title: 'Versions', body: 'needle needle'),
      );
      final initial = await library.note(id: made.id);
      final saved = await library.save(
        id: made.id,
        baseRev: initial.rev,
        body: 'needle again',
      );
      final history = await library.history(id: made.id);
      expect(history.versions, isNotEmpty);
      expect(
        await library.historyDiff(
          id: made.id,
          rev: history.versions.last.rev,
        ),
        isNotEmpty,
      );
      await library.restoreVersion(
        id: made.id,
        rev: history.versions.last.rev,
        baseRev: saved.rev,
      );

      const options = ReplaceOptions(
        find: 'needle',
        matchCase: false,
        wholeWord: false,
        regex: false,
        skipCode: false,
        includeTrash: false,
      );
      final preview = await library.replacePreview(options: options);
      final planned = preview.notes
          .map((note) => PlannedNote(id: note.id, rev: note.rev))
          .toList();
      final replaced = await library.replaceApply(
        options: options,
        replacement: 'seed',
        planned: planned,
      );
      expect(replaced.replaced, greaterThan(0));
      await library.replaceUndo(batch: replaced.batch);

      expect(await library.backupIfDue(), isNotNull);
      final manual = await library.backUpNow();
      expect(manual.manual, isTrue);
      expect(await library.backups(), isNotEmpty);
      await library.deleteBackup(path: manual.path);
    });
    test('a new library holds no notes', () async {
      final library = await open();
      expect((await library.list(query: everyNote)).rows, isEmpty);
      expect(await library.tags(), isEmpty);
      final counts = await library.overview();
      expect(
        [counts.all, counts.pinned, counts.recent, counts.trash],
        everyElement(0),
      );
    });

    test('creates, reads, saves and trashes a note', () async {
      final library = await open();
      final saved = await library.create(
        note: const NewNote(title: 'First', body: 'hello **world**'),
      );
      expect(saved.id, isNotEmpty);

      final rows = (await library.list(query: everyNote)).rows;
      expect(rows, hasLength(1));
      expect(rows.single.id, saved.id);
      expect(rows.single.title, 'First');
      expect(rows.single.snippet, contains('hello'));
      expect(rows.single.locked, isFalse);

      final note = await library.note(id: saved.id);
      expect(note.body, 'hello **world**');
      expect(note.lock, isA<LockState_Plain>());
      expect(note.rev, saved.rev);

      final changed = await library.save(
        id: saved.id,
        baseRev: note.rev,
        body: 'changed text',
      );
      expect(changed.rev, isNot(saved.rev));
      expect((await library.note(id: saved.id)).body, 'changed text');

      await library.trash(id: saved.id);
      expect((await library.list(query: everyNote)).rows, isEmpty);
      final trashed = (await library.list(query: trashedNotes)).rows;
      expect(trashed.map((row) => row.id), <String>[saved.id]);
    });

    test(
      'a stale save is a conflict and an unknown note is not found',
      () async {
        final library = await open();
        final saved = await library.create(
          note: const NewNote(title: 'Plan', body: 'one'),
        );
        await library.save(id: saved.id, baseRev: saved.rev, body: 'two');
        await expectLater(
          library.save(id: saved.id, baseRev: saved.rev, body: 'three'),
          throwsA(isA<KansoError_Conflict>()),
        );
        await expectLater(
          library.note(id: 'note:nope'),
          throwsA(isA<KansoError_NotFound>()),
        );
      },
    );

    test(
      "a new note takes its list's place and the preferred notebook",
      () async {
        final library = await open();
        final first = await library.createNotebook(name: 'Archive');
        final second = await library.createNotebook(name: 'Zed');
        await makeNote(library, 'Seed', 'plan #veg');
        final tag = (await library.tags()).single;

        Future<NoteView> made(NewNote note) async =>
            library.note(id: (await library.create(note: note)).id);

        // Nothing said: the first notebook; a preference is taken when it
        // exists, and passed over when it doesn't.
        expect(
          (await made(const NewNote(title: 'A', body: ''))).notebookId,
          first.id,
        );
        expect(
          (await made(
            NewNote(title: 'B', body: '', preferredNotebookId: second.id),
          )).notebookId,
          second.id,
        );
        expect(
          (await made(
            const NewNote(
              title: 'C',
              body: '',
              preferredNotebookId: 'book:gone',
            ),
          )).notebookId,
          first.id,
        );
        // A notebook's own list wins over the preference.
        expect(
          (await made(
            NewNote(
              title: 'D',
              body: '',
              place: ListPlace.notebook(id: first.id),
              preferredNotebookId: second.id,
            ),
          )).notebookId,
          first.id,
        );
        // A tag's list tags the note; Pinned pins it; the trash's goes nowhere.
        final tagged = await made(
          NewNote(
            title: 'E',
            body: '',
            place: ListPlace.tag(id: tag.id),
          ),
        );
        expect(tagged.tags, <String>[tag.id]);
        final pinned = await made(
          const NewNote(title: 'F', body: '', place: ListPlace.pinned()),
        );
        expect(pinned.pinned, isTrue);
        final fromTrash = await made(
          const NewNote(title: 'G', body: '', place: ListPlace.trash()),
        );
        expect(fromTrash.notebookId, first.id);
        expect(
          (await made(
            const NewNote(title: 'H', body: '', pinned: true),
          )).pinned,
          isTrue,
        );
      },
    );

    test('lists notebooks with their counts', () async {
      final library = await open();
      await makeNote(library, 'One', 'text');
      await makeNote(library, 'Two', 'text');
      final notebooks = await library.notebooks();
      expect(notebooks, isNotEmpty);
      expect(notebooks.fold<int>(0, (sum, book) => sum + book.count), 2);
      final inBook = await library.list(
        query: ListQuery(
          place: ListPlace.notebook(id: notebooks.first.id),
          pinnedOnly: false,
          sort: SortKey.title,
          descending: false,
        ),
      );
      expect(inBook.rows.map((row) => row.title), <String>['One', 'Two']);
      expect(inBook.shape.byDate, isFalse);
    });

    test('search finds a note and says where the words matched', () async {
      final library = await open();
      await makeNote(library, 'Grocery list', 'buy tulips on Tuesday');
      await makeNote(library, 'Other', 'nothing here');
      final hits = await library.search(
        text: 'tulips',
        scope: const SearchScope.all(),
        limit: 10,
      );
      expect(hits, hasLength(1));
      expect(hits.single.row.title, 'Grocery list');
      expect(hits.single.snippetRanges, isNotEmpty);
      final range = hits.single.snippetRanges.first;
      expect(
        hits.single.row.snippet.substring(range.start, range.end),
        'tulips',
      );
      expect(
        await library.search(
          text: 'zebra',
          scope: const SearchScope.all(),
          limit: 10,
        ),
        isEmpty,
      );
    });

    test(
      'a list is pinned first and laid out as the desktop lays it',
      () async {
        final library = await open();
        final first = await makeNote(library, 'First', 'a');
        final second = await makeNote(library, 'Second', 'b');
        expect(
          (await library.change(
            ids: <String>[first],
            change: const NoteChange.togglePin(),
          )).count,
          1,
        );
        final all = await library.list(query: everyNote);
        expect(all.rows.map((row) => row.id), <String>[first, second]);
        expect(all.rows.first.pinned, isTrue);
        expect(all.shape.byDate, isTrue);
        expect(all.shape.groupPinned, isTrue);
        expect(all.shape.date, DateField.updated);

        final recent = await library.list(
          query: const ListQuery(
            place: ListPlace.recent(),
            pinnedOnly: false,
            sort: SortKey.title,
            descending: false,
          ),
        );
        expect(recent.shape.sort, SortKey.updated);
        expect(recent.shape.groupPinned, isFalse);
        expect(recent.rows, hasLength(2));

        final pinned = await library.list(
          query: const ListQuery(
            place: ListPlace.pinned(),
            pinnedOnly: false,
            sort: SortKey.updated,
            descending: true,
          ),
        );
        expect(pinned.rows.map((row) => row.id), <String>[first]);
        final counts = await library.overview();
        expect(counts.all, 2);
        expect(counts.pinned, 1);
        expect(counts.recent, 2);
        expect(counts.trash, 0);
        await expectLater(
          library.list(
            query: const ListQuery(
              place: ListPlace.all(),
              statusOnly: 'someday',
              pinnedOnly: false,
              sort: SortKey.updated,
              descending: true,
            ),
          ),
          throwsA(isA<KansoError_Invalid>()),
        );
      },
    );

    test('several notes change at once: move, trash and unpin', () async {
      final library = await open();
      final a = await makeNote(library, 'A', 'a');
      final b = await makeNote(library, 'B', 'b');
      final garden = await library.createNotebook(name: 'Garden');
      expect(
        (await library.change(
          ids: <String>[a, b],
          change: NoteChange.move(notebookId: garden.id),
        )).count,
        2,
      );
      expect(
        (await library.note(id: a)).notebookId,
        garden.id,
      );
      expect(
        (await library.change(
          ids: <String>[a, b],
          change: NoteChange.move(notebookId: garden.id),
        )).count,
        0,
        reason: 'already there',
      );
      expect(
        (await library.change(
          ids: <String>[a],
          change: const NoteChange.trash(),
        )).count,
        1,
      );
      expect((await library.overview()).trash, 1);
      await expectLater(
        library.change(
          ids: <String>['note:nope'],
          change: const NoteChange.togglePin(),
        ),
        throwsA(isA<KansoError_NotFound>()),
      );
    });

    test('a change says what to tell, and Undo takes it back', () async {
      final library = await open();
      final a = await makeNote(library, 'A', 'a');
      final b = await makeNote(library, 'B', 'b');
      final garden = await library.createNotebook(name: 'Garden');
      final before = (await library.note(id: a)).notebookId;

      final moved = await library.change(
        ids: <String>[a, b],
        change: NoteChange.move(notebookId: garden.id),
      );
      expect(moved.count, 2);
      expect(moved.message, 'Moved 2 notes to Garden.');
      expect(moved.undo, isNotNull);
      expect((await library.note(id: a)).notebookId, garden.id);

      expect(await library.undo(ticket: moved.undo!), 2);
      expect((await library.note(id: a)).notebookId, before);
      expect((await library.note(id: b)).notebookId, before);
      expect(
        await library.undo(ticket: moved.undo!),
        0,
        reason: 'a ticket is used once',
      );

      final nothing = await library.change(
        ids: <String>[a],
        change: NoteChange.move(notebookId: before),
      );
      expect((nothing.count, nothing.undo, nothing.message), (0, null, ''));
    });

    test('Undo keeps a field that changed again since', () async {
      final library = await open();
      final a = await makeNote(library, 'A', 'a');
      final garden = await library.createNotebook(name: 'Garden');
      final shed = await library.createNotebook(name: 'Shed');
      final moved = await library.change(
        ids: <String>[a],
        change: NoteChange.move(notebookId: garden.id),
      );
      await library.change(
        ids: <String>[a],
        change: NoteChange.move(notebookId: shed.id),
      );
      expect(await library.undo(ticket: moved.undo!), 0);
      expect((await library.note(id: a)).notebookId, shed.id);
    });

    test('a ticked task is taken back with its ticket', () async {
      final library = await open();
      final saved = await library.create(
        note: const NewNote(title: 'Todo', body: '- [ ] one\n- [ ] two'),
      );
      final ticked = await library.setTask(
        noteId: saved.id,
        line: 0,
        text: 'one',
        checked: true,
      );
      expect(ticked, isA<Ticked_Changed>());
      expect(
        (await library.note(id: saved.id)).body,
        contains('- [x] one'),
      );
      expect(await library.undo(ticket: (ticked as Ticked_Changed).undo), 1);
      expect(
        (await library.note(id: saved.id)).body,
        contains('- [ ] one'),
      );
    });

    test('a search looks in titles, tags or a notebook', () async {
      final library = await open();
      // New notes go in the first notebook by name: not the garden.
      await library.createNotebook(name: 'Archive');
      final garden = await library.createNotebook(name: 'Zed');
      final named = await makeNote(library, 'Tomato plan', 'water #veg daily');
      final texted = await makeNote(library, 'Shopping', 'buy tomato seeds');
      final planted = (await library.create(
        note: NewNote(
          notebookId: garden.id,
          title: 'Beds',
          body: 'tomato rows',
        ),
      )).id;
      Future<Set<String>> found(String text, SearchScope scope) async {
        final hits = await library.search(text: text, scope: scope, limit: 0);
        return hits.map((hit) => hit.row.id).toSet();
      }

      expect(
        await found('tomato', const SearchScope.all()),
        <String>{named, texted, planted},
      );
      expect(await found('tomato', const SearchScope.titles()), <String>{
        named,
      });
      expect(
        await found('tomato', SearchScope.notebook(id: garden.id)),
        <String>{planted},
      );
      // The `#veg` in the text made the tag.
      expect(await found('veg', const SearchScope.tags()), <String>{named});
    });

    test(
      'notebooks come in tree order with depth, and text gives tags',
      () async {
        final library = await open();
        final work = await library.createNotebook(name: 'Work');
        final acme = await library.createNotebook(
          name: 'Acme',
          parentId: work.id,
        );
        expect((work.depth, acme.depth), (0, 1));
        expect(acme.parentId, work.id);
        expect(
          (await library.notebooks()).map((book) => (book.name, book.depth)),
          <(String, int)>[('Work', 0), ('Acme', 1)],
        );
        await expectLater(
          library.createNotebook(name: '  '),
          throwsA(isA<KansoError_Invalid>()),
        );
        await expectLater(
          library.createNotebook(name: 'X', parentId: 'book:nowhere'),
          throwsA(isA<KansoError_NotFound>()),
        );

        final id = await makeNote(library, 'Todo', 'call #errand soon');
        final tags = await library.tags();
        expect(tags.map((tag) => tag.name), <String>['errand']);
        expect((await library.note(id: id)).tags, <String>[tags.single.id]);
        final note = await library.note(id: id);
        await library.save(id: id, baseRev: note.rev, body: 'call soon');
        expect((await library.note(id: id)).tags, isEmpty);
      },
    );

    test('the properties of a note are its front matter', () async {
      final library = await open();
      final id = await makeNote(
        library,
        'P',
        '---\nrating: 4.5\ndone: true\nowner: Ada\n---\nbody',
      );
      final props = await library.properties(id: id);
      expect(props.map((p) => p.key), <String>['rating', 'done', 'owner']);
      expect(props.map((p) => p.kind), <PropertyKind>[
        PropertyKind.number,
        PropertyKind.bool,
        PropertyKind.text,
      ]);
      expect(props.last.text, 'Ada');
      final none = await makeNote(library, 'None', 'no front matter');
      expect(await library.properties(id: none), isEmpty);
    });

    test('a query view is a table, and a bad one is refused', () async {
      final library = await open();
      final a = await makeNote(
        library,
        'Alpha',
        '---\nstatus: open\nrank: 2\n---\n',
      );
      final b = await makeNote(
        library,
        'Beta',
        '---\nstatus: open\nrank: 1\n---\n',
      );
      final table = await library.queryView(
        source:
            'from: status:none has:rank\ncolumns: rank, status\n'
            'sort: rank asc',
      );
      expect(table.columns, <String>['Title', 'rank', 'status']);
      expect(table.rows.map((row) => row.id), <String>[b, a]);
      expect(table.rows.first.cells, <String?>['1', 'open']);
      final without = await library.queryView(
        source: 'from: status:none has:rank',
        selfId: a,
      );
      expect(without.rows.map((row) => row.id), <String>[b]);
      await expectLater(
        library.queryView(source: 'nonsense'),
        throwsA(isA<KansoError_Invalid>()),
      );
    });

    test('the changes stream reports what is written', () async {
      final library = await open();
      final written = <String>{};
      final heard = <String>{};
      final reported = Completer<void>();
      final subscription = library.changes().listen((ids) {
        heard.addAll(ids);
        if (heard.any(written.contains) && !reported.isCompleted) {
          reported.complete();
        }
      });
      // Cancelling waits for the library's next event: don't wait for it.
      addTearDown(() => unawaited(subscription.cancel()));
      // The stream is set up on a thread of the library's, so a note written
      // at once may come before it: write until one is reported. (The ID of
      // a new notebook may come first; it is not a note written here.)
      for (var i = 0; i < 40 && !reported.isCompleted; i++) {
        written.add(await makeNote(library, 'Change $i', 'text'));
        await Future.any(<Future<void>>[
          reported.future,
          Future<void>.delayed(const Duration(milliseconds: 250)),
        ]);
      }
      await reported.future.timeout(const Duration(seconds: 5));
      expect(heard.any(written.contains), isTrue);
    });

    test('saves the shared items waiting in a folder', () async {
      final library = await open();
      final folder = await Directory.systemTemp.createTemp('kanso_inbox_');
      addTearDown(() => folder.delete(recursive: true));
      final inbox = p.join(folder.path, 'inbox');

      // No folder yet: nothing to save.
      final none = await library.saveDrops(inbox: inbox);
      expect(none.notes, isEmpty);
      expect(none.problems, isEmpty);
      expect(none.kept, 0);

      const id = 'ab12ab12ab12ab12ab12ab12ab12ab12';
      final drop = Directory(p.join(inbox, id))..createSync(recursive: true);
      File(p.join(drop.path, 'drop.json')).writeAsStringSync(
        jsonEncode(<String, Object?>{
          'version': 1,
          'createdAt': 1,
          'subject': 'From the sheet',
          'text': 'Shared words',
        }),
      );
      final saved = await library.saveDrops(inbox: inbox);
      expect(saved.notes, <String>['note:cap-$id']);
      expect(saved.problems, isEmpty);
      expect(drop.existsSync(), isFalse);
      final note = await library.note(id: 'note:cap-$id');
      expect(note.title, 'From the sheet');
      expect(note.body, contains('Shared words'));
    });

    test('captures a share made in the app', () async {
      final library = await open();
      final outcome = await library.capture(
        share: Share(
          subject: 'Receipt',
          text: 'Lunch',
          files: <ShareFile>[
            ShareFile(
              name: 'Receipt.pdf',
              bytes: Uint8List.fromList(utf8.encode('%PDF-1.7\n%%EOF\n')),
            ),
            ShareFile(
              name: 'notes.zip',
              bytes: Uint8List.fromList(<int>[0x50, 0x4b]),
            ),
          ],
        ),
      );
      expect(outcome.noteId, startsWith('note:cap-'));
      expect(outcome.problems, hasLength(1));
      final note = await library.note(id: outcome.noteId);
      expect(note.title, 'Receipt');
      expect(note.body, contains('Receipt.pdf'));
      await expectLater(
        library.capture(share: const Share(files: <ShareFile>[])),
        throwsA(isA<KansoError_Invalid>()),
      );
    });

    test("opens today's note once, from the phone's date", () async {
      final library = await open();
      const morning = LocalTime(
        year: 2026,
        month: 10,
        day: 26,
        hour: 8,
        minute: 5,
        second: 0,
      );
      final id = await library.todayNote(now: morning);
      expect(id, 'note:daily-2026-10-26');
      const night = LocalTime(
        year: 2026,
        month: 10,
        day: 26,
        hour: 23,
        minute: 59,
        second: 59,
      );
      expect(await library.todayNote(now: night), id);
      expect((await library.note(id: id)).title, '2026-10-26');
      await expectLater(
        library.todayNote(
          now: const LocalTime(
            year: 2026,
            month: 2,
            day: 30,
            hour: 1,
            minute: 0,
            second: 0,
          ),
        ),
        throwsA(isA<KansoError_Invalid>()),
      );
    });

    test('lists open tasks in groups, ticks one and takes it back', () async {
      final library = await open();
      final id = await makeNote(
        library,
        'Plan',
        '- [ ] Pay rent @2020-01-01\n- [ ] Later @2999-01-01\n- [ ] Plain\n',
      );
      final board = await library.tasks(
        query: const TaskListQuery(
          showDone: false,
          kept: <String>[],
        ),
      );
      expect(
        board.groups.map((group) => group.group),
        <TaskGroupKind>[
          TaskGroupKind.overdue,
          TaskGroupKind.upcoming,
          TaskGroupKind.noDate,
        ],
      );
      expect(
        board.groups.map((group) => group.label),
        <String>['Overdue', 'Upcoming', 'No date'],
      );
      final rent = board.groups.first.items.single;
      expect(rent.text, 'Pay rent');
      expect(rent.sourceText, 'Pay rent @2020-01-01');
      expect(rent.noteId, id);
      expect(rent.noteTitle, 'Plan');
      expect(rent.line, 0);
      expect(rent.dueTone, DueTone.overdue);
      expect(rent.dueLabel, startsWith('Overdue'));
      expect(await library.overdueTasks(), 1);

      final ticked = await library.setTask(
        noteId: id,
        line: rent.line,
        text: rent.sourceText,
        checked: true,
      );
      expect(ticked, isA<Ticked_Changed>());
      expect(await library.overdueTasks(), 0);
      final kept = await library.tasks(
        query: TaskListQuery(
          showDone: false,
          kept: <String>[rent.key],
        ),
      );
      expect(kept.groups.first.group, TaskGroupKind.overdue);
      expect(kept.groups.first.items.single.checked, isTrue);

      expect(await library.undo(ticket: (ticked as Ticked_Changed).undo), 1);
      expect(await library.overdueTasks(), 1);
    });

    test('leaves the tasks of locked notes out', () async {
      final library = await open();
      final visible = await makeNote(library, 'Seen', '- [ ] open @2020-01-01');
      final hidden = await makeNote(library, 'Hidden', '- [ ] SEALEDTASK');
      final locks = library.locks();
      await locks.lockNote(id: hidden, password: lockPassword);
      expect(await locks.isUnlocked(), isTrue);
      for (var round = 0; round < 2; round++) {
        final board = await library.tasks(
          query: const TaskListQuery(
            showDone: true,
            kept: <String>[],
          ),
        );
        final items = board.groups.expand((group) => group.items).toList();
        expect(items.map((item) => item.noteId), <String>[visible]);
        expect(items.map((item) => item.text), isNot(contains('SEALEDTASK')));
        expect(await library.overdueTasks(), 1);
        await locks.lock();
      }
    });

    test('the parts of a library come from it', () async {
      final library = await open();
      expect(library.locks(), isA<Object>());
      expect(library.syncing(), isA<Object>());
      expect(library.renderer(), isA<Object>());
    });

    test('a task box in the page is ticked against the text drawn', () async {
      final library = await open();
      final id = await makeNote(library, 'Chores', '- [ ] Sweep\n');
      final rev = (await library.note(id: id)).rev;
      final ticked = await library.toggleTaskAt(
        noteId: id,
        index: 0,
        baseRev: rev,
      );
      expect(ticked, isA<Ticked_Changed>());
      expect((await library.note(id: id)).body, contains('- [x] Sweep'));
      // A page drawn before the tick is out of date.
      await expectLater(
        library.toggleTaskAt(noteId: id, index: 0, baseRev: rev),
        throwsA(isA<KansoError_Conflict>()),
      );
    });

    test('a wiki-link to no note makes it, once', () async {
      final library = await open();
      final from = await makeNote(library, 'From', '[[Seed]]');
      final made = await library.createLinkedNote(
        title: 'Seed',
        fromNote: from,
      );
      expect((await library.note(id: made.id)).title, 'Seed');
      final again = await library.createLinkedNote(
        title: 'Seed',
        fromNote: from,
      );
      expect(again.id, made.id);
    });

    test('an attachment no readable note links is not found', () async {
      final library = await open();
      await expectLater(
        library.attachment(id: 'file:nope'),
        throwsA(isA<KansoError_NotFound>()),
      );
    });

    test('notebooks take an icon and a color, and are renamed', () async {
      final library = await open();
      final work = await library.createNotebook(name: 'Work');

      expect(KansoLibrary.notebookIcons(query: ''), isNotEmpty);
      expect(
        KansoLibrary.notebookIcons(query: 'fold').map((icon) => icon.name),
        contains('folder'),
      );
      expect(KansoLibrary.notebookColors(), isNotEmpty);
      expect(KansoLibrary.tagColors().first.name, 'default');
      expect(KansoLibrary.parseColor(text: '#FA0'), '#ffaa00');
      expect(KansoLibrary.parseColor(text: '  #2185D0 '), '#2185d0');
      expect(KansoLibrary.parseColor(text: 'blue'), isNull);
      expect(
        KansoLibrary.statuses().map((status) => status.status),
        <String>['none', 'active', 'onHold', 'completed', 'dropped'],
      );

      await library.setNotebookIcon(id: work.id, icon: 'folder');
      await library.setNotebookColor(id: work.id, color: '#2185D0');
      var books = await library.notebooks();
      expect(books.single.icon, 'folder');
      expect(books.single.color, '#2185d0');
      await library.setNotebookIcon(id: work.id);
      await library.setNotebookColor(id: work.id);
      books = await library.notebooks();
      expect((books.single.icon, books.single.color), (null, null));
      await expectLater(
        library.setNotebookIcon(id: work.id, icon: 'nonsense'),
        throwsA(isA<KansoError_Invalid>()),
      );
      await expectLater(
        library.setNotebookColor(id: work.id, color: 'blue'),
        throwsA(isA<KansoError_Invalid>()),
      );

      await library.renameNotebook(id: work.id, name: 'Studio');
      expect((await library.notebooks()).single.name, 'Studio');
      await expectLater(
        library.renameNotebook(id: work.id, name: '  '),
        throwsA(isA<KansoError_Invalid>()),
      );
      await expectLater(
        library.renameNotebook(id: 'book:nope', name: 'Nope'),
        throwsA(isA<KansoError_NotFound>()),
      );
    });

    test('tags are made, renamed, coloured and deleted', () async {
      final library = await open();
      final tag = await library.createTag(name: ' Errand ');
      expect(tag.name, 'Errand');
      await library.renameTag(id: tag.id, name: 'Errands');
      expect((await library.tags()).single.name, 'Errands');
      await library.setTagColor(id: tag.id, color: 'red');
      expect((await library.tags()).single.color, isNot(0));
      await expectLater(
        library.setTagColor(id: tag.id, color: 'mauve'),
        throwsA(isA<KansoError_Invalid>()),
      );
      await expectLater(
        library.createTag(name: ' '),
        throwsA(isA<KansoError_Invalid>()),
      );
      await library.deleteTag(id: tag.id);
      expect(await library.tags(), isEmpty);
      await expectLater(
        library.deleteTag(id: tag.id),
        throwsA(isA<KansoError_NotFound>()),
      );
    });

    test('a note is copied, linked and deleted for good', () async {
      final library = await open();
      final note = await makeNote(library, 'Plans', 'body');
      final copy = await library.duplicate(id: note);
      expect(copy.id, isNot(note));
      expect((await library.note(id: copy.id)).title, contains('Plans'));

      final wiki = await library.noteLink(id: note, kanso: false);
      expect(wiki.text, '[[Plans]]');
      expect(wiki.message, 'Link copied.');
      final kanso = await library.noteLink(id: note, kanso: true);
      expect(kanso.text, contains('Plans'));
      expect(kanso.message, 'Link copied.');

      await library.change(
        ids: <String>[note],
        change: const NoteChange.setStatus(status: 'completed'),
      );
      final done = (await library.list(query: everyNote)).rows
          .firstWhere((row) => row.id == note);
      expect(done.status, 'completed');

      await library.change(
        ids: <String>[copy.id],
        change: const NoteChange.trash(),
      );
      await library.change(
        ids: <String>[copy.id],
        change: const NoteChange.restore(),
      );
      expect(
        (await library.list(query: everyNote)).rows.map((row) => row.id),
        contains(copy.id),
      );
      final doomedCopy = await makeNote(library, 'Doomed copy', 'x');
      await library.change(
        ids: <String>[doomedCopy],
        change: const NoteChange.trash(),
      );
      await library.change(
        ids: <String>[doomedCopy],
        change: const NoteChange.deleteForever(),
      );
      expect(
        (await library.list(query: trashedNotes)).rows.map((row) => row.id),
        isNot(contains(doomedCopy)),
      );
      expect(
        (await library.list(query: everyNote)).rows.map((row) => row.id),
        isNot(contains(doomedCopy)),
      );

      final doomed = await makeNote(library, 'Doomed', 'x');
      await library.change(
        ids: <String>[doomed],
        change: const NoteChange.trash(),
      );
      expect(await library.emptyTrash(), greaterThanOrEqualTo(1));
      expect((await library.overview()).trash, 0);
    });

    test('the sidebar folds notebooks and lists the tags', () async {
      final library = await open();
      final garden = await library.createNotebook(name: 'Garden');
      final bed = await library.createNotebook(
        name: 'Bed',
        parentId: garden.id,
      );
      await makeNote(library, 'Seed', '#veg');

      const closed = FoldState(open: <String>[], folded: <String>[]);
      var rows = await library.visibleNotebooks(folds: closed);
      expect(rows.map((row) => row.row.id), <String>[garden.id]);
      expect(rows.single.parent, isTrue);
      expect(rows.single.open, isFalse);

      // A tap on the arrow opens it; another folds it again.
      final opened = await library.toggleFold(folds: closed, id: garden.id);
      expect(opened.open, contains(garden.id));
      rows = await library.visibleNotebooks(folds: opened);
      expect(rows.map((row) => row.row.id), <String>[garden.id, bed.id]);
      expect(rows.first.open, isTrue);
      final folded = await library.toggleFold(folds: opened, id: garden.id);
      expect(folded.open, isNot(contains(garden.id)));
      expect(
        (await library.visibleNotebooks(folds: folded)).map(
          (row) => row.row.id,
        ),
        <String>[garden.id],
      );

      // The notebook showing keeps the ones around it open.
      final around = await library.visibleNotebooks(
        folds: closed,
        showing: bed.id,
      );
      expect(around.map((row) => row.row.id), <String>[garden.id, bed.id]);

      // A leaf tag sits in the chips; nested names fold into a tree.
      var places = await library.tagPlaces(
        all: true,
        openPaths: const <String>[],
      );
      expect(places.chips.map((tag) => tag.name), <String>['veg']);
      expect(places.more, isFalse);
      expect(places.tree, isEmpty);
      final veg = places.chips.single;

      // The selected tag's ID keeps its chip when only a few show.
      places = await library.tagPlaces(
        all: false,
        openPaths: const <String>[],
        selected: veg.id,
      );
      expect(places.chips.map((tag) => tag.id), contains(veg.id));

      // A name that nests another tag becomes a level; the rows below it
      // show once the level opens.
      await library.createTag(name: 'veg/salad');
      places = await library.tagPlaces(all: true, openPaths: const <String>[]);
      expect(places.tree.map((row) => row.path), <String>['veg']);
      expect(places.tree.single.open, isFalse);
      expect(places.tree.single.parent, isTrue);
      expect(places.tree.single.count, 1);
      // Nesting moves a tag out of the chips: it lists in the tree instead.
      expect(places.chips, isEmpty);
      places = await library.tagPlaces(
        all: true,
        openPaths: const <String>['veg'],
      );
      expect(places.tree.map((row) => row.path), <String>['veg', 'veg/salad']);
      expect(places.tree.first.open, isTrue);
      expect(places.tree.last.open, isFalse);
      expect(places.tree.last.tagId, isNotNull);

      await library.deleteNotebook(id: garden.id);
      expect(
        (await library.notebooks()).map((book) => book.id),
        isNot(contains(bed.id)),
      );
    });
  });
}
