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
import '../../../core/models/data/ask_receipt.dart';
import '../../../core/models/data/ask_scope.dart';
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
    // What the scope, the time range and saving do touch one another and the
    // newest turn, so they are taken one at a time, in order.
    on<AskScopeChosenEvent>(
      (event, emit) => emit(state.copyWith(scope: event.scope)),
      transformer: sequential(),
    );
    on<AskScopesRequestedEvent>(_onScopesRequested, transformer: sequential());
    on<AskTimeDroppedEvent>(_onTimeDropped, transformer: sequential());
    on<AskSearchedAllEvent>(_onSearchedAll, transformer: sequential());
    on<AskAnswerSavedEvent>(_onAnswerSaved, transformer: sequential());
  }

  final AiRepository _ai;
  StreamSubscription<AiSettings>? _settingsChanges;
  StreamSubscription<AskUpdate>? _updates;

  Future<void> _onStarted(
    AskStartedEvent event,
    Emitter<AskState> emit,
  ) async {
    await _settingsChanges?.cancel();
    final scope = event.scope ?? AskScope.everything;
    emit(AskState(scope: scope));
    final result = await _ai.settings();
    if (emit.isDone) {
      return;
    }
    final settings = result.toOption().fold(() => null, (found) => found);
    if (settings == null) {
      emit(AskState(status: LoadStatus.failed, scope: scope));
      return;
    }
    emit(
      AskState(
        status: LoadStatus.ready,
        aiEnabled: settings.enabled,
        privacy: settings.privacy,
        scope: scope,
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
    await _begin(event.question.trim(), emit);
  }

  /// Starts [question] as the next turn, over [AskState.scope].
  Future<void> _begin(
    String question,
    Emitter<AskState> emit, {
    bool ignoreTime = false,
  }) async {
    if (question.isEmpty || state.busy || !state.aiEnabled) {
      return;
    }
    emit(
      state.copyWith(
        phase: AskPhase.searching,
        saveFailure: null,
        turns: <AskTurn>[
          ...state.turns,
          AskTurn(question: question, scope: state.scope),
        ],
      ),
    );
    await _updates?.cancel();
    _updates = _ai
        .ask(question, scope: state.scope, ignoreTime: ignoreTime)
        .listen(
          (update) => add(AskEvent.updateReceived(update)),
          onDone: () => add(const AskEvent.streamEnded()),
        );
  }

  Future<void> _onScopesRequested(
    AskScopesRequestedEvent event,
    Emitter<AskState> emit,
  ) async {
    final result = await _ai.askScopes();
    if (emit.isDone) {
      return;
    }
    result.fold((_) {}, (scopes) => emit(state.copyWith(scopes: scopes)));
  }

  /// Asks the newest question again without its time range.
  Future<void> _onTimeDropped(
    AskTimeDroppedEvent event,
    Emitter<AskState> emit,
  ) async {
    final last = state.turns.lastOrNull;
    if (last == null || last.time == null || state.busy) {
      return;
    }
    await _ai.forgetLast();
    emit(
      state.copyWith(turns: state.turns.take(state.turns.length - 1).toList()),
    );
    await _begin(last.question, emit, ignoreTime: true);
  }

  /// Asks the newest question again over all notes.
  Future<void> _onSearchedAll(
    AskSearchedAllEvent event,
    Emitter<AskState> emit,
  ) async {
    final last = state.turns.lastOrNull;
    if (last == null || !last.narrow || state.busy) {
      return;
    }
    emit(
      state.copyWith(
        scope: AskScope.everything,
        turns: state.turns.take(state.turns.length - 1).toList(),
      ),
    );
    await _begin(last.question, emit);
  }

  /// Saves the newest answer as a note: the one write Ask makes.
  Future<void> _onAnswerSaved(
    AskAnswerSavedEvent event,
    Emitter<AskState> emit,
  ) async {
    final last = state.turns.lastOrNull;
    if (last == null ||
        last.outcome != AskOutcome.answered ||
        last.savedNoteId != null ||
        state.busy) {
      return;
    }
    final result = await _ai.saveLastAnswer();
    if (emit.isDone) {
      return;
    }
    result.fold(
      (error) => emit(state.copyWith(saveFailure: error.message)),
      (id) => emit(
        state.copyWith(
          saveFailure: null,
          turns: <AskTurn>[
            ...state.turns.take(state.turns.length - 1),
            last.copyWith(savedNoteId: id),
          ],
        ),
      ),
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
      AskTime(:final time) => (turn.copyWith(time: time), state.phase),
      AskRestarted(:final line) => (
        turn.copyWith(notice: line),
        state.phase,
      ),
      AskDone done => (
        turn.copyWith(
          answer: done.answer,
          used: _withNew(turn.used, done.used),
          receipts: done.receipts,
          alsoRead: done.alsoRead,
          banner: done.banner,
          unverified: done.unverified,
          noSource: done.noSource,
          neverWrote: done.neverWrote,
          closest: done.closest,
          time: done.time ?? turn.time,
          outcome: AskOutcome.answered,
        ),
        AskPhase.idle,
      ),
      AskNothingMatches(:final narrow) => (
        turn.copyWith(outcome: AskOutcome.nothingMatches, narrow: narrow),
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
    // A conversation that was ended starts over with the question asked.
    final earlier = update is AskRestarted
        ? const <AskTurn>[]
        : state.turns.take(state.turns.length - 1);
    return state.copyWith(
      phase: phase,
      turns: <AskTurn>[...earlier, next],
    );
  }

  /// Where the lines [receipt] pointed at are now: [receipt] itself when they
  /// have not moved, the receipt with its new lines when they have, `null`
  /// when they are gone.
  Future<AskReceipt?> relocated(AskReceipt receipt) async {
    final found = await _ai.relocateReceipt(receipt);
    return found.fold(
      (_) => receipt,
      (lines) => lines == null
          ? null
          : receipt.copyWith(start: lines.start, end: lines.end),
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
      await _ai.stop();
    }
    await _ai.clear();
    return super.close();
  }
}
