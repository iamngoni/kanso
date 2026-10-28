// Kanso — sign_in_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'sign_in_bloc.dart';

@freezed
sealed class SignInEvent with _$SignInEvent {
  /// The screen opened: use the default server and ask what it offers.
  const factory SignInEvent.started() = SignInStartedEvent;

  /// The server row was tapped, or its field put away.
  const factory SignInEvent.serverEdited({required bool editing}) =
      SignInServerEditedEvent;

  /// Another server was typed: use it (an empty address is the default one)
  /// and ask what it offers.
  const factory SignInEvent.serverChosen(String address) =
      SignInServerChosenEvent;

  /// The person asked to try the server again.
  const factory SignInEvent.methodsRetried() = SignInMethodsRetriedEvent;

  /// Create an account, or back to signing in.
  const factory SignInEvent.modeChanged(SignInMode mode) =
      SignInModeChangedEvent;

  /// Sign in was pressed with a username and password.
  const factory SignInEvent.passwordSubmitted({
    required String username,
    required String password,
    required String deviceName,
  }) = SignInPasswordSubmittedEvent;

  /// Create account was pressed.
  const factory SignInEvent.signUpSubmitted({
    required String username,
    required String password,
    required String repeat,
    required String deviceName,
  }) = SignInSignUpSubmittedEvent;

  /// Sign in with a passkey was pressed.
  const factory SignInEvent.passkeySignInPressed({required String deviceName}) =
      SignInPasskeySignInPressedEvent;

  /// Create with a passkey was pressed.
  const factory SignInEvent.passkeySignUpPressed({
    required String username,
    required String deviceName,
  }) = SignInPasskeySignUpPressedEvent;

  /// Continue with Apple, Google or GitHub was pressed.
  const factory SignInEvent.providerPressed({
    required ProviderKind provider,
    required String deviceName,
  }) = SignInProviderPressedEvent;

  /// Open GitHub was pressed: its page for the code opens again.
  const factory SignInEvent.githubOpened() = SignInGithubOpenedEvent;

  /// Cancel was pressed while a provider's flow runs.
  const factory SignInEvent.providerCancelled() = SignInProviderCancelledEvent;

  /// A username was chosen for the identity a provider vouched for.
  const factory SignInEvent.usernameChosen(String username) =
      SignInUsernameChosenEvent;

  /// The question about the notes on this phone was answered; null cancels.
  const factory SignInEvent.bindingAnswered(BindingAnswer? answer) =
      SignInBindingAnsweredEvent;
}
