// Kanso — seeded_account_blocs.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:kanso/account/state/account_details/account_details_bloc.dart';
import 'package:kanso/account/state/recover/recover_bloc.dart';
import 'package:kanso/account/state/recovery_key/recovery_key_bloc.dart';
import 'package:kanso/account/state/sign_in/sign_in_bloc.dart';
import 'package:kanso/account/state/unlock/unlock_bloc.dart';
import 'package:kanso/core/state/account/account_bloc.dart';
import 'package:kanso/core/state/device_approval/device_approval_bloc.dart';

import 'fixture_account_repository.dart';
import 'fixture_approvals_repository.dart';
import 'mock_repositories.dart';

// The BLoCs of the account screens for pictures: each starts in the state
// it is given, ignores the events a page sends it when it opens, and can be
// moved to another state with `seed`, as a call to the library would.

/// An `AccountBloc` in the state it is given.
class SeededAccountBloc extends AccountBloc {
  SeededAccountBloc([AccountBlocState seed = const AccountBlocState()])
    : super(FixtureAccountRepository()) {
    emit(seed);
  }

  @override
  void add(AccountEvent event) {}

  void seed(AccountBlocState next) => emit(next);
}

/// A `DeviceApprovalBloc` in the state it is given.
class SeededDeviceApprovalBloc extends DeviceApprovalBloc {
  SeededDeviceApprovalBloc([
    DeviceApprovalState seed = const DeviceApprovalState(),
  ]) : super(FixtureApprovalsRepository()) {
    emit(seed);
  }

  @override
  void add(DeviceApprovalEvent event) {}

  void seed(DeviceApprovalState next) => emit(next);
}

/// A `SignInBloc` in the state it is given.
class SeededSignInBloc extends SignInBloc {
  SeededSignInBloc([SignInState seed = const SignInState()])
    : super(
        FixtureAccountRepository(),
        MockSyncRepository(),
        MockAuthSessionRepository(),
      ) {
    emit(seed);
  }

  @override
  void add(SignInEvent event) {}

  void seed(SignInState next) => emit(next);
}

/// An `UnlockBloc` in the state it is given.
class SeededUnlockBloc extends UnlockBloc {
  SeededUnlockBloc([UnlockState seed = const UnlockState()])
    : super(FixtureAccountRepository()) {
    emit(seed);
  }

  @override
  void add(UnlockEvent event) {}

  void seed(UnlockState next) => emit(next);
}

/// A `RecoverBloc` in the state it is given.
class SeededRecoverBloc extends RecoverBloc {
  SeededRecoverBloc([RecoverState seed = const RecoverState()])
    : super(FixtureAccountRepository(), MockSyncRepository()) {
    emit(seed);
  }

  @override
  void add(RecoverEvent event) {}

  void seed(RecoverState next) => emit(next);
}

/// A `RecoveryKeyBloc` in the state it is given.
class SeededRecoveryKeyBloc extends RecoveryKeyBloc {
  SeededRecoveryKeyBloc([RecoveryKeyState seed = const RecoveryKeyState()])
    : super(FixtureAccountRepository()) {
    emit(seed);
  }

  @override
  void add(RecoveryKeyEvent event) {}

  void seed(RecoveryKeyState next) => emit(next);
}

/// An `AccountDetailsBloc` in the state it is given.
class SeededAccountDetailsBloc extends AccountDetailsBloc {
  SeededAccountDetailsBloc([
    AccountDetailsState seed = const AccountDetailsState(),
  ]) : super(
         MockAccountSettingsRepository(),
         FixtureAccountRepository(),
         MockAuthSessionRepository(),
       ) {
    emit(seed);
  }

  @override
  void add(AccountDetailsEvent event) {}

  void seed(AccountDetailsState next) => emit(next);
}
