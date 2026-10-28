// Kanso — ask_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/data/ai_note_ref.dart';
import '../../../core/models/data/ai_settings.dart';
import '../../../core/models/data/ask_update.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/ai_repository.dart';
import '../../models/data/ask_outcome.dart';
import '../../models/data/ask_phase.dart';
import '../../models/data/ask_turn.dart';

part 'ask_bloc.freezed.dart';
part 'ask_event.dart';
part 'ask_state.dart';

/// Ask your notes: a conversation of questions answered from the person's
/// own notes. Nothing of it is saved. A question being answered ends when
/// it is stopped, when the conversation is cleared and when the screen
/// closes.
@injectable
class AskBloc extends Bloc<AskEvent, AskState> {
  AskBloc(this._ai) : super(const AskState()) {
    on<AskStartedEvent>(_onStarted, transformer: restartable());
    on<AskSettingsChangedEvent>(
      (event, emit) => emit(
        state.copyWith(
          aiEnabled: event.settings.enabled,
          privacy: event.settings.privacy,
        ),
      ),
    );
    on<AskQuestionAskedEvent>(_onAsked, transformer: droppable());
    on<AskUpdateReceivedEvent>(
      (event, emit) => emit(_applied(event.update)),
      transformer: sequential(),
    );
    on<AskStreamEndedEvent>(_onStreamEnded);
    on<AskStoppedEvent>(_onStopped, transformer: droppable());
    on<AskClearedEvent>(_onCleared);
    on<AskRetriedEvent>(_onRetried);
  }

  final AiRepository _ai;
  StreamSubscription<AiSettings>? _settingsChanges;
  StreamSubscription<AskUpdate>? _updates;

  Future<void> _onStarted(
    AskStartedEvent event,
    Emitter<AskState> emit,
  ) async {
    await _settingsChanges?.cancel();
    emit(const AskState());
    final result = await _ai.settings();
    if (emit.isDone) {
      return;
    }
    final settings = result.toOption().fold(() => null, (found) => found);
    if (settings == null) {
      emit(const AskState(status: LoadStatus.failed));
      return;
    }
    emit(
      AskState(
        status: LoadStatus.ready,
        aiEnabled: settings.enabled,
        privacy: settings.privacy,
      ),
    );
    // AI turned on from here, in Settings, is seen on coming back.
    _settingsChanges = _ai.settingsChanges().listen(
      (settings) => add(AskEvent.settingsChanged(settings)),
    );
  }

  Future<void> _onAsked(
    AskQuestionAskedEvent event,
    Emitter<AskState> emit,
  ) async {
    final question = event.question.trim();
    if (question.isEmpty || state.busy || !state.aiEnabled) {
      return;
    }
    emit(
      state.copyWith(
        phase: AskPhase.searching,
        turns: <AskTurn>[
          ...state.turns,
          AskTurn(question: question),
        ],
      ),
    );
    await _updates?.cancel();
    _updates = _ai
        .ask(question)
        .listen(
          (update) => add(AskEvent.updateReceived(update)),
          onDone: () => add(const AskEvent.streamEnded()),
        );
  }

  /// [state] with [update] of the question being answered applied.
  AskState _applied(AskUpdate update) {
    if (state.turns.isEmpty || !state.busy) {
      return state;
    }
    final turn = state.turns.last;
    final (AskTurn next, AskPhase phase) = switch (update) {
      AskSearching() => (turn, AskPhase.searching),
      AskThinking() => (turn, AskPhase.thinking),
      AskUsedNotes(:final notes) => (
        turn.copyWith(used: _withNew(turn.used, notes)),
        state.phase,
      ),
      AskAnswer(:final text) => (
        turn.copyWith(answer: text),
        AskPhase.answering,
      ),
      AskDone(:final answer, :final used) => (
        turn.copyWith(
          answer: answer,
          used: _withNew(turn.used, used),
          outcome: AskOutcome.answered,
        ),
        AskPhase.idle,
      ),
      AskNothingMatches() => (
        turn.copyWith(outcome: AskOutcome.nothingMatches),
        AskPhase.idle,
      ),
      AskCutOff() => (
        turn.copyWith(outcome: AskOutcome.cutOff),
        AskPhase.idle,
      ),
      AskFailed(:final message) => (
        turn.copyWith(outcome: AskOutcome.failed, failure: message),
        AskPhase.idle,
      ),
    };
    if (phase == AskPhase.idle) {
      unawaited(_updates?.cancel());
      _updates = null;
    }
    return state.copyWith(
      phase: phase,
      turns: <AskTurn>[...state.turns.take(state.turns.length - 1), next],
    );
  }

  /// [known] with the notes of [more] it does not have yet, in order.
  List<AiNoteRef> _withNew(List<AiNoteRef> known, List<AiNoteRef> more) {
    return <AiNoteRef>[
      ...known,
      for (final note in more)
        if (!known.any((seen) => seen.id == note.id)) note,
    ];
  }

  /// The updates ended without saying how: an answer that was written is
  /// done, and with none the question failed.
  void _onStreamEnded(AskStreamEndedEvent event, Emitter<AskState> emit) {
    if (!state.busy) {
      return;
    }
    final turn = state.turns.last;
    final ended = turn.answer.isEmpty
        ? turn.copyWith(outcome: AskOutcome.failed)
        : turn.copyWith(outcome: AskOutcome.answered);
    unawaited(_updates?.cancel());
    _updates = null;
    emit(
      state.copyWith(
        phase: AskPhase.idle,
        turns: <AskTurn>[...state.turns.take(state.turns.length - 1), ended],
      ),
    );
  }

  Future<void> _onStopped(
    AskStoppedEvent event,
    Emitter<AskState> emit,
  ) async {
    if (!state.busy) {
      return;
    }
    await _updates?.cancel();
    _updates = null;
    final turn = state.turns.last;
    emit(
      state.copyWith(
        phase: AskPhase.idle,
        turns: <AskTurn>[
          ...state.turns.take(state.turns.length - 1),
          turn.copyWith(outcome: AskOutcome.stopped),
        ],
      ),
    );
    await _ai.stop();
  }

  Future<void> _onCleared(
    AskClearedEvent event,
    Emitter<AskState> emit,
  ) async {
    await _updates?.cancel();
    _updates = null;
    emit(state.copyWith(phase: AskPhase.idle, turns: const <AskTurn>[]));
    await _ai.clear();
  }

  /// Asks the last question again after it failed: the failed turn is
  /// replaced.
  void _onRetried(AskRetriedEvent event, Emitter<AskState> emit) {
    final last = state.turns.lastOrNull;
    if (last == null || last.outcome != AskOutcome.failed || state.busy) {
      return;
    }
    emit(
      state.copyWith(turns: state.turns.take(state.turns.length - 1).toList()),
    );
    add(AskEvent.questionAsked(last.question));
  }

  @override
  Future<void> close() async {
    await _settingsChanges?.cancel();
    await _updates?.cancel();
    if (state.busy) {
      unawaited(_ai.stop());
    }
    return super.close();
  }
}
