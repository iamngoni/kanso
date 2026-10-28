// Kanso — dictation_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../models/data/dictation_capability.dart';
import '../../models/data/dictation_failure.dart';
import '../../models/data/dictation_permission.dart';
import '../../models/data/dictation_signal.dart';
import '../../models/data/dictation_status.dart';
import '../../models/data/dictation_unavailable_reason.dart';
import '../../models/repos/abs/dictation_repository.dart';
import '../../models/repos/abs/dictation_text_rules.dart';

part 'dictation_bloc.freezed.dart';
part 'dictation_event.dart';
part 'dictation_state.dart';

/// Dictation: the phone's recognizer, through the permission it needs, into
/// the phrases the person spoke.
///
/// A phrase is a segment once the recognizer finishes it; until then its
/// words are the partial. When the session ends with words not yet finished
/// (the recognizer was stopped, or failed), they are kept as the last
/// segment, so nothing said is lost.
@injectable
class DictationBloc extends Bloc<DictationEvent, DictationState> {
  DictationBloc(this.repository, this.rules) : super(const DictationState()) {
    on<DictationCheckedEvent>(onChecked, transformer: restartable());
    on<DictationStartedEvent>(onStarted, transformer: droppable());
    on<DictationStoppedEvent>(onStopped, transformer: sequential());
    on<DictationFinishRequestedEvent>(onFinishRequested);
    on<DictationCancelledEvent>(onCancelled, transformer: sequential());
    on<DictationSettingsRequestedEvent>(
      (event, emit) => repository.openSettings(),
      transformer: droppable(),
    );
    on<DictationSignalReceivedEvent>(onSignal, transformer: sequential());
    signals = repository.signals().listen(
      (signal) => add(DictationEvent.signalReceived(signal)),
    );
  }

  final DictationRepository repository;
  final DictationTextRules rules;
  late final StreamSubscription<DictationSignal> signals;

  /// Whether this BLoC started the session whose signals are coming. The
  /// recognizer is the phone's one; another screen's BLoC hears its signals
  /// too, and ignores them.
  bool inSession = false;

  /// Invalidates a start while availability or permission is being checked.
  int startGeneration = 0;

  /// Whether the person asked to be done while the session ran.
  bool finishing = false;

  Future<void> onChecked(
    DictationCheckedEvent event,
    Emitter<DictationState> emit,
  ) async {
    if (state.active) {
      return;
    }
    final answer = await repository.availability();
    answer.fold((error) {}, (capability) {
      if (!capability.available) {
        emit(unavailable(capability));
      } else if (blocked(capability.permission)) {
        emit(denied(capability.permission));
      } else if (state.status == DictationStatus.unavailable ||
          state.status == DictationStatus.denied) {
        // Allowed since the last look, in Settings.
        emit(const DictationState());
      }
    });
  }

  Future<void> onStarted(
    DictationStartedEvent event,
    Emitter<DictationState> emit,
  ) async {
    if (state.active) {
      return;
    }
    final generation = ++startGeneration;
    finishing = false;
    emit(
      DictationState(
        status: DictationStatus.requesting,
        showOverlay: event.showOverlay,
      ),
    );
    final answer = await repository.availability(localeTag: event.localeTag);
    if (generation != startGeneration || emit.isDone) {
      return;
    }
    final capability = answer.fold<DictationCapability?>((_) => null, (c) => c);
    if (capability == null) {
      emit(failed(DictationFailure.unknown, showOverlay: event.showOverlay));
      return;
    }
    if (!capability.available) {
      emit(unavailable(capability, showOverlay: event.showOverlay));
      return;
    }
    var permission = capability.permission;
    if (permission == DictationPermission.notDetermined) {
      final asked = await repository.requestPermission();
      if (generation != startGeneration || emit.isDone) {
        return;
      }
      permission = asked.fold(
        (_) => DictationPermission.denied,
        (granted) => granted,
      );
    }
    if (blocked(permission)) {
      emit(denied(permission, showOverlay: event.showOverlay));
      return;
    }
    if (permission != DictationPermission.granted) {
      emit(failed(DictationFailure.unknown, showOverlay: event.showOverlay));
      return;
    }
    inSession = true;
    final started = await repository.start(localeTag: event.localeTag);
    if (generation != startGeneration || emit.isDone) {
      inSession = false;
      await repository.cancel();
      return;
    }
    started.fold(
      (error) {
        inSession = false;
        emit(failed(DictationFailure.unknown, showOverlay: event.showOverlay));
      },
      (_) {
        // The listening signal moves it on.
      },
    );
  }

  Future<void> onStopped(
    DictationStoppedEvent event,
    Emitter<DictationState> emit,
  ) async {
    if (!state.active) {
      return;
    }
    await repository.stop();
  }

  Future<void> onFinishRequested(
    DictationFinishRequestedEvent event,
    Emitter<DictationState> emit,
  ) async {
    if (!state.active) {
      emit(state.copyWith(finished: true));
      return;
    }
    finishing = true;
    await repository.stop();
  }

  Future<void> onCancelled(
    DictationCancelledEvent event,
    Emitter<DictationState> emit,
  ) async {
    startGeneration++;
    if (!state.active) {
      emit(const DictationState());
      return;
    }
    inSession = false;
    emit(const DictationState());
    await repository.cancel();
  }

  Future<void> onSignal(
    DictationSignalReceivedEvent event,
    Emitter<DictationState> emit,
  ) async {
    if (!inSession) {
      return;
    }
    switch (event.signal) {
      case DictationListeningSignal():
        if (state.status == DictationStatus.requesting) {
          emit(state.copyWith(status: DictationStatus.listening));
        }
      case DictationPartialSignal(:final text):
        emit(
          await written(
            state.copyWith(status: DictationStatus.listening, partial: text),
          ),
        );
      case DictationFinalSignal(:final text):
        emit(
          await written(
            state.copyWith(
              status: DictationStatus.listening,
              segments: <String>[
                ...state.segments,
                if (text.trim().isNotEmpty) text.trim(),
              ],
              partial: '',
            ),
          ),
        );
      case DictationLevelSignal(:final level):
        if (state.status == DictationStatus.listening) {
          emit(state.copyWith(level: level));
        }
      case DictationFailedSignal(:final failure):
        emit(
          failure == DictationFailure.permissionDenied
              ? denied(DictationPermission.denied, keeping: state)
              : state.copyWith(
                  status: DictationStatus.error,
                  failure: failure,
                ),
        );
      case DictationEndedSignal():
        final heard = state.partial.trim();
        emit(
          await written(
            state.copyWith(
              status:
                  state.status == DictationStatus.error ||
                      state.status == DictationStatus.denied
                  ? state.status
                  : DictationStatus.idle,
              segments: <String>[
                ...state.segments,
                if (heard.isNotEmpty) heard,
              ],
              partial: '',
              level: 0,
              finished: finishing,
            ),
          ),
        );
        finishing = false;
        inSession = false;
    }
  }

  /// [next] with its words as the rules write them.
  Future<DictationState> written(DictationState next) async {
    Future<String> through(String transcript) async {
      final made = await rules.dictated(
        before: '',
        after: '',
        transcript: transcript,
      );
      return made?.text.trim() ?? '';
    }

    final finishedText = await through(next.segments.join(' '));
    return next.copyWith(
      finishedText: finishedText,
      text: next.partial.isEmpty
          ? finishedText
          : await through(next.transcript),
    );
  }

  /// Whether [permission] keeps dictation from listening.
  static bool blocked(DictationPermission permission) =>
      permission == DictationPermission.denied ||
      permission == DictationPermission.restricted;

  static DictationState unavailable(
    DictationCapability capability, {
    bool showOverlay = false,
  }) => DictationState(
    status: DictationStatus.unavailable,
    reason: capability.reason ?? DictationUnavailableReason.noRecognizer,
    showOverlay: showOverlay,
  );

  static DictationState denied(
    DictationPermission permission, {
    DictationState? keeping,
    bool showOverlay = false,
  }) => (keeping ?? const DictationState()).copyWith(
    status: DictationStatus.denied,
    restricted: permission == DictationPermission.restricted,
    showOverlay: keeping?.showOverlay ?? showOverlay,
  );

  static DictationState failed(
    DictationFailure failure, {
    bool showOverlay = false,
  }) => DictationState(
    status: DictationStatus.error,
    failure: failure,
    showOverlay: showOverlay,
  );

  @override
  Future<void> close() async {
    startGeneration++;
    // Not awaited: a stream that has already ended never answers a cancel.
    unawaited(signals.cancel());
    return super.close();
  }
}
