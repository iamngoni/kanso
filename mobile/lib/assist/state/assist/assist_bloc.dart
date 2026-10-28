// Kanso — assist_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/data/assist_action.dart';
import '../../../core/models/data/assist_action_info.dart';
import '../../../core/models/data/assist_update.dart';
import '../../../core/models/repos/abs/ai_repository.dart';
import '../../models/data/assist_status.dart';

part 'assist_bloc.freezed.dart';
part 'assist_event.dart';
part 'assist_state.dart';

/// The inline assistant and the note summary: one suggestion at a time,
/// written for text or a note, to be taken or thrown away. Nothing is
/// changed in a note by this; the sheet applies an accepted suggestion
/// through the editor.
@injectable
class AssistBloc extends Bloc<AssistEvent, AssistState> {
  AssistBloc(this._ai) : super(const AssistState()) {
    on<AssistRequestedEvent>(_onRequested, transformer: restartable());
    on<AssistSummaryRequestedEvent>(_onSummary, transformer: restartable());
    on<AssistUpdateReceivedEvent>(
      (event, emit) => emit(_applied(event.update)),
      transformer: sequential(),
    );
    on<AssistStreamEndedEvent>(_onStreamEnded);
    on<AssistStoppedEvent>(_onStopped);
    on<AssistChangeNoticedEvent>(
      (event, emit) => emit(state.copyWith(changed: true)),
    );
    on<AssistDiscardedEvent>(_onDiscarded);
  }

  final AiRepository _ai;
  StreamSubscription<AssistUpdate>? _updates;

  /// The assistant's actions as the core lists them.
  List<AssistActionInfo> get actions => _ai.assistActions();

  Future<void> _onRequested(
    AssistRequestedEvent event,
    Emitter<AssistState> emit,
  ) async {
    emit(AssistState(status: AssistStatus.running, action: event.action));
    await _listen(
      _ai.assist(
        noteId: event.noteId,
        action: event.action,
        before: event.before,
        selection: event.selection,
      ),
    );
  }

  Future<void> _onSummary(
    AssistSummaryRequestedEvent event,
    Emitter<AssistState> emit,
  ) async {
    emit(
      const AssistState(
        status: AssistStatus.running,
        action: AssistAction.summarize,
        wholeNote: true,
      ),
    );
    await _listen(_ai.summarize(event.noteId));
  }

  Future<void> _listen(Stream<AssistUpdate> updates) async {
    await _updates?.cancel();
    _updates = updates.listen(
      (update) => add(AssistEvent.updateReceived(update)),
      onDone: () => add(const AssistEvent.streamEnded()),
    );
  }

  AssistState _applied(AssistUpdate update) {
    if (state.status != AssistStatus.running) {
      return state;
    }
    final next = switch (update) {
      AssistText(:final text) => state.copyWith(suggestion: text),
      AssistDone(:final suggestion) => state.copyWith(
        status: suggestion.trim().isEmpty
            ? AssistStatus.failed
            : AssistStatus.ready,
        suggestion: suggestion.trim(),
      ),
      AssistKeptOut() => state.copyWith(status: AssistStatus.keptOut),
      AssistFailed(:final message) => state.copyWith(
        status: AssistStatus.failed,
        failure: message,
      ),
    };
    if (next.status != AssistStatus.running) {
      unawaited(_updates?.cancel());
      _updates = null;
    }
    return next;
  }

  void _onStreamEnded(AssistStreamEndedEvent event, Emitter<AssistState> emit) {
    if (state.status != AssistStatus.running) {
      return;
    }
    unawaited(_updates?.cancel());
    _updates = null;
    // As on the desktop: the model ended without saying it was done.
    emit(state.copyWith(status: AssistStatus.failed));
  }

  /// What arrived stays. A rewrite stopped part way cannot be accepted (it
  /// would leave the text half changed); text to go after the selection can.
  Future<void> _onStopped(
    AssistStoppedEvent event,
    Emitter<AssistState> emit,
  ) async {
    if (state.status != AssistStatus.running) {
      return;
    }
    await _updates?.cancel();
    _updates = null;
    emit(state.copyWith(status: AssistStatus.stopped));
  }

  Future<void> _onDiscarded(
    AssistDiscardedEvent event,
    Emitter<AssistState> emit,
  ) async {
    await _updates?.cancel();
    _updates = null;
    emit(const AssistState());
  }

  @override
  Future<void> close() async {
    await _updates?.cancel();
    return super.close();
  }
}
