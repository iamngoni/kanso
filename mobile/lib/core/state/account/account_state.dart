// Kanso — account_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'account_bloc.dart';

/// The account as the app follows it. (Not `AccountState`, which is the
/// library's own word for the same thing and is held in [account].)
@freezed
abstract class AccountBlocState with _$AccountBlocState {
  const factory AccountBlocState({
    /// The account now; null until it is first read.
    AccountState? account,
  }) = _AccountBlocState;

  const AccountBlocState._();

  /// Whether an account is signed in and open on this phone.
  bool get signedIn => account is AccountState_SignedIn;

  /// The account that is signed in but has no key on this phone yet, if
  /// there is one.
  LockedInfo? get locked {
    final current = account;
    return current is AccountState_Locked ? current.field0 : null;
  }

  /// The username of the account that is signed in or locked.
  String? get username {
    final current = account;
    return switch (current) {
      AccountState_SignedIn(:final username) => username,
      AccountState_Locked(:final field0) => field0.username,
      _ => null,
    };
  }
}
