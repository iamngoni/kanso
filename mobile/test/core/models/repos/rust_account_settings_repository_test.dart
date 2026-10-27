// Kanso — rust_account_settings_repository_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/account_settings.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/impl/rust_account_settings_repository.dart';
import 'package:mocktail/mocktail.dart';

class _MockSettings extends Mock implements KansoAccountSettings {}

const AccountDetails _details = AccountDetails(
  username: 'ana',
  server: 'https://sync.example.invalid',
  devices: <DeviceRow>[],
  hasPassword: true,
  providers: <LinkedProviderRow>[],
  passkeys: <PasskeyRow>[],
  recoveryKeyUnsaved: false,
);

void main() {
  late _MockSettings settings;
  late RustAccountSettingsRepository repository;

  setUp(() {
    settings = _MockSettings();
    repository = RustAccountSettingsRepository(settings);
  });

  group('RustAccountSettingsRepository', () {
    test('the details, and a device signed out', () async {
      when(settings.details).thenAnswer((_) async => _details);
      when(
        () => settings.revokeDevice(deviceId: 'd2'),
      ).thenAnswer((_) async => _details);
      expect(
        await repository.details(),
        const Right<ApplicationError, AccountDetails>(_details),
      );
      expect(
        await repository.revokeDevice('d2'),
        const Right<ApplicationError, AccountDetails>(_details),
      );
    });

    test('a session the server ended is the signed-out error', () async {
      when(settings.details).thenAnswer(
        (_) => Future<AccountDetails>.error(const KansoError.signedOut()),
      );
      final result = await repository.details();
      expect(result.swap().toOption().toNullable(), isA<SignedOutError>());
    });

    test('signing out and deleting return unit', () async {
      when(settings.signOut).thenAnswer((_) async {});
      when(
        () => settings.deleteAccount(password: 'pw', typedUsername: 'ana'),
      ).thenAnswer((_) async {});
      expect(
        await repository.signOut(),
        const Right<ApplicationError, Unit>(unit),
      );
      expect(
        await repository.deleteAccount(password: 'pw', typedUsername: 'ana'),
        const Right<ApplicationError, Unit>(unit),
      );
    });

    test('a new password and a new recovery key give the key', () async {
      const key = RecoveryKeyText(key: 'ABCD-EFGH');
      when(
        () => settings.changePassword(current: 'old', newPassword: 'new'),
      ).thenAnswer((_) async => key);
      when(
        () => settings.newRecoveryKey(currentPassword: 'pw'),
      ).thenAnswer((_) async => key);
      expect(
        await repository.changePassword(current: 'old', newPassword: 'new'),
        const Right<ApplicationError, RecoveryKeyText>(key),
      );
      expect(
        await repository.newRecoveryKey('pw'),
        const Right<ApplicationError, RecoveryKeyText>(key),
      );
    });

    test('passkeys are added and removed, and providers unlinked', () async {
      when(
        () => settings.addPasskey(currentPassword: 'pw'),
      ).thenAnswer((_) async => _details);
      when(
        () => settings.removePasskey(id: 'p1', currentPassword: 'pw'),
      ).thenAnswer((_) async => _details);
      when(
        () => settings.unlinkProvider(
          provider: ProviderKind.google,
          currentPassword: 'pw',
        ),
      ).thenAnswer((_) async => _details);
      for (final result in <Either<ApplicationError, AccountDetails>>[
        await repository.addPasskey('pw'),
        await repository.removePasskey(id: 'p1', currentPassword: 'pw'),
        await repository.unlinkProvider(
          provider: ProviderKind.google,
          currentPassword: 'pw',
        ),
      ]) {
        expect(result, const Right<ApplicationError, AccountDetails>(_details));
      }
    });

    test('linking a provider is the library’s stream of steps', () async {
      when(
        () => settings.linkProvider(
          provider: ProviderKind.github,
          currentPassword: 'pw',
        ),
      ).thenAnswer(
        (_) => Stream<ProviderEvent>.value(const ProviderEvent.cancelled()),
      );
      expect(
        await repository
            .linkProvider(provider: ProviderKind.github, currentPassword: 'pw')
            .toList(),
        <ProviderEvent>[const ProviderEvent.cancelled()],
      );
    });

    test('the recovery key kept is read from the settings’ handle', () async {
      when(settings.pendingRecoveryKey).thenAnswer((_) async => null);
      expect(
        await repository.pendingRecoveryKey(),
        const Right<ApplicationError, RecoveryKeyText?>(null),
      );
    });
  });
}
