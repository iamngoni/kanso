// Kanso — unlock_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/account_repository.dart';
import '../../models/data/account_outcome.dart';
import '../../models/data/form_problem.dart';
import '../../utils/account_forms.dart';

part 'unlock_bloc.freezed.dart';
part 'unlock_event.dart';
part 'unlock_state.dart';

/// Opening the notes on a phone that is signed in but has no key for them:
/// with the account's password, by another device approving this one, or
/// (see `RecoverBloc`) with the recovery key. Or signing out instead.
@injectable
class UnlockBloc extends Bloc<UnlockEvent, UnlockState> {
  UnlockBloc(this._account) : super(const UnlockState()) {
    on<UnlockStartedEvent>(_onStarted, transformer: restartable());
    on<UnlockPasswordSubmittedEvent>(
      _onPasswordSubmitted,
      transformer: droppable(),
    );
    on<UnlockApprovalRequestedEvent>(
      _onApprovalRequested,
      transformer: droppable(),
    );
    on<UnlockApprovalCancelledEvent>(_onApprovalCancelled);
    on<UnlockSignedOutEvent>(_onSignedOut, transformer: droppable());
    on<UnlockBindingAnsweredEvent>(
      _onBindingAnswered,
      transformer: droppable(),
    );
  }

  final AccountRepository _account;

  /// Whether the person cancelled the wait: what the library still says of
  /// it is not shown.
  bool _approvalCancelled = false;

  Future<void> _onStarted(
    UnlockStartedEvent event,
    Emitter<UnlockState> emit,
  ) async {
    // A request this phone made earlier is picked up where it was.
    if (event.resumeApproval) {
      add(UnlockEvent.approvalRequested(deviceName: event.deviceName));
    }
  }

  Future<void> _onPasswordSubmitted(
    UnlockPasswordSubmittedEvent event,
    Emitter<UnlockState> emit,
  ) async {
    final problem = AccountForms.unlockProblem(event.password, event.username);
    if (problem != null) {
      emit(state.copyWith(problem: problem, error: null));
      return;
    }
    emit(state.copyWith(busy: true, problem: null, error: null));
    final result = await _account.unlockWithPassword(event.password);
    emit(
      result.fold((error) => state.copyWith(busy: false, error: error), _after),
    );
  }

  Future<void> _onApprovalRequested(
    UnlockApprovalRequestedEvent event,
    Emitter<UnlockState> emit,
  ) async {
    _approvalCancelled = false;
    emit(
      state.copyWith(
        approval: const ApprovalWait(
          phase: ApprovalPhase.requesting(),
          otherDevices: 0,
          offerRecovery: false,
        ),
        problem: null,
        error: null,
      ),
    );
    await emit.onEach<ApprovalWait>(
      _account.requestApproval(event.deviceName),
      onData: (wait) {
        if (!_approvalCancelled) {
          emit(state.copyWith(approval: wait));
        }
      },
    );
    if (_approvalCancelled ||
        state.approval?.phase is! ApprovalPhase_Approved) {
      return;
    }
    // Approved: the library has the key, and says what the account is now.
    final result = await _account.state();
    emit(
      result.fold(
        (error) => state.copyWith(approval: null, error: error),
        (account) => _after(account, clearApproval: true),
      ),
    );
  }

  Future<void> _onApprovalCancelled(
    UnlockApprovalCancelledEvent event,
    Emitter<UnlockState> emit,
  ) async {
    _approvalCancelled = true;
    emit(state.copyWith(approval: null));
    await _account.cancelApproval();
  }

  Future<void> _onSignedOut(
    UnlockSignedOutEvent event,
    Emitter<UnlockState> emit,
  ) async {
    emit(state.copyWith(busy: true, problem: null, error: null));
    final result = await _account.cancelLocked();
    emit(
      result.fold(
        (error) => state.copyWith(busy: false, error: error),
        (_) => state.copyWith(busy: false, outcome: AccountOutcome.signedOut),
      ),
    );
  }

  Future<void> _onBindingAnswered(
    UnlockBindingAnsweredEvent event,
    Emitter<UnlockState> emit,
  ) async {
    emit(state.copyWith(question: null, busy: true));
    final result = await _account.answerBinding(event.answer);
    emit(
      result.fold((error) => state.copyWith(busy: false, error: error), _after),
    );
  }

  UnlockState _after(
    AccountState account, {
    bool busy = false,
    bool clearApproval = false,
  }) {
    final base = state.copyWith(
      busy: busy,
      approval: clearApproval ? null : state.approval,
    );
    return switch (account) {
      AccountState_SignedIn() => base.copyWith(
        outcome: AccountOutcome.signedIn,
      ),
      AccountState_Question(:final field0) => base.copyWith(question: field0),
      AccountState_SignedOut() => base.copyWith(
        outcome: AccountOutcome.signedOut,
      ),
      AccountState_Locked() => base,
    };
  }
}
