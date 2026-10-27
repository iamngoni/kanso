// Kanso — sign_in_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:dartz/dartz.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/account_repository.dart';
import '../../../core/models/repos/abs/auth_session_repository.dart';
import '../../../core/models/repos/abs/sync_repository.dart';
import '../../models/data/account_outcome.dart';
import '../../models/data/form_problem.dart';
import '../../models/data/provider_code.dart';
import '../../models/data/sign_in_action.dart';
import '../../models/data/sign_in_mode.dart';
import '../../utils/account_forms.dart';
import '../../utils/provider_flow_driver.dart';

part 'sign_in_bloc.freezed.dart';
part 'sign_in_event.dart';
part 'sign_in_state.dart';

/// Signing in and making an account: the server and what it offers, a
/// username and password, a passkey, Apple, Google or GitHub, the question
/// about the notes already on this phone, and the recovery key of a new
/// account.
///
/// A flow ends in [SignInState.outcome]; the screen moves on from there.
@injectable
class SignInBloc extends Bloc<SignInEvent, SignInState> {
  SignInBloc(this._account, this._sync, this._session)
    : super(const SignInState()) {
    on<SignInStartedEvent>(_onStarted, transformer: restartable());
    on<SignInServerEditedEvent>(_onServerEdited);
    on<SignInServerChosenEvent>(_onServerChosen, transformer: restartable());
    on<SignInMethodsRetriedEvent>(
      _onMethodsRetried,
      transformer: restartable(),
    );
    on<SignInModeChangedEvent>(_onModeChanged);
    on<SignInPasswordSubmittedEvent>(
      _onPasswordSubmitted,
      transformer: droppable(),
    );
    on<SignInSignUpSubmittedEvent>(
      _onSignUpSubmitted,
      transformer: droppable(),
    );
    on<SignInPasskeySignInPressedEvent>(
      _onPasskeySignIn,
      transformer: droppable(),
    );
    on<SignInPasskeySignUpPressedEvent>(
      _onPasskeySignUp,
      transformer: droppable(),
    );
    on<SignInProviderPressedEvent>(
      _onProviderPressed,
      transformer: droppable(),
    );
    on<SignInGithubOpenedEvent>(_onGithubOpened);
    on<SignInProviderCancelledEvent>(_onProviderCancelled);
    on<SignInUsernameChosenEvent>(_onUsernameChosen, transformer: droppable());
    on<SignInBindingAnsweredEvent>(
      _onBindingAnswered,
      transformer: droppable(),
    );
  }

  final AccountRepository _account;
  final SyncRepository _sync;
  final AuthSessionRepository _session;

  Future<void> _onStarted(
    SignInStartedEvent event,
    Emitter<SignInState> emit,
  ) async {
    final found = await _sync.defaultServer();
    final server = found.fold((_) => '', (address) => address);
    emit(state.copyWith(server: server, defaultServer: server));
    await _loadMethods(emit);
  }

  void _onServerEdited(
    SignInServerEditedEvent event,
    Emitter<SignInState> emit,
  ) {
    emit(state.copyWith(editingServer: event.editing));
  }

  Future<void> _onServerChosen(
    SignInServerChosenEvent event,
    Emitter<SignInState> emit,
  ) async {
    final typed = event.address.trim();
    emit(
      state.copyWith(
        server: typed.isEmpty ? state.defaultServer : typed,
        editingServer: false,
      ),
    );
    await _loadMethods(emit);
  }

  Future<void> _onMethodsRetried(
    SignInMethodsRetriedEvent event,
    Emitter<SignInState> emit,
  ) => _loadMethods(emit);

  Future<void> _loadMethods(Emitter<SignInState> emit) async {
    emit(
      state.copyWith(
        methodsStatus: LoadStatus.loading,
        methods: null,
        error: null,
      ),
    );
    final result = await _account.serverMethods(state.server);
    emit(
      result.fold(
        (error) => state.copyWith(
          methodsStatus: LoadStatus.failed,
          methods: null,
          error: error,
        ),
        (methods) =>
            state.copyWith(methodsStatus: LoadStatus.ready, methods: methods),
      ),
    );
  }

  void _onModeChanged(SignInModeChangedEvent event, Emitter<SignInState> emit) {
    emit(state.copyWith(mode: event.mode, problem: null, error: null));
  }

  Future<void> _onPasswordSubmitted(
    SignInPasswordSubmittedEvent event,
    Emitter<SignInState> emit,
  ) async {
    final problem = AccountForms.signInProblem(event.username, event.password);
    if (problem != null) {
      emit(state.copyWith(problem: problem, error: null));
      return;
    }
    emit(_busy(SignInAction.password));
    final result = await _account.signInPassword(
      server: state.server,
      username: AccountForms.normalizeUsername(event.username)!,
      password: event.password,
      deviceName: event.deviceName,
    );
    emit(_afterState(result));
  }

  Future<void> _onSignUpSubmitted(
    SignInSignUpSubmittedEvent event,
    Emitter<SignInState> emit,
  ) async {
    final problem = AccountForms.signUpProblem(
      event.username,
      event.password,
      event.repeat,
    );
    if (problem != null) {
      emit(state.copyWith(problem: problem, error: null));
      return;
    }
    emit(_busy(SignInAction.create));
    final result = await _account.signUpPassword(
      server: state.server,
      username: AccountForms.normalizeUsername(event.username)!,
      password: event.password,
      deviceName: event.deviceName,
    );
    emit(_afterSignUp(result));
  }

  Future<void> _onPasskeySignIn(
    SignInPasskeySignInPressedEvent event,
    Emitter<SignInState> emit,
  ) async {
    emit(_busy(SignInAction.passkey));
    final result = await _account.passkeySignIn(
      server: state.server,
      deviceName: event.deviceName,
    );
    emit(_afterState(result));
  }

  Future<void> _onPasskeySignUp(
    SignInPasskeySignUpPressedEvent event,
    Emitter<SignInState> emit,
  ) async {
    final problem = AccountForms.newUsernameProblem(event.username);
    if (problem != null) {
      emit(state.copyWith(problem: problem, error: null));
      return;
    }
    emit(_busy(SignInAction.passkey));
    final result = await _account.passkeySignUp(
      server: state.server,
      username: AccountForms.normalizeUsername(event.username)!,
      deviceName: event.deviceName,
    );
    emit(_afterSignUp(result));
  }

  Future<void> _onProviderPressed(
    SignInProviderPressedEvent event,
    Emitter<SignInState> emit,
  ) async {
    emit(_busy(SignInAction.ofProvider(event.provider)));
    final driver = ProviderFlowDriver(_session, _account.providerPageEnded);
    await emit.onEach<ProviderEvent>(
      _account.startProvider(
        server: state.server,
        provider: event.provider,
        deviceName: event.deviceName,
      ),
      onData: (step) {
        driver.handle(step);
        emit(_afterProviderStep(step));
      },
    );
    // A flow that ended without saying how is over all the same.
    if (state.busy != null && state.busy != SignInAction.binding) {
      emit(state.copyWith(busy: null, code: null));
    }
  }

  void _onGithubOpened(
    SignInGithubOpenedEvent event,
    Emitter<SignInState> emit,
  ) {
    final page = state.code?.verificationUri;
    if (page != null) {
      // The flow does not wait on this page: it polls GitHub for the code
      // being entered, so how this session ends is no one's business.
      unawaited(_session.open(page));
    }
  }

  Future<void> _onProviderCancelled(
    SignInProviderCancelledEvent event,
    Emitter<SignInState> emit,
  ) async {
    await _account.cancelProvider();
    await _session.close();
    emit(state.copyWith(busy: null, code: null, suggestedUsername: null));
  }

  Future<void> _onUsernameChosen(
    SignInUsernameChosenEvent event,
    Emitter<SignInState> emit,
  ) async {
    final problem = AccountForms.newUsernameProblem(event.username);
    if (problem != null) {
      emit(state.copyWith(problem: problem, error: null));
      return;
    }
    emit(_busy(SignInAction.username));
    final result = await _account.finishProviderSignUp(
      AccountForms.normalizeUsername(event.username)!,
    );
    emit(_afterSignUp(result));
  }

  Future<void> _onBindingAnswered(
    SignInBindingAnsweredEvent event,
    Emitter<SignInState> emit,
  ) async {
    emit(state.copyWith(question: null, busy: SignInAction.binding));
    final result = await _account.answerBinding(event.answer);
    var next = _afterState(result);
    // The key of an account made over this phone's own notes is made when
    // the question is answered; it is the library that has it.
    if (next.creating &&
        next.outcome == AccountOutcome.signedIn &&
        next.recoveryKey == null) {
      final key = await _account.pendingRecoveryKey();
      next = next.copyWith(
        recoveryKey: key.fold((_) => null, (found) => found),
      );
    }
    emit(next);
  }

  SignInState _busy(SignInAction action) {
    return state.copyWith(
      busy: action,
      problem: null,
      error: null,
      question: null,
      creating: false,
      recoveryKey: null,
      outcome: null,
    );
  }

  /// The screen after a call that gave the account back.
  SignInState _afterState(Either<ApplicationError, AccountState> result) {
    return result.fold(
      (error) => state.copyWith(busy: null, error: error),
      _afterAccount,
    );
  }

  /// The screen after a call that made an account.
  SignInState _afterSignUp(Either<ApplicationError, SignedUp> result) {
    return result.fold((error) => state.copyWith(busy: null, error: error), (
      made,
    ) {
      final key = made.recoveryKey.key.isEmpty ? null : made.recoveryKey;
      return _afterAccount(
        made.state,
      ).copyWith(creating: true, recoveryKey: key, suggestedUsername: null);
    });
  }

  SignInState _afterAccount(AccountState account) {
    return switch (account) {
      AccountState_SignedIn() => state.copyWith(
        busy: null,
        code: null,
        outcome: AccountOutcome.signedIn,
      ),
      AccountState_Locked() => state.copyWith(
        busy: null,
        code: null,
        outcome: AccountOutcome.locked,
      ),
      AccountState_Question(:final field0) => state.copyWith(
        busy: null,
        code: null,
        question: field0,
      ),
      // Nothing changed: a passkey sheet that was dismissed, a sign-in that
      // was cancelled.
      AccountState_SignedOut() => state.copyWith(busy: null, code: null),
    };
  }

  SignInState _afterProviderStep(ProviderEvent step) {
    return switch (step) {
      ProviderEvent_OpenPage() || ProviderEvent_ClosePage() => state,
      ProviderEvent_ShowCode(:final userCode, :final verificationUri) =>
        state.copyWith(
          code: ProviderCode(
            userCode: userCode,
            verificationUri: verificationUri,
          ),
        ),
      ProviderEvent_NeedsUsername(:final suggested) => state.copyWith(
        busy: null,
        code: null,
        suggestedUsername: suggested,
      ),
      ProviderEvent_Finished(:final field0) => _afterAccount(field0),
      ProviderEvent_Failed(:final message) => state.copyWith(
        busy: null,
        code: null,
        error: ApplicationError.invalid(message: message),
      ),
      ProviderEvent_Cancelled() => state.copyWith(busy: null, code: null),
    };
  }
}
