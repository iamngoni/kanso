// Kanso — account_blocs.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart' show addTearDown;
import 'package:kanso/account/state/account_details/account_details_bloc.dart';
import 'package:kanso/account/state/recover/recover_bloc.dart';
import 'package:kanso/account/state/recovery_key/recovery_key_bloc.dart';
import 'package:kanso/account/state/sign_in/sign_in_bloc.dart';
import 'package:kanso/account/state/unlock/unlock_bloc.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/state/account/account_bloc.dart';
import 'package:kanso/core/state/device_approval/device_approval_bloc.dart';
import 'package:kanso/injection.dart';

import 'seeded_account_blocs.dart';

/// The account screens' BLoCs for a test of the whole app, each in the state
/// the test gives it (see `seeded_account_blocs.dart`): the app-wide ones are
/// provided above the app, and the pages' own are what the service locator
/// makes for them.
///
/// By default the phone has no account and nothing asks to join one.
class AccountBlocs {
  AccountBlocs({
    AccountBlocState account = const AccountBlocState(
      account: AccountState.signedOut(),
    ),
    DeviceApprovalState approval = const DeviceApprovalState(),
    SignInState signIn = const SignInState(),
    UnlockState unlock = const UnlockState(),
    RecoverState recover = const RecoverState(),
    RecoveryKeyState recoveryKey = const RecoveryKeyState(),
    AccountDetailsState details = const AccountDetailsState(),
  }) : account = SeededAccountBloc(account),
       approval = SeededDeviceApprovalBloc(approval),
       signIn = SeededSignInBloc(signIn),
       unlock = SeededUnlockBloc(unlock),
       recover = SeededRecoverBloc(recover),
       recoveryKey = SeededRecoveryKeyBloc(recoveryKey),
       details = SeededAccountDetailsBloc(details) {
    addTearDown(() async {
      for (final bloc in <BlocBase<Object?>>[
        this.account,
        this.approval,
        this.signIn,
        this.unlock,
        this.recover,
        this.recoveryKey,
        this.details,
      ]) {
        unawaited(bloc.close());
      }
    });
  }

  final SeededAccountBloc account;
  final SeededDeviceApprovalBloc approval;
  final SeededSignInBloc signIn;
  final SeededUnlockBloc unlock;
  final SeededRecoverBloc recover;
  final SeededRecoveryKeyBloc recoveryKey;
  final SeededAccountDetailsBloc details;

  /// Providers of the app-wide ones, to put above the app.
  List<BlocProvider<dynamic>> get providers => <BlocProvider<dynamic>>[
    BlocProvider<AccountBloc>.value(value: account),
    BlocProvider<DeviceApprovalBloc>.value(value: approval),
  ];

  /// Makes the service locator hand out the pages' BLoCs. A page closes its
  /// BLoC as it goes, so each is handed out once.
  void register() {
    void factory<T extends Object>(T Function() make) {
      if (getIt.isRegistered<T>()) {
        getIt.unregister<T>();
      }
      getIt.registerFactory<T>(make);
    }

    factory<SignInBloc>(() => signIn);
    factory<UnlockBloc>(() => unlock);
    factory<RecoverBloc>(() => recover);
    factory<RecoveryKeyBloc>(() => recoveryKey);
    factory<AccountDetailsBloc>(() => details);
  }
}
