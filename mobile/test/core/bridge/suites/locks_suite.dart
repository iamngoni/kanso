// Kanso — locks_suite.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';

import '../../../helpers/biometric_answer.dart';
import 'open_library.dart';

/// The lock session, through the bridge: every call of `KansoLocks`.
///
/// [answer] stands in for the native plugin, handing a biometric check's
/// answer to the library's C entry; without it the biometric calls that need
/// one are left out.
void locksSuite(OpenLibrary open, {BiometricAnswer? answer}) {
  group('KansoLocks', () {
    test(
      'a locked note closes to its title, and opens with the password',
      () async {
        final library = await open();
        final locks = library.locks();
        final id = await makeNote(library, 'Plan', 'the secret plan');

        // Nothing is locked yet, so there is no password to try.
        await expectLater(
          locks.unlock(password: lockPassword),
          throwsA(isA<KansoError_Invalid>()),
        );
        expect(await locks.isUnlocked(), isFalse);

        final events = <LockEvent>[];
        final subscription = locks.lockEvents().listen(events.add);
        // Cancelling waits for the library's next event: don't wait for it.
        addTearDown(() => unawaited(subscription.cancel()));
        await Future<void>.delayed(const Duration(milliseconds: 200));

        // Locking the first note sets the password, and leaves it open.
        final saved = await locks.lockNote(id: id, password: lockPassword);
        expect(saved.id, id);
        expect(await locks.isUnlocked(), isTrue);
        expect(await locks.isNoteUnlocked(id: id), isTrue);
        final open_ = await library.note(id: id);
        expect(open_.lock, isA<LockState_Open>());
        expect(open_.body, 'the secret plan');

        await locks.lock();
        expect(await locks.isUnlocked(), isFalse);
        final closed = await library.note(id: id);
        expect(closed.lock, isA<LockState_Closed>());
        expect(closed.body, isNull);
        // The title shows; the text never does.
        expect(closed.title, 'Plan');
        final row = (await library.list(query: everyNote)).rows.single;
        expect(row.locked, isTrue);
        expect(row.title, 'Plan');
        expect(row.snippet, isEmpty);
        expect(await library.properties(id: id), isEmpty);
        expect(
          await library.search(
            text: 'secret',
            scope: const SearchScope.all(),
            limit: 10,
          ),
          isEmpty,
        );
        await expectLater(
          library.save(id: id, baseRev: closed.rev, body: 'overwritten'),
          throwsA(isA<KansoError_Locked>()),
        );
        await expectLater(
          library.renderer().previewHtml(id: id, palette: _palette),
          throwsA(isA<KansoError_Locked>()),
        );

        expect(await locks.unlock(password: 'not the password'), isFalse);
        expect(await locks.unlock(password: lockPassword), isTrue);
        expect((await library.note(id: id)).body, 'the secret plan');
        await expectLater(
          locks.isNoteUnlocked(id: 'note:nope'),
          throwsA(isA<KansoError_NotFound>()),
        );

        await Future<void>.delayed(const Duration(milliseconds: 200));
        expect(events, <LockEvent>[
          const LockEvent.unlocked(),
          const LockEvent.locked(reason: LockReason.manual),
          const LockEvent.unlocked(),
        ]);
      },
      timeout: const Timeout(Duration(minutes: 3)),
    );

    test(
      'locking with the open session, removing locks and the status',
      () async {
        final library = await open();
        final locks = library.locks();
        final first = await makeNote(library, 'One', 'ONE-TEXT');
        final second = await makeNote(library, 'Two', 'TWO-TEXT');
        final status = await locks.lockStatus();
        expect(status.hasLockPassword, isFalse);
        expect(status.unlocked, isFalse);
        expect(status.biometricsOn, isFalse);
        expect(await locks.libraryId(), startsWith('l_'));

        // No key is held yet: nothing to lock with.
        await expectLater(
          locks.lockNoteWithSession(id: first),
          throwsA(isA<KansoError_Locked>()),
        );
        await locks.lockNote(id: first, password: lockPassword);
        expect((await locks.lockStatus()).hasLockPassword, isTrue);
        expect((await locks.lockStatus()).unlocked, isTrue);
        final saved = await locks.lockNoteWithSession(id: second);
        expect(saved.id, second);

        // Leaving the app closes the notes.
        final events = <LockEvent>[];
        final subscription = locks.lockEvents().listen(events.add);
        addTearDown(() => unawaited(subscription.cancel()));
        await Future<void>.delayed(const Duration(milliseconds: 200));
        await locks.lockForBackground();
        expect((await locks.lockStatus()).unlocked, isFalse);
        expect((await library.note(id: second)).body, isNull);
        await expectLater(
          locks.removeNoteLock(id: second),
          throwsA(isA<KansoError_Locked>()),
        );
        expect(
          await locks.unlockNote(id: second, password: 'wrong one!'),
          false,
        );
        expect(
          await locks.unlockNote(id: second, password: lockPassword),
          true,
        );
        final removed = await locks.removeNoteLock(id: second);
        expect(removed.id, second);
        final plain = await library.note(id: second);
        expect(plain.lock, isA<LockState_Plain>());
        expect(plain.body, 'TWO-TEXT');
        await Future<void>.delayed(const Duration(milliseconds: 200));
        expect(
          events,
          contains(const LockEvent.locked(reason: LockReason.background)),
        );
      },
      timeout: const Timeout(Duration(minutes: 3)),
    );

    test(
      'a notebook is locked, opened and unlocked, and the password changed',
      () async {
        final library = await open();
        final locks = library.locks();
        final vault = (await library.createNotebook(name: 'Vault')).id;
        final inside = (await library.create(
          note: NewNote(notebookId: vault, title: 'In', body: 'IN-TEXT'),
        )).id;
        expect(await locks.notebookIsOpen(bookId: vault), isTrue);
        await expectLater(
          locks.lockNotebook(bookId: vault),
          throwsA(isA<KansoError_Locked>()),
        );
        expect(
          await locks.lockNotebook(bookId: vault, password: lockPassword),
          1,
        );
        expect(await locks.notebookIsOpen(bookId: vault), isTrue);
        await locks.lock();
        expect(await locks.notebookIsOpen(bookId: vault), isFalse);
        expect((await library.note(id: inside)).body, isNull);
        await expectLater(
          locks.removeNotebookLock(bookId: vault),
          throwsA(isA<KansoError_Locked>()),
        );
        expect(
          await locks.unlockNotebook(bookId: vault, password: 'wrong one!'),
          isFalse,
        );
        expect(
          await locks.unlockNotebook(bookId: vault, password: lockPassword),
          isTrue,
        );
        expect((await library.note(id: inside)).body, 'IN-TEXT');

        // A new password: the old one opens nothing afterwards.
        await expectLater(
          locks.changePassword(old: 'not the password', new_: 'a new password'),
          throwsA(isA<KansoError_Auth>()),
        );
        expect(
          await locks.changePassword(old: lockPassword, new_: 'a new password'),
          0,
        );
        await locks.lock();
        expect(await locks.unlock(password: lockPassword), isFalse);
        expect(await locks.unlock(password: 'a new password'), isTrue);

        await locks.removeNotebookLock(bookId: vault);
        final after = (await library.create(
          note: NewNote(notebookId: vault, title: 'After', body: 'AFTER'),
        )).id;
        expect((await library.note(id: after)).lock, isA<LockState_Plain>());
        // Its notes keep their own locks.
        await locks.lock();
        expect((await library.note(id: inside)).body, isNull);
      },
      timeout: const Timeout(Duration(minutes: 5)),
    );

    test(
      'biometrics keep the keys, open them after a check, and forget them',
      () async {
        final library = await open();
        final locks = library.locks();
        final id = await makeNote(library, 'Plan', 'the secret plan');
        await expectLater(
          locks.beginBiometricEnrollment(),
          throwsA(isA<KansoError_Locked>()),
        );
        expect(await locks.biometricsOn(), isFalse);
        expect(await locks.beginBiometricUnlock(), isNull);
        await locks.turnOffBiometrics();

        await locks.lockNote(id: id, password: lockPassword);
        final ticket = await locks.beginBiometricEnrollment();
        expect(ticket, isNonZero);
        if (answer == null) {
          return;
        }
        final secret = Uint8List.fromList(List<int>.filled(32, 7));
        // An unknown ticket, a failed check, then the enrolment, once.
        expect(answer(ticket + 1, 0, secret), 0);
        expect(answer(ticket, 4), 1);
        expect(await locks.biometricsOn(), isFalse);
        final next = await locks.beginBiometricEnrollment();
        expect(answer(next, 0, secret), 1);
        expect(answer(next, 0, secret), 0);
        expect(await locks.biometricsOn(), isTrue);
        expect((await locks.lockStatus()).biometricsOn, isTrue);

        await locks.lock();
        final unlock = (await locks.beginBiometricUnlock())!;
        expect(answer(unlock, 0, secret), 1);
        expect(await locks.isUnlocked(), isTrue);
        expect((await library.note(id: id)).body, 'the secret plan');

        // A secret that isn't the platform's forgets the keys.
        await locks.lock();
        final wrong = (await locks.beginBiometricUnlock())!;
        expect(answer(wrong, 0, Uint8List(32)), -1);
        expect(await locks.isUnlocked(), isFalse);
        expect(await locks.biometricsOn(), isFalse);
        expect(await locks.beginBiometricUnlock(), isNull);

        // Invalidated biometrics forget them too, and so does turning off.
        await locks.unlock(password: lockPassword);
        expect(answer(await locks.beginBiometricEnrollment(), 0, secret), 1);
        await locks.lock();
        expect(answer((await locks.beginBiometricUnlock())!, 2), 1);
        expect(await locks.biometricsOn(), isFalse);
        await locks.unlock(password: lockPassword);
        expect(answer(await locks.beginBiometricEnrollment(), 0, secret), 1);
        await locks.turnOffBiometrics();
        expect(await locks.biometricsOn(), isFalse);
      },
      timeout: const Timeout(Duration(minutes: 5)),
    );
  });
}

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
