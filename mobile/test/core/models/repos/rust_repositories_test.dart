// Kanso — rust_repositories_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/library.dart';
import 'package:kanso/core/bridge/api/locks.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/impl/rust_library_repository.dart';
import 'package:kanso/core/models/repos/impl/rust_session_repository.dart';
import 'package:mocktail/mocktail.dart';

class _MockLibrary extends Mock implements KansoLibrary {}

class _MockLocks extends Mock implements KansoLocks {}

const _query = ListQuery(
  place: ListPlace.all(),
  pinnedOnly: false,
  sort: SortKey.updated,
  descending: true,
);

const _shape = ListShape(
  sort: SortKey.updated,
  byDate: true,
  date: DateField.updated,
  groupPinned: true,
);

void main() {
  group('RustLibraryRepository', () {
    late _MockLibrary library;
    late RustLibraryRepository repository;

    setUp(() {
      library = _MockLibrary();
      repository = RustLibraryRepository(library);
    });

    test('returns the notes the library lists', () async {
      const empty = NoteList(rows: <NoteRow>[], shape: _shape);
      when(() => library.list(query: _query)).thenAnswer((_) async => empty);
      expect(
        await repository.list(_query),
        const Right<ApplicationError, NoteList>(empty),
      );
    });

    test('a note the library does not have is a not-found error', () async {
      when(() => library.note(id: 'n_1')).thenAnswer(
        (_) => Future<NoteView>.error(const KansoError.notFound()),
      );
      final result = await repository.note('n_1');
      expect(result.swap().toOption().toNullable(), isA<NotFoundError>());
    });

    test('trashing returns unit', () async {
      when(() => library.trash(id: 'n_1')).thenAnswer((_) async {});
      expect(
        await repository.trash('n_1'),
        const Right<ApplicationError, Unit>(unit),
      );
    });

    test(
      'a change returns how many notes were written, and its ticket',
      () async {
        const change = NoteChange.move(notebookId: 'b_2');
        when(
          () => library.change(ids: <String>['n_1', 'n_2'], change: change),
        ).thenAnswer(
          (_) async => const Changed(
            count: 2,
            undo: 5,
            message: 'Moved 2 notes to Work.',
          ),
        );
        expect(
          await repository.change(<String>['n_1', 'n_2'], change),
          const Right<ApplicationError, Changed>(
            Changed(count: 2, undo: 5, message: 'Moved 2 notes to Work.'),
          ),
        );
      },
    );

    test('a change into a locked notebook is a locked error', () async {
      when(
        () => library.change(
          ids: <String>['n_1'],
          change: const NoteChange.togglePin(),
        ),
      ).thenAnswer((_) => Future<Changed>.error(const KansoError.locked()));
      final result = await repository.change(
        <String>['n_1'],
        const NoteChange.togglePin(),
      );
      expect(result.swap().toOption().toNullable(), isA<LockedError>());
    });

    test(
      'a search looks in all by default and in the scope it is given',
      () async {
        when(
          () => library.search(
            text: 'tulips',
            scope: const SearchScope.all(),
            limit: 50,
          ),
        ).thenAnswer((_) async => const <Hit>[]);
        when(
          () => library.search(
            text: 'tulips',
            scope: const SearchScope.notebook(id: 'b_1'),
            limit: 5,
          ),
        ).thenAnswer((_) async => const <Hit>[]);
        expect(
          await repository.search('tulips'),
          const Right<ApplicationError, List<Hit>>(<Hit>[]),
        );
        expect(
          await repository.search(
            'tulips',
            scope: const SearchScope.notebook(id: 'b_1'),
            limit: 5,
          ),
          const Right<ApplicationError, List<Hit>>(<Hit>[]),
        );
      },
    );

    test('a notebook is made inside its parent', () async {
      const made = NotebookRow(
        id: 'b_2',
        name: 'Plans',
        parentId: 'b_1',
        count: 0,
        locked: false,
        depth: 1, hasChildren: false,
      );
      when(
        () => library.createNotebook(name: 'Plans', parentId: 'b_1'),
      ).thenAnswer((_) async => made);
      expect(
        await repository.createNotebook('Plans', parentId: 'b_1'),
        const Right<ApplicationError, NotebookRow>(made),
      );
    });

    test('a conflict carries the revision the library is at', () async {
      when(
        () => library.save(id: 'n_1', baseRev: '1-a', body: 'x'),
      ).thenAnswer(
        (_) =>
            Future<Saved>.error(const KansoError.conflict(currentRev: '2-b')),
      );
      final result = await repository.save(
        id: 'n_1',
        baseRev: '1-a',
        body: 'x',
      );
      expect(
        (result.swap().toOption().toNullable()! as ConflictError).currentRev,
        '2-b',
      );
    });

    test("the changes are the library's stream", () async {
      when(() => library.changes()).thenAnswer(
        (_) => Stream<List<String>>.value(const <String>['n_1']),
      );
      expect(await repository.changes().first, <String>['n_1']);
    });
  });

  group('RustSessionRepository', () {
    test('a wrong password is false, not an error', () async {
      final locks = _MockLocks();
      when(() => locks.unlock(password: 'nope')).thenAnswer((_) async => false);
      expect(
        await RustSessionRepository(locks).unlock('nope'),
        const Right<ApplicationError, bool>(false),
      );
    });

    test(
      'every lock call reaches the session with its own arguments',
      () async {
        final locks = _MockLocks();
        const saved = Saved(id: 'n_1', rev: '2-b');
        const status = LockStatus(
          hasLockPassword: true,
          unlocked: false,
          biometricsOn: true,
        );
        when(locks.lockForBackground).thenAnswer((_) async {});
        when(locks.lockStatus).thenAnswer((_) async => status);
        when(
          () => locks.unlockNote(id: 'n_1', password: 'pw'),
        ).thenAnswer((_) async => true);
        when(
          () => locks.unlockNotebook(bookId: 'b_1', password: 'pw'),
        ).thenAnswer((_) async => false);
        when(
          () => locks.lockNoteWithSession(id: 'n_1'),
        ).thenAnswer((_) async => saved);
        when(
          () => locks.removeNoteLock(id: 'n_1'),
        ).thenAnswer((_) async => saved);
        when(
          () => locks.lockNotebook(bookId: 'b_1', password: 'pw'),
        ).thenAnswer((_) async => 3);
        when(
          () => locks.removeNotebookLock(bookId: 'b_1'),
        ).thenAnswer((_) async {});
        when(
          () => locks.notebookIsOpen(bookId: 'b_1'),
        ).thenAnswer((_) async => true);
        when(
          () => locks.changePassword(old: 'a', new_: 'b'),
        ).thenAnswer((_) async => 2);
        when(locks.libraryId).thenAnswer((_) async => 'l_abc');
        when(locks.biometricsOn).thenAnswer((_) async => true);
        when(locks.beginBiometricEnrollment).thenAnswer((_) async => 11);
        when(locks.beginBiometricUnlock).thenAnswer((_) async => null);
        when(locks.turnOffBiometrics).thenAnswer((_) async {});

        final repository = RustSessionRepository(locks);
        Right<ApplicationError, T> right<T>(T value) =>
            Right<ApplicationError, T>(value);
        expect(await repository.lockForBackground(), right(unit));
        expect(await repository.lockStatus(), right(status));
        expect(await repository.unlockNote('n_1', 'pw'), right(true));
        expect(await repository.unlockNotebook('b_1', 'pw'), right(false));
        expect(await repository.lockNoteWithSession('n_1'), right(saved));
        expect(await repository.removeNoteLock('n_1'), right(saved));
        expect(await repository.lockNotebook('b_1', password: 'pw'), right(3));
        expect(await repository.removeNotebookLock('b_1'), right(unit));
        expect(await repository.notebookIsOpen('b_1'), right(true));
        expect(await repository.changePassword('a', 'b'), right(2));
        expect(await repository.libraryId(), right('l_abc'));
        expect(await repository.biometricsOn(), right(true));
        expect(await repository.beginBiometricEnrollment(), right(11));
        expect(await repository.beginBiometricUnlock(), right<int?>(null));
        expect(await repository.turnOffBiometrics(), right(unit));
      },
    );

    test("a locked notebook or a wrong password is the app's error", () async {
      final locks = _MockLocks();
      when(() => locks.removeNotebookLock(bookId: 'b_1')).thenAnswer(
        (_) => Future<void>.error(const KansoError.locked()),
      );
      when(() => locks.changePassword(old: 'x', new_: 'y')).thenAnswer(
        (_) => Future<int>.error(
          const KansoError.auth(message: "That isn't the lock password."),
        ),
      );
      final repository = RustSessionRepository(locks);
      expect(
        (await repository.removeNotebookLock(
          'b_1',
        )).swap().toOption().toNullable(),
        isA<LockedError>(),
      );
      expect(
        (await repository.changePassword(
          'x',
          'y',
        )).swap().toOption().toNullable(),
        isA<AuthError>(),
      );
    });

    test('a panic that got past the library is an internal error', () async {
      final locks = _MockLocks();
      when(locks.isUnlocked).thenAnswer(
        (_) => Future<bool>.error(StateError('x')),
      );
      final result = await RustSessionRepository(locks).isUnlocked();
      expect(result.swap().toOption().toNullable(), isA<UnexpectedError>());
    });
  });
}
