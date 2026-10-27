// Kanso — sync_suite.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/sync.dart';
import 'package:kanso/core/bridge/api/types.dart';

import 'open_library.dart';

/// A server that is not there: nothing listens on port 1, so a request fails
/// at once, with no network and no name lookup.
const String unreachableServer = 'http://127.0.0.1:1';

/// Sync and sign-in, through the bridge: every call of `KansoSync`, against
/// no server.
void syncSuite(OpenLibrary open) {
  group('KansoSync', () {
    test('a new library is signed out', () async {
      final sync = (await open()).syncing();
      final status = await sync.status();
      expect(status.phase, SyncPhase.signedOut);
      expect(status.username, isNull);
      expect(status.problem, isNull);
    });

    test('the default server is an https address', () async {
      expect(await KansoSync.defaultServer(), startsWith('https://'));
    });

    test('the status stream starts with the status now', () async {
      final sync = (await open()).syncing();
      final first = await sync.statusStream().first.timeout(
        const Duration(seconds: 10),
      );
      expect(first.phase, SyncPhase.signedOut);
    });

    test('a server that is not there is a network error', () async {
      final sync = (await open()).syncing();
      await expectLater(
        sync.checkServer(url: unreachableServer),
        throwsA(isA<KansoError_Network>()),
      );
      await expectLater(
        sync.signInPassword(
          server: unreachableServer,
          username: 'ada',
          password: 'correct horse battery',
          deviceName: 'Test phone',
        ),
        throwsA(isA<KansoError_Network>()),
      );
      expect((await sync.status()).phase, SyncPhase.signedOut);
    });

    test('the app can say it is in front or behind', () async {
      final sync = (await open()).syncing();
      await sync.setForeground(foreground: true);
      await sync.setForeground(foreground: false);
      expect((await sync.status()).phase, SyncPhase.signedOut);
    });

    test('the settings start on, every five minutes, take only the desktop’s '
        'choices and are kept', () async {
      final sync = (await open()).syncing();
      final first = await sync.settings();
      expect(first.enabled, isTrue);
      expect(first.intervalSecs, 300);
      final options = KansoSync.intervalOptions();
      expect(options.map((option) => option.minutes), <int>[
        1,
        5,
        15,
        30,
        60,
        240,
      ]);
      expect(options.map((option) => option.label), contains('1 hour'));
      expect(KansoSync.intervalLabel(minutes: 240), '4 hours');
      expect(KansoSync.intervalPhrase(minutes: 15), 'every 15 minutes');
      final changed = await sync.setSettings(
        enabled: false,
        intervalSecs: 3600,
      );
      expect(changed.enabled, isFalse);
      expect(changed.intervalSecs, 3600);
      expect(await sync.settings(), changed);
      await expectLater(
        sync.setSettings(enabled: true, intervalSecs: 7),
        throwsA(isA<KansoError_Invalid>()),
      );
      expect(await sync.settings(), changed);
    });

    test('the status line says what the desktop says', () async {
      final sync = (await open()).syncing();
      final status = await sync.status();
      final line = KansoSync.statusLine(
        status: status,
        settings: await sync.settings(),
        when: 'just now',
        resume: 'at 10:00',
      );
      expect(line.label, 'Sync is off');
      expect(line.tone, SyncTone.quiet);
      expect(line.detail, contains('only on this device'));
    });

    test('syncing while signed out says so, and signing out is fine', () async {
      final sync = (await open()).syncing();
      await expectLater(
        sync.syncNow(),
        throwsA(isA<KansoError_SignedOut>()),
      );
      await sync.signOut();
      expect((await sync.status()).phase, SyncPhase.signedOut);
    });
  });
}
