// Kanso — account_details_bloc_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/account/models/data/account_notice.dart';
import 'package:kanso/account/models/data/account_outcome.dart';
import 'package:kanso/account/models/data/account_task.dart';
import 'package:kanso/account/models/data/form_problem.dart';
import 'package:kanso/account/state/account_details/account_details_bloc.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/fixture_account_repository.dart';
import '../../../helpers/mock_repositories.dart';

const String _password = 'correct horse battery';

const AccountDetails _details = AccountDetails(
  username: 'ana',
  server: 'https://sync.example.invalid',
  devices: <DeviceRow>[
    DeviceRow(id: 'd1', title: 'iPhone', thisDevice: true, lastSeenMs: 1),
    DeviceRow(id: 'd2', title: 'Ana’s Mac', thisDevice: false, lastSeenMs: 2),
  ],
  storageLine: '850 items · 3.1 MB of 100 MB',
  hasPassword: true,
  providers: <LinkedProviderRow>[
    LinkedProviderRow(provider: ProviderKind.github, linked: false),
  ],
  passkeys: <PasskeyRow>[
    PasskeyRow(id: 'p1', label: 'iCloud Keychain', addedMs: 1, canUnlock: true),
  ],
  recoveryKeyUnsaved: false,
);

AccountDetails _withPasskeys(List<PasskeyRow> passkeys) => AccountDetails(
  username: _details.username,
  server: _details.server,
  devices: _details.devices,
  storageLine: _details.storageLine,
  hasPassword: _details.hasPassword,
  providers: _details.providers,
  passkeys: passkeys,
  recoveryKeyUnsaved: false,
);

void main() {
  late MockAccountSettingsRepository settings;
  late MockAccountRepository account;
  late MockAuthSessionRepository session;

  setUp(() {
    settings = MockAccountSettingsRepository();
    account = MockAccountRepository();
    session = MockAuthSessionRepository();
    when(settings.details).thenAnswer(
      (_) async => const Right<ApplicationError, AccountDetails>(_details),
    );
    when(session.close).thenAnswer(
      (_) async => const Right<ApplicationError, Unit>(unit),
    );
    when(
      () => session.open(any(), callbackScheme: any(named: 'callbackScheme')),
    ).thenAnswer((_) async => const Right<ApplicationError, String?>(null));
    when(() => account.providerPageEnded(any())).thenAnswer(
      (_) async => const Right<ApplicationError, Unit>(unit),
    );
    when(account.cancelProvider).thenAnswer(
      (_) async => const Right<ApplicationError, Unit>(unit),
    );
  });

  Future<AccountDetailsBloc> started() async {
    final bloc = AccountDetailsBloc(settings, account, session)
      ..add(const AccountDetailsEvent.started());
    await bloc.stream.firstWhere((state) => state.status == LoadStatus.ready);
    return bloc;
  }

  group('AccountDetailsBloc, reading the account', () {
    test(
      'reads the details, and tries again when they could not be read',
      () async {
        var calls = 0;
        when(settings.details).thenAnswer((_) async {
          calls++;
          return calls == 1
              ? const Left<ApplicationError, AccountDetails>(
                  ApplicationError.network(message: 'Can’t reach the server.'),
                )
              : const Right<ApplicationError, AccountDetails>(_details);
        });
        final bloc = AccountDetailsBloc(settings, account, session)
          ..add(const AccountDetailsEvent.started());
        await bloc.stream.firstWhere(
          (state) => state.status == LoadStatus.failed,
        );
        expect(bloc.state.loadError, isA<NetworkError>());

        bloc.add(const AccountDetailsEvent.started());
        await bloc.stream.firstWhere(
          (state) => state.status == LoadStatus.ready,
        );
        expect(bloc.state.details, _details);
        expect(bloc.state.loadError, isNull);
        await bloc.close();
      },
    );
  });

  group('AccountDetailsBloc, devices', () {
    test(
      'signing another device out gives the page again, with a notice',
      () async {
        when(() => settings.revokeDevice('d2')).thenAnswer(
          (_) async => Right<ApplicationError, AccountDetails>(
            AccountDetails(
              username: _details.username,
              server: _details.server,
              devices: <DeviceRow>[_details.devices.first],
              storageLine: _details.storageLine,
              hasPassword: true,
              providers: _details.providers,
              passkeys: _details.passkeys,
              recoveryKeyUnsaved: false,
            ),
          ),
        );
        final bloc = await started();
        bloc.add(const AccountDetailsEvent.deviceRevoked('d2'));
        await bloc.stream.firstWhere((state) => state.notice != null);
        expect(
          bloc.state.notice,
          const AccountNotice.deviceSignedOut(name: 'Ana’s Mac'),
        );
        expect(bloc.state.details?.devices, hasLength(1));
        expect(bloc.state.busy, isNull);
        await bloc.close();
      },
    );

    test('a refusal is the error and the page stays as it was', () async {
      when(() => settings.revokeDevice('d2')).thenAnswer(
        (_) async => const Left<ApplicationError, AccountDetails>(
          ApplicationError.network(message: 'Can’t reach the server.'),
        ),
      );
      final bloc = await started();
      bloc.add(const AccountDetailsEvent.deviceRevoked('d2'));
      await bloc.stream.firstWhere((state) => state.error != null);
      expect(bloc.state.details, _details);
      expect(bloc.state.busy, isNull);
      expect(bloc.state.notice, isNull);
      await bloc.close();
    });

    test('a change is busy while it goes through', () async {
      final gate = Completer<void>();
      when(() => settings.revokeDevice('d2')).thenAnswer((_) async {
        await gate.future;
        return const Right<ApplicationError, AccountDetails>(_details);
      });
      final bloc = await started();
      bloc.add(const AccountDetailsEvent.deviceRevoked('d2'));
      await bloc.stream.firstWhere(
        (state) => state.busy == AccountTask.revokeDevice,
      );
      // Another change asked meanwhile is dropped.
      bloc.add(const AccountDetailsEvent.deviceRevoked('d2'));
      gate.complete();
      await bloc.stream.firstWhere((state) => state.busy == null);
      verify(() => settings.revokeDevice('d2')).called(1);
      await bloc.close();
    });
  });

  group('AccountDetailsBloc, signing out and deleting', () {
    test('signing out ends the page', () async {
      when(settings.signOut).thenAnswer(
        (_) async => const Right<ApplicationError, Unit>(unit),
      );
      final bloc = await started();
      bloc.add(const AccountDetailsEvent.signedOut());
      await bloc.stream.firstWhere((state) => state.outcome != null);
      expect(bloc.state.outcome, AccountOutcome.signedOut);
      await bloc.close();
    });

    test('a sign-out the secret store refused is the error', () async {
      when(settings.signOut).thenAnswer(
        (_) async => const Left<ApplicationError, Unit>(
          ApplicationError.auth(message: 'The secret store refused.'),
        ),
      );
      final bloc = await started();
      bloc.add(const AccountDetailsEvent.signedOut());
      await bloc.stream.firstWhere((state) => state.error != null);
      expect(bloc.state.outcome, isNull);
      await bloc.close();
    });

    test('deleting needs the name typed as the account’s', () async {
      final bloc = await started();
      bloc.add(
        const AccountDetailsEvent.accountDeleted(
          password: _password,
          typedUsername: 'someone',
        ),
      );
      await pumpEventQueue();
      expect(bloc.state.problem, FormProblem.usernameInvalid);
      verifyNever(
        () => settings.deleteAccount(
          password: any(named: 'password'),
          typedUsername: any(named: 'typedUsername'),
        ),
      );
      await bloc.close();
    });

    test('deleting with the name and the password ends the page', () async {
      when(
        () => settings.deleteAccount(password: _password, typedUsername: 'ana'),
      ).thenAnswer((_) async => const Right<ApplicationError, Unit>(unit));
      final bloc = await started();
      bloc.add(
        const AccountDetailsEvent.accountDeleted(
          password: _password,
          typedUsername: ' ana ',
        ),
      );
      await bloc.stream.firstWhere((state) => state.outcome != null);
      expect(bloc.state.outcome, AccountOutcome.signedOut);
      await bloc.close();
    });

    test('a wrong password for deleting is the error', () async {
      when(
        () => settings.deleteAccount(password: _password, typedUsername: 'ana'),
      ).thenAnswer(
        (_) async => const Left<ApplicationError, Unit>(
          ApplicationError.auth(message: 'That isn’t your current password.'),
        ),
      );
      final bloc = await started();
      bloc.add(
        const AccountDetailsEvent.accountDeleted(
          password: _password,
          typedUsername: 'ana',
        ),
      );
      await bloc.stream.firstWhere((state) => state.error != null);
      expect(bloc.state.outcome, isNull);
      await bloc.close();
    });
  });

  group('AccountDetailsBloc, the password and the recovery key', () {
    test('a new password is checked before anything is sent', () async {
      final bloc = await started();
      bloc.add(
        const AccountDetailsEvent.passwordChanged(
          current: _password,
          newPassword: 'short',
          repeat: 'short',
        ),
      );
      await pumpEventQueue();
      expect(bloc.state.problem, FormProblem.passwordTooShort);

      bloc.add(
        const AccountDetailsEvent.passwordChanged(
          current: _password,
          newPassword: 'another long password',
          repeat: 'a different one entirely',
        ),
      );
      await pumpEventQueue();
      expect(bloc.state.problem, FormProblem.passwordsDiffer);
      verifyNever(
        () => settings.changePassword(
          current: any(named: 'current'),
          newPassword: any(named: 'newPassword'),
        ),
      );
      await bloc.close();
    });

    test('a changed password gives the new recovery key to show', () async {
      when(
        () => settings.changePassword(
          current: _password,
          newPassword: 'another long password',
        ),
      ).thenAnswer(
        (_) async =>
            const Right<ApplicationError, RecoveryKeyText>(sampleRecoveryKey),
      );
      final bloc = await started();
      bloc.add(
        const AccountDetailsEvent.passwordChanged(
          current: _password,
          newPassword: 'another long password',
          repeat: 'another long password',
        ),
      );
      await bloc.stream.firstWhere((state) => state.newKey != null);
      expect(bloc.state.newKey, sampleRecoveryKey);
      expect(bloc.state.details?.recoveryKeyUnsaved, isTrue);

      bloc.add(const AccountDetailsEvent.keyShown());
      await bloc.stream.firstWhere((state) => state.newKey == null);
      await bloc.close();
    });

    test('a wrong current password is the error', () async {
      when(
        () => settings.changePassword(
          current: _password,
          newPassword: 'another long password',
        ),
      ).thenAnswer(
        (_) async => const Left<ApplicationError, RecoveryKeyText>(
          ApplicationError.auth(message: 'That isn’t your current password.'),
        ),
      );
      final bloc = await started();
      bloc.add(
        const AccountDetailsEvent.passwordChanged(
          current: _password,
          newPassword: 'another long password',
          repeat: 'another long password',
        ),
      );
      await bloc.stream.firstWhere((state) => state.error != null);
      expect(bloc.state.newKey, isNull);
      await bloc.close();
    });

    test('a new recovery key is shown', () async {
      when(() => settings.newRecoveryKey(_password)).thenAnswer(
        (_) async =>
            const Right<ApplicationError, RecoveryKeyText>(sampleRecoveryKey),
      );
      final bloc = await started();
      bloc.add(
        const AccountDetailsEvent.recoveryKeyRequested(
          currentPassword: _password,
        ),
      );
      await bloc.stream.firstWhere((state) => state.newKey != null);
      expect(bloc.state.newKey, sampleRecoveryKey);
      await bloc.close();
    });
  });

  group('AccountDetailsBloc, passkeys', () {
    test('an added passkey says whether it can unlock', () async {
      const added = PasskeyRow(
        id: 'p2',
        label: 'Security key',
        addedMs: 2,
        canUnlock: false,
      );
      when(() => settings.addPasskey(_password)).thenAnswer(
        (_) async => Right<ApplicationError, AccountDetails>(
          _withPasskeys(<PasskeyRow>[..._details.passkeys, added]),
        ),
      );
      final bloc = await started();
      bloc.add(
        const AccountDetailsEvent.passkeyAdded(currentPassword: _password),
      );
      await bloc.stream.firstWhere((state) => state.notice != null);
      expect(
        bloc.state.notice,
        const AccountNotice.passkeyAdded(canUnlock: false),
      );
      expect(bloc.state.details?.passkeys, hasLength(2));
      await bloc.close();
    });

    test('a removed passkey leaves the list', () async {
      when(
        () => settings.removePasskey(id: 'p1', currentPassword: _password),
      ).thenAnswer(
        (_) async => Right<ApplicationError, AccountDetails>(
          _withPasskeys(const <PasskeyRow>[]),
        ),
      );
      final bloc = await started();
      bloc.add(
        const AccountDetailsEvent.passkeyRemoved(
          id: 'p1',
          currentPassword: _password,
        ),
      );
      await bloc.stream.firstWhere((state) => state.notice != null);
      expect(bloc.state.notice, const AccountNotice.passkeyRemoved());
      expect(bloc.state.details?.passkeys, isEmpty);
      await bloc.close();
    });

    test('the last way in refused is the error', () async {
      when(
        () => settings.removePasskey(id: 'p1', currentPassword: _password),
      ).thenAnswer(
        (_) async => const Left<ApplicationError, AccountDetails>(
          ApplicationError.invalid(
            message:
                'This is your last way to sign in. If you remove it, you’ll '
                'need your recovery key.',
          ),
        ),
      );
      final bloc = await started();
      bloc.add(
        const AccountDetailsEvent.passkeyRemoved(
          id: 'p1',
          currentPassword: _password,
        ),
      );
      await bloc.stream.firstWhere((state) => state.error != null);
      expect(bloc.state.details, _details);
      await bloc.close();
    });
  });

  group('AccountDetailsBloc, providers', () {
    test('unlinking one gives the page again', () async {
      when(
        () => settings.unlinkProvider(
          provider: ProviderKind.github,
          currentPassword: _password,
        ),
      ).thenAnswer(
        (_) async => const Right<ApplicationError, AccountDetails>(_details),
      );
      final bloc = await started();
      bloc.add(
        const AccountDetailsEvent.providerUnlinked(
          provider: ProviderKind.github,
          currentPassword: _password,
        ),
      );
      await bloc.stream.firstWhere((state) => state.notice != null);
      expect(
        bloc.state.notice,
        const AccountNotice.providerUnlinked(provider: ProviderKind.github),
      );
      await bloc.close();
    });

    test(
      'linking drives the sign-in page, shows the code and reads again',
      () async {
        final steps = StreamController<ProviderEvent>();
        when(
          () => settings.linkProvider(
            provider: ProviderKind.github,
            currentPassword: _password,
          ),
        ).thenAnswer((_) => steps.stream);
        final bloc = await started();
        bloc.add(
          const AccountDetailsEvent.providerLinked(
            provider: ProviderKind.github,
            currentPassword: _password,
          ),
        );
        await bloc.stream.firstWhere(
          (state) => state.busy == AccountTask.linkProvider,
        );
        expect(bloc.state.linking, ProviderKind.github);

        steps
          ..add(
            const ProviderEvent.openPage(url: 'https://gh.example.invalid/l'),
          )
          ..add(
            const ProviderEvent.showCode(
              userCode: 'ABCD-1234',
              verificationUri: 'https://gh.example.invalid/login/device',
            ),
          );
        await bloc.stream.firstWhere((state) => state.code != null);
        verify(() => session.open('https://gh.example.invalid/l')).called(1);

        steps
          ..add(const ProviderEvent.closePage())
          ..add(
            const ProviderEvent.finished(
              AccountState.signedIn(username: 'ana', server: 'https://s'),
            ),
          );
        unawaited(steps.close());
        await bloc.stream.firstWhere((state) => state.notice != null);
        expect(
          bloc.state.notice,
          const AccountNotice.providerLinked(provider: ProviderKind.github),
        );
        expect(bloc.state.busy, isNull);
        expect(bloc.state.linking, isNull);
        expect(bloc.state.code, isNull);
        await bloc.close();
      },
    );

    test(
      'a link that failed is the library’s words, and nothing changes',
      () async {
        final steps = StreamController<ProviderEvent>();
        when(
          () => settings.linkProvider(
            provider: ProviderKind.github,
            currentPassword: _password,
          ),
        ).thenAnswer((_) => steps.stream);
        final bloc = await started();
        bloc.add(
          const AccountDetailsEvent.providerLinked(
            provider: ProviderKind.github,
            currentPassword: _password,
          ),
        );
        await bloc.stream.firstWhere((state) => state.linking != null);
        steps.add(
          const ProviderEvent.failed(
            message: 'That account is already linked to another Kanso account.',
          ),
        );
        unawaited(steps.close());
        await bloc.stream.firstWhere(
          (state) => state.error != null && state.busy == null,
        );
        expect(bloc.state.notice, isNull);
        await bloc.close();
      },
    );

    test('Cancel stops the link and closes the page', () async {
      final steps = StreamController<ProviderEvent>();
      when(
        () => settings.linkProvider(
          provider: ProviderKind.github,
          currentPassword: _password,
        ),
      ).thenAnswer((_) => steps.stream);
      final bloc = await started();
      bloc.add(
        const AccountDetailsEvent.providerLinked(
          provider: ProviderKind.github,
          currentPassword: _password,
        ),
      );
      await bloc.stream.firstWhere((state) => state.linking != null);
      bloc.add(const AccountDetailsEvent.providerLinkCancelled());
      await bloc.stream.firstWhere((state) => state.linking == null);
      verify(account.cancelProvider).called(1);
      verify(session.close).called(greaterThanOrEqualTo(1));
      unawaited(steps.close());
      await bloc.close();
    });
  });

  test('a message is put away', () async {
    when(() => settings.revokeDevice('d2')).thenAnswer(
      (_) async => const Left<ApplicationError, AccountDetails>(
        ApplicationError.network(message: 'Can’t reach the server.'),
      ),
    );
    final bloc = await started();
    bloc.add(const AccountDetailsEvent.deviceRevoked('d2'));
    await bloc.stream.firstWhere((state) => state.error != null);
    bloc.add(const AccountDetailsEvent.messageDismissed());
    await bloc.stream.firstWhere((state) => state.error == null);
    await bloc.close();
  });
}
