// Kanso — account_details_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:dartz/dartz.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/account_repository.dart';
import '../../../core/models/repos/abs/account_settings_repository.dart';
import '../../../core/models/repos/abs/auth_session_repository.dart';
import '../../models/data/account_notice.dart';
import '../../models/data/account_outcome.dart';
import '../../models/data/account_task.dart';
import '../../models/data/form_problem.dart';
import '../../models/data/provider_code.dart';
import '../../utils/account_forms.dart';
import '../../utils/provider_flow_driver.dart';

part 'account_details_bloc.freezed.dart';
part 'account_details_event.dart';
part 'account_details_state.dart';

/// The Account page: the devices and ways in of the account that is signed
/// in, and the changes to them. Each change that asks for a proof is made
/// with the account's password.
@injectable
class AccountDetailsBloc
    extends Bloc<AccountDetailsEvent, AccountDetailsState> {
  AccountDetailsBloc(this._settings, this._account, this._session)
    : super(const AccountDetailsState()) {
    on<AccountDetailsStartedEvent>(_onStarted, transformer: restartable());
    on<AccountDetailsDeviceRevokedEvent>(
      _onDeviceRevoked,
      transformer: droppable(),
    );
    on<AccountDetailsSignedOutEvent>(_onSignedOut, transformer: droppable());
    on<AccountDetailsPasswordChangedEvent>(
      _onPasswordChanged,
      transformer: droppable(),
    );
    on<AccountDetailsRecoveryKeyRequestedEvent>(
      _onRecoveryKeyRequested,
      transformer: droppable(),
    );
    on<AccountDetailsAccountDeletedEvent>(
      _onAccountDeleted,
      transformer: droppable(),
    );
    on<AccountDetailsPasskeyAddedEvent>(
      _onPasskeyAdded,
      transformer: droppable(),
    );
    on<AccountDetailsPasskeyRemovedEvent>(
      _onPasskeyRemoved,
      transformer: droppable(),
    );
    on<AccountDetailsProviderLinkedEvent>(
      _onProviderLinked,
      transformer: droppable(),
    );
    on<AccountDetailsProviderLinkCancelledEvent>(_onProviderLinkCancelled);
    on<AccountDetailsProviderUnlinkedEvent>(
      _onProviderUnlinked,
      transformer: droppable(),
    );
    on<AccountDetailsKeyShownEvent>(_onKeyShown);
    on<AccountDetailsMessageDismissedEvent>(_onMessageDismissed);
  }

  final AccountSettingsRepository _settings;
  final AccountRepository _account;
  final AuthSessionRepository _session;

  Future<void> _onStarted(
    AccountDetailsStartedEvent event,
    Emitter<AccountDetailsState> emit,
  ) async {
    emit(const AccountDetailsState());
    final result = await _settings.details();
    emit(
      result.fold(
        (error) =>
            AccountDetailsState(status: LoadStatus.failed, loadError: error),
        (details) =>
            AccountDetailsState(status: LoadStatus.ready, details: details),
      ),
    );
  }

  Future<void> _onDeviceRevoked(
    AccountDetailsDeviceRevokedEvent event,
    Emitter<AccountDetailsState> emit,
  ) async {
    final name = state.details?.devices
        .where((device) => device.id == event.deviceId)
        .map((device) => device.title)
        .firstOrNull;
    emit(_busy(AccountTask.revokeDevice));
    final result = await _settings.revokeDevice(event.deviceId);
    emit(
      _afterDetails(result, AccountNotice.deviceSignedOut(name: name ?? '')),
    );
  }

  Future<void> _onSignedOut(
    AccountDetailsSignedOutEvent event,
    Emitter<AccountDetailsState> emit,
  ) async {
    emit(_busy(AccountTask.signOut));
    final result = await _settings.signOut();
    emit(_afterEnd(result));
  }

  Future<void> _onPasswordChanged(
    AccountDetailsPasswordChangedEvent event,
    Emitter<AccountDetailsState> emit,
  ) async {
    final problem = AccountForms.newPasswordProblem(
      event.newPassword,
      event.repeat,
      state.details?.username ?? '',
    );
    if (problem != null) {
      emit(state.copyWith(problem: problem, error: null));
      return;
    }
    emit(_busy(AccountTask.changePassword));
    final result = await _settings.changePassword(
      current: event.current,
      newPassword: event.newPassword,
    );
    emit(_afterKey(result));
  }

  Future<void> _onRecoveryKeyRequested(
    AccountDetailsRecoveryKeyRequestedEvent event,
    Emitter<AccountDetailsState> emit,
  ) async {
    emit(_busy(AccountTask.newRecoveryKey));
    final result = await _settings.newRecoveryKey(event.currentPassword);
    emit(_afterKey(result));
  }

  Future<void> _onAccountDeleted(
    AccountDetailsAccountDeletedEvent event,
    Emitter<AccountDetailsState> emit,
  ) async {
    if (event.typedUsername.trim() != state.details?.username) {
      emit(state.copyWith(problem: FormProblem.usernameInvalid, error: null));
      return;
    }
    emit(_busy(AccountTask.deleteAccount));
    final result = await _settings.deleteAccount(
      password: event.password,
      typedUsername: event.typedUsername.trim(),
    );
    emit(_afterEnd(result));
  }

  Future<void> _onPasskeyAdded(
    AccountDetailsPasskeyAddedEvent event,
    Emitter<AccountDetailsState> emit,
  ) async {
    final before = <String>{
      for (final passkey in state.details?.passkeys ?? <PasskeyRow>[])
        passkey.id,
    };
    emit(_busy(AccountTask.addPasskey));
    final result = await _settings.addPasskey(event.currentPassword);
    final canUnlock = result
        .fold(
          (_) => <PasskeyRow>[],
          (details) => details.passkeys
              .where((passkey) => !before.contains(passkey.id))
              .toList(),
        )
        .any((passkey) => passkey.canUnlock);
    emit(
      _afterDetails(result, AccountNotice.passkeyAdded(canUnlock: canUnlock)),
    );
  }

  Future<void> _onPasskeyRemoved(
    AccountDetailsPasskeyRemovedEvent event,
    Emitter<AccountDetailsState> emit,
  ) async {
    emit(_busy(AccountTask.removePasskey));
    final result = await _settings.removePasskey(
      id: event.id,
      currentPassword: event.currentPassword,
    );
    emit(_afterDetails(result, const AccountNotice.passkeyRemoved()));
  }

  Future<void> _onProviderLinked(
    AccountDetailsProviderLinkedEvent event,
    Emitter<AccountDetailsState> emit,
  ) async {
    emit(_busy(AccountTask.linkProvider).copyWith(linking: event.provider));
    final driver = ProviderFlowDriver(_session, _account.providerPageEnded);
    var linked = false;
    await emit.onEach<ProviderEvent>(
      _settings.linkProvider(
        provider: event.provider,
        currentPassword: event.currentPassword,
      ),
      onData: (step) {
        driver.handle(step);
        switch (step) {
          case ProviderEvent_ShowCode(:final userCode, :final verificationUri):
            emit(
              state.copyWith(
                code: ProviderCode(
                  userCode: userCode,
                  verificationUri: verificationUri,
                ),
              ),
            );
          case ProviderEvent_Finished():
            linked = true;
          case ProviderEvent_Failed(:final message):
            emit(
              state.copyWith(error: ApplicationError.invalid(message: message)),
            );
          case ProviderEvent_OpenPage():
          case ProviderEvent_ClosePage():
          case ProviderEvent_NeedsUsername():
          case ProviderEvent_Cancelled():
            break;
        }
      },
    );
    emit(state.copyWith(busy: null, linking: null, code: null));
    if (linked) {
      final result = await _settings.details();
      emit(
        _afterDetails(
          result,
          AccountNotice.providerLinked(provider: event.provider),
        ),
      );
    }
  }

  Future<void> _onProviderLinkCancelled(
    AccountDetailsProviderLinkCancelledEvent event,
    Emitter<AccountDetailsState> emit,
  ) async {
    await _account.cancelProvider();
    await _session.close();
    emit(state.copyWith(busy: null, linking: null, code: null));
  }

  Future<void> _onProviderUnlinked(
    AccountDetailsProviderUnlinkedEvent event,
    Emitter<AccountDetailsState> emit,
  ) async {
    emit(_busy(AccountTask.unlinkProvider));
    final result = await _settings.unlinkProvider(
      provider: event.provider,
      currentPassword: event.currentPassword,
    );
    emit(
      _afterDetails(
        result,
        AccountNotice.providerUnlinked(provider: event.provider),
      ),
    );
  }

  void _onKeyShown(
    AccountDetailsKeyShownEvent event,
    Emitter<AccountDetailsState> emit,
  ) {
    emit(state.copyWith(newKey: null));
  }

  void _onMessageDismissed(
    AccountDetailsMessageDismissedEvent event,
    Emitter<AccountDetailsState> emit,
  ) {
    emit(state.copyWith(error: null, problem: null, notice: null));
  }

  AccountDetailsState _busy(AccountTask task) {
    return state.copyWith(
      busy: task,
      problem: null,
      error: null,
      notice: null,
      newKey: null,
    );
  }

  /// The page after a call that gave the account's details again.
  AccountDetailsState _afterDetails(
    Either<ApplicationError, AccountDetails> result,
    AccountNotice notice,
  ) {
    return result.fold(
      (error) => state.copyWith(busy: null, error: error),
      (details) => state.copyWith(busy: null, details: details, notice: notice),
    );
  }

  /// The page after a call that made a new recovery key: the key is shown,
  /// and the page reads again, as the key is unsaved now.
  AccountDetailsState _afterKey(
    Either<ApplicationError, RecoveryKeyText> result,
  ) {
    return result.fold(
      (error) => state.copyWith(busy: null, error: error),
      (key) => state.copyWith(
        busy: null,
        newKey: key,
        details: state.details?.copyWithUnsavedKey(),
      ),
    );
  }

  /// The page after a call that ended the account's session on this phone.
  AccountDetailsState _afterEnd(Either<ApplicationError, Unit> result) {
    return result.fold(
      (error) => state.copyWith(busy: null, error: error),
      (_) => state.copyWith(busy: null, outcome: AccountOutcome.signedOut),
    );
  }
}

/// A change to [AccountDetails] the page makes itself.
extension on AccountDetails {
  /// These details, once a recovery key is made that is not yet saved.
  AccountDetails copyWithUnsavedKey() {
    return AccountDetails(
      username: username,
      server: server,
      devices: devices,
      storageLine: storageLine,
      hasPassword: hasPassword,
      providers: providers,
      passkeys: passkeys,
      recoveryKeyUnsaved: true,
    );
  }
}
