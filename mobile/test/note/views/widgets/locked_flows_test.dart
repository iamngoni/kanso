// Kanso — locked_flows_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/settings/views/widgets/settings_pane.dart';
import 'package:kanso_lock/kanso_lock.dart';

import '../../../helpers/browse_blocs.dart';
import '../../../helpers/fixture_device_lock_repository.dart';
import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/fixture_session_repository.dart';
import '../../../helpers/kanso_viewport.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/pump_browse_app.dart';

final String _locked =
    '/notes/note/${Uri.encodeComponent(MockIds.privateJournal)}';
final String _plain = '/notes/note/${Uri.encodeComponent(MockIds.syncDesign)}';

/// What the private journal says once it is open.
const String _journalText = 'A quiet week.';

/// The mock library whose private journal follows [session]: the closed note
/// while the notes are locked, and its text once they are open.
class _JournalLibrary extends FixtureLibraryRepository {
  _JournalLibrary(this.session);

  final FixtureSessionRepository session;

  @override
  Future<Either<ApplicationError, NoteView>> note(String id) {
    if (id != MockIds.privateJournal) {
      return super.note(id);
    }
    return Future<Either<ApplicationError, NoteView>>.value(
      Right<ApplicationError, NoteView>(
        session.unlocked
            ? mockNote(
                id: MockIds.privateJournal,
                title: 'Private journal',
                body: _journalText,
                open: true,
                notebookId: MockIds.journal,
              )
            : mockLockedNote,
      ),
    );
  }
}

/// The app on a phone, open at [at], over [session] and [device].
Future<BrowseApp> _pump(
  WidgetTester tester, {
  required String at,
  required FixtureSessionRepository session,
  FixtureDeviceLockRepository? device,
}) {
  const viewport = KansoViewport.phone;
  return atMockNow(
    () => pumpBrowseApp(
      tester,
      viewport.size,
      at: at,
      library: _JournalLibrary(session),
      session: session,
      device: device,
      devicePixelRatio: viewport.devicePixelRatio,
      safeArea: viewport.safeArea,
      tabletDevice: viewport.tablet,
    ),
  );
}

Future<void> _tap(WidgetTester tester, String text) async {
  await tester.tap(find.text(text).last);
  await tester.pumpAndSettle();
}

/// Types [text] into the last text field on screen, the top sheet's.
Future<void> _type(WidgetTester tester, String text, {int fromEnd = 0}) async {
  final fields = find.byType(EditableText);
  await tester.enterText(
    fields.at(fields.evaluate().length - 1 - fromEnd),
    text,
  );
  await tester.pump();
}

Future<void> _more(WidgetTester tester) async {
  await tester.tap(find.bySemanticsLabel('More').last);
  await tester.pumpAndSettle();
}

void main() {
  group('a closed locked note', () {
    testWidgets('shows its title and the card, never its text', (
      tester,
    ) async {
      await _pump(tester, at: _locked, session: FixtureSessionRepository());
      expect(find.text('Private journal'), findsWidgets);
      expect(find.text('This note is locked'), findsOneWidget);
      expect(find.textContaining(mockSecret), findsNothing);
      expect(find.text('Use password'), findsOneWidget);
      expect(find.text('Unlock with Face ID'), findsNothing);
      // A closed note has no lock action to offer.
      expect(find.bySemanticsLabel('More'), findsNothing);
    });

    testWidgets('opens with Face ID when biometrics are on', (tester) async {
      final session = FixtureSessionRepository(biometrics: true);
      // The core opens the notes and says so when the check ends well.
      final device = FixtureDeviceLockRepository()
        ..onUnlock = () {
          session.unlocked = true;
          session.events.add(const LockEvent.unlocked());
        };
      await _pump(tester, at: _locked, session: session, device: device);
      await _tap(tester, 'Unlock with Face ID');
      expect(session.calls, contains('beginBiometricUnlock'));
      expect(device.calls.single, startsWith('unlock:42:'));
      expect(find.text('This note is locked'), findsNothing);
      expect(find.textContaining(_journalText), findsOneWidget);
      // Face ID was on already: nothing is offered.
      expect(find.text('Use Face ID?'), findsNothing);
    });

    testWidgets('stays closed when Face ID is cancelled', (tester) async {
      final session = FixtureSessionRepository(biometrics: true);
      final device = FixtureDeviceLockRepository()
        ..outcome = BiometricOutcome.cancelled;
      await _pump(tester, at: _locked, session: session, device: device);
      await _tap(tester, 'Unlock with Face ID');
      expect(device.calls.single, startsWith('unlock:42:'));
      expect(session.unlocked, isFalse);
      expect(find.text('This note is locked'), findsOneWidget);
      expect(find.textContaining(_journalText), findsNothing);
    });

    testWidgets('offers the fingerprint on a phone that has one', (
      tester,
    ) async {
      await _pump(
        tester,
        at: _locked,
        session: FixtureSessionRepository(biometrics: true),
        device: FixtureDeviceLockRepository(kind: BiometricKind.fingerprint),
      );
      expect(find.textContaining('Unlock with'), findsOneWidget);
      expect(find.text('Unlock with Face ID'), findsNothing);
    });

    testWidgets('a wrong password says so and keeps the sheet', (
      tester,
    ) async {
      final session = FixtureSessionRepository();
      await _pump(tester, at: _locked, session: session);
      await _tap(tester, 'Use password');
      expect(find.text('Unlock note'), findsOneWidget);
      await _type(tester, 'not the password');
      await _tap(tester, 'Unlock');
      expect(find.text("That isn't the lock password."), findsOneWidget);
      expect(find.text('Unlock note'), findsOneWidget);
      expect(session.unlocked, isFalse);
      expect(find.textContaining(_journalText), findsNothing);
    });

    testWidgets('the right password opens it, then offers Face ID once', (
      tester,
    ) async {
      final session = FixtureSessionRepository();
      await _pump(tester, at: _locked, session: session);
      await _tap(tester, 'Use password');
      await _type(tester, session.password);
      await _tap(tester, 'Unlock');
      expect(session.unlocked, isTrue);
      expect(find.text('Unlock note'), findsNothing);
      expect(find.text('This note is locked'), findsNothing);
      expect(find.textContaining(_journalText), findsOneWidget);
      expect(find.text('Use Face ID?'), findsOneWidget);
      await _tap(tester, 'Not now');
      expect(find.text('Use Face ID?'), findsNothing);
      expect(session.biometrics, isFalse);
    });

    testWidgets('accepting the offer turns Face ID on', (tester) async {
      final session = FixtureSessionRepository();
      final device = FixtureDeviceLockRepository();
      await _pump(tester, at: _locked, session: session, device: device);
      await _tap(tester, 'Use password');
      await _type(tester, session.password);
      await _tap(tester, 'Unlock');
      await _tap(tester, 'Use Face ID');
      expect(session.calls, contains('beginBiometricEnrollment'));
      expect(device.calls, contains(startsWith('enroll:41:')));
      expect(find.text('Use Face ID?'), findsNothing);
    });

    testWidgets('locks again from the menu, and asks for the password', (
      tester,
    ) async {
      final session = FixtureSessionRepository();
      await _pump(tester, at: _locked, session: session);
      await _tap(tester, 'Use password');
      await _type(tester, session.password);
      await _tap(tester, 'Unlock');
      await _tap(tester, 'Not now');
      expect(find.textContaining(_journalText), findsOneWidget);
      await _more(tester);
      await _tap(tester, 'Lock notes now');
      expect(session.calls, contains('lock'));
      expect(session.unlocked, isFalse);
      expect(find.textContaining(_journalText), findsNothing);
      expect(find.text('This note is locked'), findsOneWidget);
      // Open again by the same way: the password.
      await _tap(tester, 'Use password');
      await _type(tester, session.password);
      await _tap(tester, 'Unlock');
      expect(session.unlocked, isTrue);
      expect(find.textContaining(_journalText), findsOneWidget);
    });
  });

  group('the note menu', () {
    testWidgets('locks a plain note, making the lock password first', (
      tester,
    ) async {
      final session = FixtureSessionRepository(hasLockPassword: false);
      await _pump(tester, at: _plain, session: session);
      await _more(tester);
      expect(find.text('Lock notes now'), findsNothing);
      await _tap(tester, 'Lock note…');
      expect(find.text('Set a lock password'), findsOneWidget);
      await _type(tester, 'short', fromEnd: 1);
      await _type(tester, 'short');
      await _tap(tester, 'Lock');
      expect(
        find.text('The lock password needs at least 8 characters.'),
        findsOneWidget,
      );
      await _type(tester, 'a long password', fromEnd: 1);
      await _type(tester, 'a different one');
      await _tap(tester, 'Lock');
      expect(find.text("The passwords don't match."), findsOneWidget);
      await _type(tester, 'a long password');
      await _tap(tester, 'Lock');
      expect(session.calls, contains('lockNote:${MockIds.syncDesign}'));
      expect(session.password, 'a long password');
      expect(find.text('Set a lock password'), findsNothing);
    });

    testWidgets('with the notes closed, locking asks for the password', (
      tester,
    ) async {
      final session = FixtureSessionRepository();
      await _pump(tester, at: _plain, session: session);
      await _more(tester);
      await _tap(tester, 'Lock note…');
      expect(find.text('Lock note'), findsOneWidget);
      await _type(tester, 'nope nope nope');
      await _tap(tester, 'Lock');
      expect(find.text("That isn't the lock password."), findsOneWidget);
      await _type(tester, session.password);
      await _tap(tester, 'Lock');
      expect(session.calls, contains('lockNote:${MockIds.syncDesign}'));
    });

    testWidgets('with the notes open, locking only confirms', (tester) async {
      final session = FixtureSessionRepository(unlocked: true);
      await _pump(tester, at: _plain, session: session);
      await _more(tester);
      await _tap(tester, 'Lock note…');
      expect(find.text('Lock this note?'), findsOneWidget);
      expect(find.byType(EditableText), findsNothing);
      await _tap(tester, 'Lock');
      expect(
        session.calls,
        contains('lockNoteWithSession:${MockIds.syncDesign}'),
      );
    });

    testWidgets('removes the lock of an open locked note', (tester) async {
      final session = FixtureSessionRepository(unlocked: true);
      await _pump(tester, at: _locked, session: session);
      await _more(tester);
      expect(find.text('Lock note…'), findsNothing);
      await _tap(tester, 'Remove lock…');
      expect(find.text('Remove the lock?'), findsOneWidget);
      await _tap(tester, 'Remove Lock');
      expect(
        session.calls,
        contains('removeNoteLock:${MockIds.privateJournal}'),
      );
      expect(find.text('Remove the lock?'), findsNothing);
    });

    testWidgets('locks the notes now', (tester) async {
      final session = FixtureSessionRepository(unlocked: true);
      await _pump(tester, at: _locked, session: session);
      await _more(tester);
      expect(find.textContaining(_journalText), findsOneWidget);
      await _tap(tester, 'Lock notes now');
      expect(session.calls, contains('lock'));
      expect(session.unlocked, isFalse);
      expect(find.text('This note is locked'), findsOneWidget);
      expect(find.textContaining(_journalText), findsNothing);
    });
  });

  group('Settings › Locked notes', () {
    Future<void> settings(WidgetTester tester, BrowseApp app) async {
      unawaited(app.router.push<void>('/settings'));
      await tester.pumpAndSettle();
      // The page is a lazily built list, so the row has to be scrolled to
      // before it exists to be found.
      await tester.scrollUntilVisible(
        find.text('Change lock password…'),
        200,
        scrollable: find.descendant(
          of: find.byType(SettingsPane),
          matching: find.byType(Scrollable),
        ),
      );
      await tester.pumpAndSettle();
    }

    testWidgets('is not there without a lock password', (tester) async {
      final app = await _pump(
        tester,
        at: '/notes',
        session: FixtureSessionRepository(hasLockPassword: false),
      );
      unawaited(app.router.push<void>('/settings'));
      await tester.pumpAndSettle();
      expect(find.text('Locked notes'), findsNothing);
      expect(find.text('Change lock password…'), findsNothing);
    });

    testWidgets('changes the lock password, saying when the old is wrong', (
      tester,
    ) async {
      final session = FixtureSessionRepository(unlocked: true);
      final app = await _pump(tester, at: '/notes', session: session);
      await settings(tester, app);
      await _tap(tester, 'Change lock password…');
      await _type(tester, 'not the password', fromEnd: 2);
      await _type(tester, 'a new password', fromEnd: 1);
      await _type(tester, 'a new password');
      await _tap(tester, 'Change');
      expect(find.text("That isn't the lock password."), findsOneWidget);
      await _type(tester, session.password, fromEnd: 2);
      await _tap(tester, 'Change');
      expect(session.password, 'a new password');
      expect(find.text('Change lock password'), findsNothing);
    });

    testWidgets('says how many items kept the old password', (tester) async {
      final session = FixtureSessionRepository(unlocked: true)..kept = 2;
      final app = await _pump(tester, at: '/notes', session: session);
      await settings(tester, app);
      await _tap(tester, 'Change lock password…');
      await _type(tester, session.password, fromEnd: 2);
      await _type(tester, 'a new password', fromEnd: 1);
      await _type(tester, 'a new password');
      await _tap(tester, 'Change');
      expect(
        find.text(
          "2 locked notes or notebooks didn't open with the current "
          'password and kept it.',
        ),
        findsOneWidget,
      );
      await _tap(tester, 'Done');
      expect(find.text('Change lock password'), findsNothing);
    });

    testWidgets('locks the notes now', (tester) async {
      final session = FixtureSessionRepository(unlocked: true);
      final app = await _pump(tester, at: '/notes', session: session);
      await settings(tester, app);
      await _tap(tester, 'Lock notes now');
      expect(session.unlocked, isFalse);
    });
  });
}
