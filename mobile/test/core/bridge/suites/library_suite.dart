// Kanso — library_suite.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';

import 'open_library.dart';

/// The notes of a library, through the bridge: every call of `KansoLibrary`.
/// It runs on this computer (`test/core/bridge`) and on a simulator or an
/// emulator (`integration_test`).
void librarySuite(OpenLibrary open) {
  group('KansoLibrary', () {
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
          await library.change(
            ids: <String>[first],
            change: const NoteChange.togglePin(),
          ),
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
        await library.change(
          ids: <String>[a, b],
          change: NoteChange.move(notebookId: garden.id),
        ),
        2,
      );
      expect(
        (await library.note(id: a)).notebookId,
        garden.id,
      );
      expect(
        await library.change(
          ids: <String>[a, b],
          change: NoteChange.move(notebookId: garden.id),
        ),
        0,
        reason: 'already there',
      );
      expect(
        await library.change(
          ids: <String>[a],
          change: const NoteChange.trash(),
        ),
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

    test('the parts of a library come from it', () async {
      final library = await open();
      expect(library.locks(), isA<Object>());
      expect(library.syncing(), isA<Object>());
      expect(library.renderer(), isA<Object>());
    });
  });
}
