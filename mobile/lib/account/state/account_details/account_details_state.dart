// Kanso — account_details_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'account_details_bloc.dart';

@freezed
abstract class AccountDetailsState with _$AccountDetailsState {
  const factory AccountDetailsState({
    @Default(LoadStatus.loading) LoadStatus status,

    /// The account; null until it is read.
    AccountDetails? details,

    /// Why the account could not be read.
    ApplicationError? loadError,

    /// The change being made, if one is.
    AccountTask? busy,

    /// What is wrong with what was typed.
    FormProblem? problem,

    /// Why the last change failed.
    ApplicationError? error,

    /// What the last change that went through did.
    AccountNotice? notice,

    /// The recovery key a change made, until the page shows it.
    RecoveryKeyText? newKey,

    /// The provider being linked.
    ProviderKind? linking,

    /// The code to type at GitHub while its link waits.
    ProviderCode? code,

    /// How the page ended; null while it has not.
    AccountOutcome? outcome,
  }) = _AccountDetailsState;
}
