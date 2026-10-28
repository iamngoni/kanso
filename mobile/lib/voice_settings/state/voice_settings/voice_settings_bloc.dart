// Kanso — voice_settings_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/data/load_status.dart';
import '../../../dictation/models/data/dictation_capability.dart';
import '../../../dictation/models/repos/abs/dictation_repository.dart';
import '../../models/data/voice_settings.dart';
import '../../models/repos/abs/voice_settings_repository.dart';

part 'voice_settings_bloc.freezed.dart';
part 'voice_settings_event.dart';
part 'voice_settings_state.dart';

/// Settings › Voice: the choices for voice input, and what the phone's
/// recognizer can do now.
@injectable
class VoiceSettingsBloc extends Bloc<VoiceSettingsEvent, VoiceSettingsState> {
  VoiceSettingsBloc(this.settings, this.dictation)
    : super(const VoiceSettingsState()) {
    on<VoiceSettingsStartedEvent>(onStarted, transformer: restartable());
    on<VoiceSettingsSpokenCommandsChangedEvent>(
      onSpokenCommandsChanged,
      transformer: sequential(),
    );
    on<VoiceSettingsSystemSettingsRequestedEvent>(
      (event, emit) => dictation.openSettings(),
      transformer: droppable(),
    );
  }

  final VoiceSettingsRepository settings;
  final DictationRepository dictation;

  Future<void> onStarted(
    VoiceSettingsStartedEvent event,
    Emitter<VoiceSettingsState> emit,
  ) async {
    final read = await settings.settings();
    final capability = await dictation.availability();
    final heard = capability.fold<DictationCapability?>((_) => null, (c) => c);
    read.fold(
      (error) => emit(
        state.copyWith(status: LoadStatus.failed, capability: heard),
      ),
      (value) => emit(
        state.copyWith(
          status: LoadStatus.ready,
          settings: value,
          capability: heard,
        ),
      ),
    );
  }

  Future<void> onSpokenCommandsChanged(
    VoiceSettingsSpokenCommandsChangedEvent event,
    Emitter<VoiceSettingsState> emit,
  ) async {
    final before = state.settings;
    final wanted = before.copyWith(spokenCommands: event.on);
    // The switch moves at once; it moves back if the choice was not kept.
    emit(state.copyWith(settings: wanted));
    final saved = await settings.save(wanted);
    saved.fold(
      (error) => emit(state.copyWith(settings: before)),
      (kept) => emit(state.copyWith(settings: kept)),
    );
  }
}
