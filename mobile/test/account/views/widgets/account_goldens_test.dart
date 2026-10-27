// Kanso — account_goldens_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

// ignore_for_file: prefer_const_constructors

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/account/models/data/provider_code.dart';
import 'package:kanso/account/models/data/sign_in_mode.dart';
import 'package:kanso/account/state/account_details/account_details_bloc.dart';
import 'package:kanso/account/state/recover/recover_bloc.dart';
import 'package:kanso/account/state/recovery_key/recovery_key_bloc.dart';
import 'package:kanso/account/state/sign_in/sign_in_bloc.dart';
import 'package:kanso/account/state/unlock/unlock_bloc.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/core/state/account/account_bloc.dart';
import 'package:kanso/core/state/device_approval/device_approval_bloc.dart';

import '../../../helpers/account_blocs.dart';
import '../../../helpers/app_golden.dart';
import '../../../helpers/fixture_account_repository.dart';
import '../../../helpers/fixture_account_settings_repository.dart';
import '../../../helpers/fixture_approvals_repository.dart';
import '../../../helpers/kanso_viewport.dart';
import '../../../helpers/pump_browse_app.dart';
import '../../../helpers/real_themes.dart';

const String _server = 'https://sync.kansoapp.app';

const AccountState _signedIn = AccountState.signedIn(
  username: 'ana',
  server: _server,
);

const LockedInfo _locked = LockedInfo(
  username: 'ana',
  server: _server,
  hasPassword: true,
  approvalPending: false,
);

const SignInState _signInReady = SignInState(
  server: _server,
  defaultServer: _server,
  methodsStatus: LoadStatus.ready,
  methods: allMethods,
);

const BindingQuestion _question = BindingQuestion(
  title: 'This device already has notes',
  body: '12 notes on this device aren’t in ana’s account.',
  answers: <BindingAnswer>[BindingAnswer.add, BindingAnswer.replace],
);

/// One screen of the account, as a picture of the whole app.
class _Screen {
  const _Screen(this.name, this.at, this.accounts, {this.interact});

  final String name;

  /// The screens opened in Settings, in order.
  final List<String> at;
  final AccountBlocs Function() accounts;
  final Future<void> Function(WidgetTester tester, BrowseApp app)? interact;
}

AccountBlocs _signedOutWith({
  SignInState signIn = _signInReady,
}) => AccountBlocs(signIn: signIn);

AccountBlocs _lockedWith({
  UnlockState unlock = const UnlockState(),
  RecoverState recover = const RecoverState(server: _server),
}) => AccountBlocs(
  account: const AccountBlocState(account: AccountState.locked(_locked)),
  unlock: unlock,
  recover: recover,
);

AccountBlocs _signedInWith({
  AccountDetailsState details = const AccountDetailsState(
    status: LoadStatus.ready,
    details: sampleDetails,
  ),
  RecoveryKeyState recoveryKey = const RecoveryKeyState(
    status: LoadStatus.ready,
    key: sampleRecoveryKey,
  ),
}) => AccountBlocs(
  account: const AccountBlocState(account: _signedIn),
  details: details,
  recoveryKey: recoveryKey,
);

Future<void> _push(WidgetTester tester, BrowseApp app, String path) async {
  unawaited(app.router.push<void>(path));
  await tester.pumpAndSettle();
}

Future<void> _openQuestion(WidgetTester tester, BrowseApp app) async {
  app.accounts.signIn.seed(
    app.accounts.signIn.state.copyWith(question: _question),
  );
  await tester.pumpAndSettle();
}

const String _signIn = '/settings/account/sign-in';
const String _unlock = '/settings/account/unlock';
const String _recover = '/settings/account/recover';
const String _key = '/settings/account/recovery-key';
const String _account = '/settings/account';

final List<_Screen> _screens = <_Screen>[
  _Screen('sign_in', <String>[_signIn], _signedOutWith),
  _Screen(
    'sign_in_error',
    <String>[_signIn],
    () => _signedOutWith(
      signIn: _signInReady.copyWith(
        error: const ApplicationError.auth(
          message: 'That username and password don’t match.',
        ),
      ),
    ),
  ),
  _Screen(
    'sign_in_server_failed',
    <String>[_signIn],
    () => _signedOutWith(
      signIn: const SignInState(
        server: 'https://sync.example.org',
        defaultServer: _server,
        methodsStatus: LoadStatus.failed,
        error: ApplicationError.network(
          message:
              'Can’t reach the server. Your notes are safe on this '
              'device.',
        ),
      ),
    ),
  ),
  _Screen(
    'sign_in_loading',
    <String>[_signIn],
    () => _signedOutWith(
      signIn: const SignInState(server: _server, defaultServer: _server),
    ),
  ),
  _Screen(
    'create_account',
    <String>[_signIn],
    () => _signedOutWith(
      signIn: _signInReady.copyWith(mode: SignInMode.create),
    ),
  ),
  _Screen(
    'github_code',
    <String>[_signIn],
    () => _signedOutWith(
      signIn: _signInReady.copyWith(
        code: const ProviderCode(
          userCode: 'WDJB-MJHT',
          verificationUri: 'https://github.com/login/device',
        ),
      ),
    ),
  ),
  _Screen(
    'choose_username',
    <String>[_signIn],
    () =>
        _signedOutWith(signIn: _signInReady.copyWith(suggestedUsername: 'ana')),
  ),
  _Screen(
    'binding_question',
    <String>[_signIn],
    _signedOutWith,
    interact: _openQuestion,
  ),
  _Screen('recovery_key', <String>[_key], _signedInWith),
  _Screen(
    'recovery_key_confirmed',
    <String>[_key],
    () => _signedInWith(
      recoveryKey: const RecoveryKeyState(
        status: LoadStatus.ready,
        key: sampleRecoveryKey,
        confirmed: true,
      ),
    ),
  ),
  _Screen('unlock', <String>[_unlock], _lockedWith),
  _Screen(
    'approval_waiting',
    <String>[_unlock],
    () => _lockedWith(
      unlock: const UnlockState(
        approval: ApprovalWait(
          phase: ApprovalPhase.waiting(),
          otherDevices: 1,
          offerRecovery: false,
        ),
      ),
    ),
  ),
  _Screen(
    'approval_compare',
    <String>[_unlock],
    () => _lockedWith(
      unlock: const UnlockState(
        approval: ApprovalWait(
          phase: ApprovalPhase.compare(),
          code: '123 456',
          otherDevices: 1,
          offerRecovery: true,
        ),
      ),
    ),
  ),
  _Screen(
    'approval_expired',
    <String>[_unlock],
    () => _lockedWith(
      unlock: const UnlockState(
        approval: ApprovalWait(
          phase: ApprovalPhase.expired(),
          otherDevices: 1,
          offerRecovery: true,
        ),
      ),
    ),
  ),
  _Screen('recover', <String>[_recover], _signedOutWith),
  _Screen('recover_locked', <String>[_recover], _lockedWith),
  _Screen('account', <String>[_account], _signedInWith),
  _Screen(
    'account_loading',
    <String>[_account],
    () => _signedInWith(details: const AccountDetailsState()),
  ),
  _Screen(
    'account_failed',
    <String>[_account],
    () => _signedInWith(
      details: const AccountDetailsState(
        status: LoadStatus.failed,
        loadError: ApplicationError.network(message: 'Offline.'),
      ),
    ),
  ),
  _Screen('settings_sync_signed_out', <String>[], AccountBlocs.new),
  _Screen('settings_sync_locked', <String>[], _lockedWith),
  _Screen('settings_sync_signed_in', <String>[], _signedInWith),
  _Screen(
    'approve_device',
    <String>[],
    () => AccountBlocs(
      account: const AccountBlocState(account: _signedIn),
      approval: DeviceApprovalState(prompts: <ApprovalPrompt>[readyPrompt()]),
    ),
  ),
  _Screen(
    'approve_device_changed',
    <String>[],
    () => AccountBlocs(
      account: const AccountBlocState(account: _signedIn),
      approval: DeviceApprovalState(
        prompts: <ApprovalPrompt>[readyPrompt(state: PromptState.changed)],
      ),
    ),
  ),
];

/// The screens that are Settings itself, not a page of the account.
const Set<String> _inSettings = <String>{
  'settings_sync_signed_out',
  'settings_sync_locked',
  'settings_sync_signed_in',
};

Future<void> _open(WidgetTester tester, BrowseApp app, _Screen screen) async {
  if (screen.at.isNotEmpty || _inSettings.contains(screen.name)) {
    await _push(tester, app, '/settings');
  }
  for (final path in screen.at) {
    await _push(tester, app, path);
  }
  await screen.interact?.call(tester, app);
}

void main() {
  loadRealThemes();

  for (final viewport in <KansoViewport>[
    KansoViewport.phone,
    ...KansoViewport.goldenSet,
  ]) {
    for (final brightness in Brightness.values) {
      group('${viewport.slug}, ${brightness.name}', () {
        for (final screen in _screens) {
          testWidgets(screen.name, (tester) async {
            await expectAppGoldenOnBothPlatforms(
              tester,
              name:
                  'account_${viewport.slug}_${screen.name}_'
                  '${brightness.name}',
              viewport: viewport,
              brightness: brightness,
              accounts: screen.accounts,
              interact: (tester, app) => _open(tester, app, screen),
            );
          });
        }
      });
    }
  }
}
