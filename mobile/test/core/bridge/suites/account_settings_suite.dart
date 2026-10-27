// Kanso — account_settings_suite.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';

import 'open_library.dart';

/// The account's page, through the bridge: every call of
/// `KansoAccountSettings`, signed out.
void accountSettingsSuite(OpenLibrary open) {
  group('KansoAccountSettings', () {
    test('signed out, the account has no page', () async {
      final settings = (await open()).accountSettings();
      await expectLater(
        settings.details(),
        throwsA(isA<KansoError_SignedOut>()),
      );
      await expectLater(
        settings.revokeDevice(deviceId: 'device'),
        throwsA(isA<KansoError_SignedOut>()),
      );
      await expectLater(
        settings.pendingRecoveryKey(),
        throwsA(isA<KansoError_SignedOut>()),
      );
    });

    test(
      'signed out, the password and recovery key stay as they are',
      () async {
        final settings = (await open()).accountSettings();
        await expectLater(
          settings.changePassword(current: 'old one', newPassword: 'new one'),
          throwsA(isA<KansoError_SignedOut>()),
        );
        await expectLater(
          settings.newRecoveryKey(currentPassword: 'old one'),
          throwsA(isA<KansoError_SignedOut>()),
        );
        await expectLater(
          settings.deleteAccount(password: 'old one', typedUsername: 'ada'),
          throwsA(isA<KansoError_SignedOut>()),
        );
      },
    );

    test('signed out, passkeys and providers are not touched', () async {
      final settings = (await open()).accountSettings();
      await expectLater(
        settings.addPasskey(currentPassword: 'old one'),
        throwsA(isA<KansoError_SignedOut>()),
      );
      await expectLater(
        settings.removePasskey(id: 'passkey', currentPassword: 'old one'),
        throwsA(isA<KansoError_Invalid>()),
      );
      await expectLater(
        settings.unlinkProvider(
          provider: ProviderKind.github,
          currentPassword: 'old one',
        ),
        throwsA(isA<KansoError_SignedOut>()),
      );
      final first = await settings
          .linkProvider(
            provider: ProviderKind.github,
            currentPassword: 'old one',
          )
          .first
          .timeout(const Duration(seconds: 10));
      expect(first, isA<ProviderEvent_Failed>());
    });

    test('signing out when signed out is fine', () async {
      final library = await open();
      await library.accountSettings().signOut();
      expect(
        await library.account().state(),
        const AccountState.signedOut(),
      );
    });
  });
}
