// Kanso — locked_property_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math';

import 'package:dartz/dartz.dart' show Either;
import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show ExploreConfig, Glados2, IntAnys, ListAnys, any;
import 'package:kanso/core/bridge/api/library.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/impl/rust_library_repository.dart';
import 'package:kanso/core/models/repos/impl/rust_renderer_repository.dart';
import 'package:kanso/core/models/repos/impl/rust_session_repository.dart';
import 'package:kanso/editor/models/repos/abs/editor_repository.dart';
import 'package:kanso/editor/models/repos/impl/rust_editor_repository.dart';

import '../../helpers/core_library.dart';
import '../../helpers/dump.dart';
import 'suites/open_library.dart';

/// A locked note's text never reaches Dart while it is closed: whatever the
/// repositories are asked, in whatever order, nothing they return holds the
/// words only the text has. The same property the Rust tests check
/// (`crates/kanso-mobile/tests/locked.rs`), through the bridge and the
/// repositories, so a cache or a copy on the Dart side would fail it.
void main() {
  setUpAll(initCoreForTests);

  // The property above is only as good as its marker: an open note does
  // show its text (search never reads a locked note, open or not).
  test('control: while the lock is open the marker shows', () async {
    final library = await openTestLibrary();
    final notes = LibraryRepositoryUnderTest(library);
    final id = await makeNote(
      library,
      'Title',
      '---\nowner: $_marker\n---\nbody of $_marker',
    );
    await library.locks().lockNote(id: id, password: lockPassword);
    expect(dump(await notes.library.note(id)), contains(_marker));
    expect(dump(await notes.library.properties(id)), contains(_marker));
    await library.locks().lock();
    expect(dump(await notes.library.note(id)), isNot(contains(_marker)));
  });

  // The same for a note open for editing: its session shows the text while
  // the lock is open, and once it closes the session is over and says only so.
  test(
    'control: a session shows the marker while open, nothing after',
    () async {
      final library = await openTestLibrary();
      final notes = LibraryRepositoryUnderTest(library);
      final id = await makeNote(
        library,
        'Title',
        '---\nowner: $_marker\n---\nbody of $_marker',
      );
      await library.locks().lockNote(id: id, password: lockPassword);
      final document = (await notes.editor.open(id)).getOrElse(
        () => throw StateError('the open note did not open'),
      );
      expect(dump(await document.snapshot()), contains(_marker));
      expect(
        dump(
          await document.styledRuns(start: 0, end: 20, themeId: 'kanso-light'),
        ),
        isNotEmpty,
      );
      await library.locks().lock();
      for (final result in <Either<ApplicationError, Object?>>[
        await document.snapshot(),
        await document.styledRuns(start: 0, end: 20, themeId: 'kanso-light'),
        await document.completion(),
        await document.flush(),
        document.undo(),
        await notes.editor.open(id),
      ]) {
        expect(dump(result), isNot(contains(_marker)));
        expect(
          result.isLeft(),
          isTrue,
          reason: dump(result),
        );
      }
    },
  );

  Glados2<int, List<int>>(
    any.intInRange(1, 8),
    any.listWithLengthInRange(1, 14, any.intInRange(0, 85)),
    ExploreConfig(random: Random(20261017), numRuns: 10),
  ).test(
    'a closed note never returns its text, whatever is asked',
    (lockedMask, operations) async {
      final library = await openTestLibrary();
      final notes = LibraryRepositoryUnderTest(library);
      // Three notes; the ones the mask picks are locked. Each text holds a
      // marker that only the text has.
      final ids = <String>[];
      for (var i = 0; i < 3; i++) {
        ids.add(
          await makeNote(
            library,
            'Title $i',
            '---\nowner: $_marker$i\nrank: $i\n---\nbody of $_marker$i',
          ),
        );
      }
      final lockedIndexes = <int>[
        for (var i = 0; i < 3; i++)
          if (lockedMask & (1 << i) != 0) i,
      ];
      for (final i in lockedIndexes) {
        await library.locks().lockNote(id: ids[i], password: lockPassword);
      }
      await library.locks().lock();

      var closed = true;
      var unlocks = 0;
      final seen = StringBuffer();
      // The notes open for editing, and those whose session the lock ended.
      final documents = <int, EditorDocument>{};
      final ended = <int>{};

      Future<String> perform(int operation) async {
        final index = operation ~/ 14 % 3;
        switch (operation % 14) {
          case 0:
            final place = <ListPlace>[
              const ListPlace.all(),
              const ListPlace.pinned(),
              const ListPlace.recent(),
            ][index];
            return dump(
              await notes.library.list(
                ListQuery(
                  place: place,
                  pinnedOnly: false,
                  sort: SortKey.updated,
                  descending: true,
                ),
              ),
            );
          case 1:
            return dump(await notes.library.note(ids[index]));
          case 2:
            final scope = <SearchScope>[
              const SearchScope.all(),
              const SearchScope.titles(),
              const SearchScope.tags(),
            ][index];
            return dump(await notes.library.search(_marker, scope: scope));
          case 3:
            return dump(await notes.library.properties(ids[index]));
          case 4:
            return dump(
              await notes.library.queryView(
                'from: status:none has:rank\ncolumns: owner, rank',
              ),
            );
          case 5:
            return dump(await notes.renderer.previewHtml(ids[index], _palette));
          case 6:
            return dump(
              <Object?>[
                await notes.library.save(
                  id: ids[index],
                  baseRev: 'x',
                  body: 'edited',
                ),
                await notes.library.change(<String>[
                  ids[index],
                ], const NoteChange.togglePin()),
                await notes.library.overview(),
              ],
            );
          case 7:
            return dump(
              await notes.library.create(
                const NewNote(title: 'Another', body: 'plain'),
              ),
            );
          case 8:
            closed = true;
            final locked = await notes.session.lock();
            // Sessions on locked notes end with the lock.
            ended.addAll(
              documents.keys.where(lockedIndexes.contains),
            );
            return dump(locked);
          case 10:
            // Opening a note for editing: refused while it is closed.
            final opened = await notes.editor.open(ids[index]);
            final isClosed = closed && lockedIndexes.contains(index);
            expect(opened.isLeft(), isClosed, reason: 'opening $index');
            opened.fold(
              (error) {
                expect(error, isA<LockedError>());
              },
              (document) {
                documents[index] = document;
                ended.remove(index);
              },
            );
            return dump(opened.map((_) => 'a document'));
          case 11:
            // Every call of a session: all of them refused once it ended,
            // none of them holding a closed note's text.
            final document = documents[index];
            if (document == null) {
              return 'no session';
            }
            final snapshot = await document.snapshot();
            final version = snapshot.fold((_) => 0, (s) => s.version);
            final results = <Either<ApplicationError, Object?>>[
              snapshot,
              await document.styledRuns(
                start: 0,
                end: 200,
                themeId: 'kanso-light',
              ),
              await document.completion(),
              await document.linkAt(0),
              await document.setProperty(key: 'added', value: '1'),
              await document.flush(),
              document.select(
                const Range16(start: 0, end: 0),
                composing: false,
              ),
              document.undo(),
              document.apply(
                TextChange(
                  baseVersion: version,
                  start: 0,
                  end: 0,
                  text: '[[T',
                  selection: const Range16(start: 3, end: 3),
                ),
              ),
            ];
            for (final result in results) {
              final failed = result.swap().toOption();
              if (ended.contains(index)) {
                expect(failed.isSome(), isTrue, reason: 'an ended session');
                expect(failed.toNullable(), isA<LockedError>());
              } else {
                expect(failed.isNone(), isTrue, reason: dump(result));
              }
            }
            return dump(results);
          case 12:
            // Typing `[[` and asking what it completes: titles.
            final document = documents[index];
            if (document == null) {
              return 'no session';
            }
            final completion = await document.completion();
            final text = dump(completion);
            // Titles only, whoever asks and whatever is open.
            expect(text, isNot(contains(_marker)));
            return text;
          default:
            // Real key derivation: a few times a case, not more.
            if (unlocks >= 2) {
              return 'skipped';
            }
            unlocks++;
            final unlocked = await notes.session.unlock(lockPassword);
            closed = false;
            return '$unlocked';
        }
      }

      for (final operation in operations) {
        final returned = await perform(operation);
        if (closed) {
          seen.writeln(returned);
        }
      }
      // Only while closed: an open session may show its text.
      final lockedMarkers = <String>[
        for (final i in lockedIndexes) '$_marker$i',
      ];
      for (final marker in lockedMarkers) {
        expect(
          seen.toString(),
          isNot(contains(marker)),
          reason: 'a closed locked note gave back $marker',
        );
      }
    },
    timeout: const Timeout(Duration(minutes: 5)),
  );
}

const String _marker = 'SECRETMARKER';

const PaletteDto _palette = PaletteDto(
  dark: false,
  background: 0xFFFFFF,
  text: 0x222222,
  muted: 0x888888,
  accent: 0x3366AA,
  border: 0xDDDDDD,
  surface: 0xF4F4F4,
  quote: 0x666666,
  taskDone: 0x888888,
  inlineCodeBackground: 0xEEEEEE,
);

/// The repositories over one library.
class LibraryRepositoryUnderTest {
  LibraryRepositoryUnderTest(KansoLibrary library)
    : library = RustLibraryRepository(library),
      session = RustSessionRepository(library.locks()),
      renderer = RustRendererRepository(library.renderer()),
      editor = RustEditorRepository(library);

  final RustLibraryRepository library;
  final RustSessionRepository session;
  final RustRendererRepository renderer;
  final RustEditorRepository editor;
}
