// Kanso — device_approval_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../bridge/api/types.dart';
import '../../models/data/application_error.dart';
import '../../models/repos/abs/approvals_repository.dart';

part 'device_approval_bloc.freezed.dart';
part 'device_approval_event.dart';
part 'device_approval_state.dart';

/// The devices asking to join the account, for this phone to approve or not.
///
/// It watches the requests while an account is signed in and the app is in
/// front, and nothing otherwise: a phone in a pocket does not hold a socket
/// open. The library challenges a request when it first shows, so a request
/// is ready to be answered once it has a name and a code.
@injectable
class DeviceApprovalBloc
    extends Bloc<DeviceApprovalEvent, DeviceApprovalState> {
  DeviceApprovalBloc(this._approvals) : super(const DeviceApprovalState()) {
    on<DeviceApprovalConditionsChangedEvent>(
      _onConditionsChanged,
      transformer: restartable(),
    );
    on<DeviceApprovalApprovedEvent>(_onApproved, transformer: droppable());
    on<DeviceApprovalDeclinedEvent>(_onDeclined, transformer: droppable());
  }

  final ApprovalsRepository _approvals;

  Future<void> _onConditionsChanged(
    DeviceApprovalConditionsChangedEvent event,
    Emitter<DeviceApprovalState> emit,
  ) async {
    if (!event.signedIn || !event.foreground) {
      emit(const DeviceApprovalState());
      return;
    }
    await emit.onEach<List<ApprovalPrompt>>(
      _approvals.watch(),
      onData: (prompts) => emit(state.copyWith(prompts: prompts)),
    );
  }

  Future<void> _onApproved(
    DeviceApprovalApprovedEvent event,
    Emitter<DeviceApprovalState> emit,
  ) async {
    // A request that changed after it was made is never approved.
    final prompt = state.prompts.where((p) => p.id == event.id).firstOrNull;
    if (prompt == null || prompt.state != PromptState.ready) {
      return;
    }
    emit(state.copyWith(answering: event.id, error: null));
    final result = await _approvals.approve(event.id);
    emit(
      result.fold(
        (error) => state.copyWith(answering: null, error: error),
        (prompts) => state.copyWith(answering: null, prompts: prompts),
      ),
    );
  }

  Future<void> _onDeclined(
    DeviceApprovalDeclinedEvent event,
    Emitter<DeviceApprovalState> emit,
  ) async {
    emit(state.copyWith(answering: event.id, error: null));
    final result = await _approvals.decline(event.id);
    emit(
      result.fold(
        (error) => state.copyWith(answering: null, error: error),
        (prompts) => state.copyWith(answering: null, prompts: prompts),
      ),
    );
  }
}
