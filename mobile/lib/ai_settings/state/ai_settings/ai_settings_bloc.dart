// Kanso — ai_settings_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:dartz/dartz.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/data/ai_provider_info.dart';
import '../../../core/models/data/ai_settings.dart';
import '../../../core/models/data/ai_settings_change.dart';
import '../../../core/models/data/ai_test_result.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/ai_repository.dart';
import '../../../core/utils/debounce.dart';
import '../../models/data/ai_key.dart';
import '../../models/data/ai_test_status.dart';

part 'ai_settings_bloc.freezed.dart';
part 'ai_settings_event.dart';
part 'ai_settings_state.dart';

/// How long the person may pause typing an address, a model or a region
/// before it is saved.
const Duration aiFieldsDebounce = Duration(milliseconds: 500);

/// Settings › AI: whether AI is on, where answers come from, the key, and
/// the Test button. Every change is saved as it is made; the key goes to the
/// secret store and is never kept here or read back.
@injectable
class AiSettingsBloc extends Bloc<AiSettingsEvent, AiSettingsState> {
  AiSettingsBloc(this._ai) : super(const AiSettingsState()) {
    on<AiSettingsStartedEvent>(_onStarted, transformer: restartable());
    on<AiSettingsEnabledChangedEvent>(
      (event, emit) => _save(
        emit,
        AiSettingsChange(
          enabled: event.enabled,
          provider: state.settings.provider,
          baseUrl: state.settings.baseUrl,
          model: state.settings.model,
          region: state.settings.region,
        ),
      ),
      transformer: sequential(),
    );
    on<AiSettingsProviderChosenEvent>(
      // A different provider starts from its own address, model and region.
      (event, emit) => _save(
        emit,
        AiSettingsChange(
          enabled: state.settings.enabled,
          provider: event.provider,
        ),
      ),
      transformer: sequential(),
    );
    on<AiSettingsFieldsChangedEvent>(
      (event, emit) => _save(emit, _changeFor(event.fields)),
      transformer: debounceRestartable<AiSettingsFieldsChangedEvent>(
        aiFieldsDebounce,
      ),
    );
    on<AiSettingsKeySavedEvent>(_onKeySaved, transformer: sequential());
    on<AiSettingsKeyRemovedEvent>(_onKeyRemoved, transformer: sequential());
    on<AiSettingsTestRequestedEvent>(_onTest, transformer: droppable());
  }

  final AiRepository _ai;

  AiSettingsChange _changeFor(AiFields fields) => AiSettingsChange(
    enabled: state.settings.enabled,
    provider: state.settings.provider,
    baseUrl: fields.address.trim(),
    model: fields.model.trim(),
    region: fields.region.trim(),
  );

  Future<void> _onStarted(
    AiSettingsStartedEvent event,
    Emitter<AiSettingsState> emit,
  ) async {
    emit(const AiSettingsState());
    final settings = await _ai.settings();
    final providers = await _ai.providers();
    final failed = settings.isLeft() || providers.isLeft();
    if (failed) {
      emit(const AiSettingsState(status: LoadStatus.failed));
      return;
    }
    emit(
      AiSettingsState(
        status: LoadStatus.ready,
        settings: settings.getOrElse(() => const AiSettings()),
        providers: providers.getOrElse(() => const <AiProviderInfo>[]),
      ),
    );
  }

  /// Saves [change]; what was found by the last Test no longer holds.
  Future<void> _save(
    Emitter<AiSettingsState> emit,
    AiSettingsChange change,
  ) async {
    final result = await _ai.saveSettings(change);
    emit(
      result.fold(
        (error) => state.copyWith(error: error),
        (settings) => state.copyWith(
          settings: settings,
          error: null,
          test: AiTestStatus.idle,
          testMessage: '',
        ),
      ),
    );
  }

  Future<void> _onKeySaved(
    AiSettingsKeySavedEvent event,
    Emitter<AiSettingsState> emit,
  ) async {
    emit(state.copyWith(keyBusy: true));
    final result = await _ai.setKey(
      provider: state.settings.provider,
      key: event.key.value.trim(),
    );
    emit(_afterKey(result));
  }

  Future<void> _onKeyRemoved(
    AiSettingsKeyRemovedEvent event,
    Emitter<AiSettingsState> emit,
  ) async {
    emit(state.copyWith(keyBusy: true));
    final result = await _ai.setKey(
      provider: state.settings.provider,
      key: '',
    );
    emit(_afterKey(result));
  }

  AiSettingsState _afterKey(Either<ApplicationError, AiSettings> result) {
    return result.fold(
      (error) => state.copyWith(keyBusy: false, error: error),
      (settings) => state.copyWith(
        keyBusy: false,
        settings: settings,
        error: null,
        test: AiTestStatus.idle,
        testMessage: '',
      ),
    );
  }

  Future<void> _onTest(
    AiSettingsTestRequestedEvent event,
    Emitter<AiSettingsState> emit,
  ) async {
    emit(state.copyWith(test: AiTestStatus.running, testMessage: ''));
    // What is typed is what is tested, even if it was not yet saved.
    final saved = await _ai.saveSettings(_changeFor(event.fields));
    final failure = saved.fold<ApplicationError?>(
      (error) => error,
      (_) => null,
    );
    if (failure != null) {
      emit(
        state.copyWith(test: AiTestStatus.failed, testMessage: failure.message),
      );
      return;
    }
    emit(
      state.copyWith(
        settings: saved.getOrElse(() => state.settings),
      ),
    );
    final result = await _ai.test();
    emit(
      result.fold(
        (error) => state.copyWith(
          test: AiTestStatus.failed,
          testMessage: error.message,
        ),
        (found) => switch (found) {
          AiTestOk(:final said) => state.copyWith(
            test: AiTestStatus.ok,
            testMessage: said,
          ),
          AiTestFailed(:final message) => state.copyWith(
            test: AiTestStatus.failed,
            testMessage: message,
          ),
        },
      ),
    );
  }
}
