// Kanso — sign_in_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'sign_in_bloc.dart';

@freezed
abstract class SignInState with _$SignInState {
  const factory SignInState({
    /// The server in use: the default one until another is typed.
    @Default('') String server,

    /// The server a new account is made on unless another is typed.
    @Default('') String defaultServer,

    /// Whether the field for another server is open.
    @Default(false) bool editingServer,

    /// Whether what the server offers has been read.
    @Default(LoadStatus.loading) LoadStatus methodsStatus,

    /// What the server offers; null until it is read.
    ServerMethods? methods,
    @Default(SignInMode.signIn) SignInMode mode,

    /// What is running, if something is.
    SignInAction? busy,

    /// What is wrong with what was typed.
    FormProblem? problem,

    /// Why the last call failed.
    ApplicationError? error,

    /// The code to type at GitHub while its flow waits.
    ProviderCode? code,

    /// The username the library suggests for a provider's new identity;
    /// non-null while the screen asks for one.
    String? suggestedUsername,

    /// The question about the notes on this phone, while it waits.
    BindingQuestion? question,

    /// Whether what is being done makes an account (not signs in to one).
    @Default(false) bool creating,

    /// The recovery key of the account just made, to write down.
    RecoveryKeyText? recoveryKey,

    /// How the flow ended; null while it has not.
    AccountOutcome? outcome,
  }) = _SignInState;

  const SignInState._();

  /// Whether the server in use is not the default one.
  bool get customServer => server != defaultServer;

  /// Whether the screen asks for a username for a provider's identity.
  bool get askingUsername => suggestedUsername != null;
}
