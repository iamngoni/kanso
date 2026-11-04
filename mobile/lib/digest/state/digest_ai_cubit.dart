// Kanso — digest_ai_cubit.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../core/bridge/api/types.dart';
import '../../core/models/repos/abs/ai_repository.dart';
import '../../core/models/repos/abs/digest_repository.dart';

/// What the AI is doing for the open loops.
enum DigestAiWork {
  /// Nothing.
  idle,

  /// Reading the recent notes for loops the rules missed.
  finding,
}

/// What the AI parts of the digest pages show.
@immutable
class DigestAiState {
  const DigestAiState({
    this.ready = false,
    this.host = '',
    this.work = DigestAiWork.idle,
    this.found,
    this.picked = const <int>{},
    this.message,
    this.kept,
    this.empty,
  });

  /// Whether AI is on and has what it needs (a saved key, when the provider
  /// wants one). Until it is, none of the AI buttons show.
  final bool ready;

  /// Where the notes would be sent, as a name.
  final String host;
  final DigestAiWork work;

  /// What "Find more open loops" found, until the sheet is closed.
  final LoopScan? found;

  /// Which of [found]'s proposals are ticked.
  final Set<int> picked;

  /// Why the last request failed, or that it found nothing.
  final String? message;

  /// How many loops the last "Keep selected" kept.
  final int? kept;

  /// The last run, when it found nothing to propose (it may still say that
  /// some suggestions no longer match their notes).
  final LoopScan? empty;

  bool get busy => work != DigestAiWork.idle;

  DigestAiState copyWith({
    bool? ready,
    String? host,
    DigestAiWork? work,
    Object? found = _keep,
    Set<int>? picked,
    Object? message = _keep,
    Object? kept = _keep,
    Object? empty = _keep,
  }) => DigestAiState(
    ready: ready ?? this.ready,
    host: host ?? this.host,
    work: work ?? this.work,
    found: identical(found, _keep) ? this.found : found as LoopScan?,
    picked: picked ?? this.picked,
    message: identical(message, _keep) ? this.message : message as String?,
    kept: identical(kept, _keep) ? this.kept : kept as int?,
    empty: identical(empty, _keep) ? this.empty : empty as LoopScan?,
  );

  @override
  bool operator ==(Object other) =>
      other is DigestAiState &&
      other.ready == ready &&
      other.host == host &&
      other.work == work &&
      other.found == found &&
      setEquals(other.picked, picked) &&
      other.message == message &&
      other.kept == kept &&
      other.empty == empty;

  @override
  int get hashCode => Object.hash(
    ready,
    host,
    work,
    found,
    Object.hashAllUnordered(picked),
    message,
    kept,
    empty,
  );
}

const Object _keep = Object();

/// The AI part of the open loops: "Find more open loops…" (proposals the
/// person ticks and keeps). The calls run in the Rust core, off the UI isolate;
/// this holds what they return. Nothing is kept without the person's say.
class DigestAiCubit extends Cubit<DigestAiState> {
  DigestAiCubit(this._digest, this._ai) : super(const DigestAiState());

  final DigestRepository _digest;
  final AiRepository _ai;

  /// Reads whether AI is configured, and again when its settings change.
  StreamSubscription<Object?>? _watch;

  Future<void> load() async {
    await _refreshReady();
    _watch ??= _ai.settingsChanges().listen(
      (_) => unawaited(_refreshReady()),
    );
  }

  Future<void> _refreshReady() async {
    final settings = (await _ai.settings()).fold((_) => null, (it) => it);
    var ready = false;
    if (settings != null && settings.enabled) {
      final providers = (await _ai.providers()).fold(
        (_) => null,
        (it) => it,
      );
      final provider = providers
          ?.where((it) => it.key == settings.provider)
          .firstOrNull;
      ready = provider != null && (settings.hasKey || !provider.needsKey);
    }
    if (!isClosed) {
      emit(state.copyWith(ready: ready, host: settings?.host ?? ''));
    }
  }

  /// Reads the recent notes with the model. A run already going on stays.
  Future<void> findLoops() async {
    if (!state.ready || state.busy) {
      return;
    }
    emit(
      state.copyWith(
        work: DigestAiWork.finding,
        message: null,
        kept: null,
        empty: null,
      ),
    );
    final result = await _digest.findLoops();
    if (isClosed) {
      return;
    }
    result.fold(
      (error) => emit(
        state.copyWith(work: DigestAiWork.idle, message: error.message),
      ),
      (scan) => emit(
        state.copyWith(
          work: DigestAiWork.idle,
          found: scan.proposals.isEmpty ? null : scan,
          picked: const <int>{},
          empty: scan.proposals.isEmpty ? scan : null,
        ),
      ),
    );
  }

  /// Ticks or unticks proposal [index] of the sheet.
  void toggle(int index) {
    final next = <int>{...state.picked};
    if (!next.remove(index)) {
      next.add(index);
    }
    emit(state.copyWith(picked: next));
  }

  /// Closes the sheet without keeping anything.
  void discardFound() =>
      emit(state.copyWith(found: null, picked: const <int>{}));

  /// Keeps the ticked proposals.
  Future<void> keepSelected() async {
    final scan = state.found;
    if (scan == null) {
      return;
    }
    final picks = <LoopProposal>[
      for (final index in state.picked.toList()..sort())
        if (index < scan.proposals.length) scan.proposals[index],
    ];
    emit(state.copyWith(found: null, picked: const <int>{}));
    if (picks.isEmpty) {
      return;
    }
    final result = await _digest.keepLoops(picks);
    if (isClosed) {
      return;
    }
    result.fold(
      (error) => emit(state.copyWith(message: error.message)),
      (count) => emit(state.copyWith(kept: count)),
    );
  }

  /// Stops the request in flight.
  void stop() => _digest.stopAi();

  @override
  Future<void> close() async {
    await _watch?.cancel();
    _digest.stopAi();
    return super.close();
  }
}
