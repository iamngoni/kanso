// Kanso — ai_status_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/data/ai_settings.dart';
import '../../../core/models/repos/abs/ai_repository.dart';

part 'ai_status_bloc.freezed.dart';
part 'ai_status_event.dart';
part 'ai_status_state.dart';

/// Whether AI is on, for the places that offer it only then: the Ask row in
/// the library, the settings row and the assistant in the editor.
@injectable
class AiStatusBloc extends Bloc<AiStatusEvent, AiStatusState> {
  AiStatusBloc(this._ai) : super(const AiStatusState()) {
    on<AiStatusStartedEvent>(_onStarted, transformer: restartable());
    on<AiStatusSettingsChangedEvent>(
      (event, emit) =>
          emit(AiStatusState(loaded: true, settings: event.settings)),
    );
  }

  final AiRepository _ai;
  StreamSubscription<AiSettings>? _settingsChanges;

  Future<void> _onStarted(
    AiStatusStartedEvent event,
    Emitter<AiStatusState> emit,
  ) async {
    await _settingsChanges?.cancel();
    final first = await _ai.settings();
    if (emit.isDone) {
      return;
    }
    first.fold(
      (error) => emit(state.copyWith(loaded: true)),
      (settings) => emit(AiStatusState(loaded: true, settings: settings)),
    );
    _settingsChanges = _ai.settingsChanges().listen(
      (settings) => add(AiStatusEvent.settingsChanged(settings)),
    );
  }

  @override
  Future<void> close() async {
    await _settingsChanges?.cancel();
    return super.close();
  }
}
