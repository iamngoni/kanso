// Kanso — discovery_settings_bloc.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';
import 'package:kanso_ocr/kanso_ocr.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/repos/abs/preferences_repository.dart';
import '../../../discovery/models/repos/abs/discovery_repository.dart';

part 'discovery_settings_bloc.freezed.dart';
part 'discovery_settings_event.dart';
part 'discovery_settings_state.dart';

/// Coordinates Discovery preferences, live model status and local OCR work.
@injectable
class DiscoverySettingsBloc
    extends Bloc<DiscoverySettingsEvent, DiscoverySettingsState> {
  DiscoverySettingsBloc(this._discovery, this._preferences)
    : _reader = KansoOcr(),
      super(const DiscoverySettingsState()) {
    on<DiscoverySettingsStartedEvent>(_onStarted);
    on<RelatedSettingChangedEvent>(_onRelatedChanged);
    on<RelatedRetryRequestedEvent>(_onRetry);
    on<OcrSettingChangedEvent>(_onOcrChanged);
    on<OcrCountReceivedEvent>(
      (event, emit) => emit(state.copyWith(ocrWaiting: event.count)),
    );
    on<RelatedStatusReceivedEvent>(
      (event, emit) => emit(state.copyWith(related: event.status)),
    );
  }

  final DiscoveryRepository _discovery;
  final PreferencesRepository _preferences;
  final TextReader _reader;
  StreamSubscription<RelatedState>? _relatedSubscription;
  StreamSubscription<List<String>>? _changesSubscription;
  bool _reading = false;
  bool _ocrPending = false;

  Future<void> _onStarted(
    DiscoverySettingsStartedEvent event,
    Emitter<DiscoverySettingsState> emit,
  ) async {
    final enabled = await _discovery.relatedEnabled();
    final status = await _discovery.relatedState();
    final preferences = await _preferences.load();
    emit(
      state.copyWith(
        relatedEnabled: enabled.fold((_) => false, (value) => value),
        related: status.fold((_) => null, (value) => value),
        ocrEnabled: preferences.fold(
          (_) => false,
          (value) => value.searchImageText,
        ),
      ),
    );
    await _relatedSubscription?.cancel();
    await _changesSubscription?.cancel();
    _relatedSubscription = _discovery.relatedStream().listen(
      (value) => add(DiscoverySettingsEvent.relatedStatusReceived(value)),
    );
    _changesSubscription = _discovery.changes().listen((_) {
      if (!state.ocrEnabled) return;
      if (_reading) {
        _ocrPending = true;
      } else {
        unawaited(_drainOcr());
      }
    });
    if (state.ocrEnabled) {
      final waiting = await _discovery.ocrWaiting();
      emit(
        state.copyWith(ocrWaiting: waiting.fold((_) => 0, (value) => value)),
      );
      unawaited(_drainOcr());
    }
  }

  Future<void> _onRelatedChanged(
    RelatedSettingChangedEvent event,
    Emitter<DiscoverySettingsState> emit,
  ) async {
    emit(state.copyWith(relatedBusy: true));
    final result = await _discovery.setRelatedNotes(on: event.enabled);
    if (result.isRight()) {
      final status = await _discovery.relatedState();
      emit(
        state.copyWith(
          relatedEnabled: event.enabled,
          relatedBusy: false,
          related: status.fold((_) => state.related, (value) => value),
        ),
      );
    } else {
      emit(state.copyWith(relatedBusy: false));
    }
  }

  Future<void> _onRetry(
    RelatedRetryRequestedEvent event,
    Emitter<DiscoverySettingsState> emit,
  ) async {
    await _onRelatedChanged(
      const RelatedSettingChangedEvent(enabled: true),
      emit,
    );
  }

  Future<void> _onOcrChanged(
    OcrSettingChangedEvent event,
    Emitter<DiscoverySettingsState> emit,
  ) async {
    final saved = await _preferences.update(
      (current) => current.copyWith(searchImageText: event.enabled),
    );
    if (saved.isLeft()) {
      return;
    }
    emit(state.copyWith(ocrEnabled: event.enabled));
    if (event.enabled) {
      unawaited(_drainOcr());
    }
  }

  Future<void> _drainOcr() async {
    if (_reading) {
      _ocrPending = true;
      return;
    }
    if (!state.ocrEnabled) return;
    _reading = true;
    try {
      if (!await _reader.available) return;
      while (state.ocrEnabled) {
        final next = await _discovery.ocrNext();
        final image = next.fold((_) => null, (value) => value);
        if (image == null) break;
        var text = '';
        try {
          text = await _reader.read(image.bytes);
        } on Object {
          text = '';
        }
        if (!state.ocrEnabled) break;
        await _discovery.ocrKeep(
          id: image.id,
          md5: image.md5,
          text: text,
          engine: _reader.engine,
        );
        final waiting = await _discovery.ocrWaiting();
        add(
          DiscoverySettingsEvent.ocrCountReceived(
            waiting.fold((_) => 0, (value) => value),
          ),
        );
      }
      final waiting = await _discovery.ocrWaiting();
      add(
        DiscoverySettingsEvent.ocrCountReceived(
          waiting.fold((_) => 0, (value) => value),
        ),
      );
    } finally {
      _reading = false;
      if (_ocrPending && state.ocrEnabled) {
        _ocrPending = false;
        unawaited(_drainOcr());
      }
    }
  }

  @override
  Future<void> close() async {
    await _relatedSubscription?.cancel();
    await _changesSubscription?.cancel();
    _reading = false;
    return super.close();
  }
}
