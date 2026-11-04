// Kanso — player_bloc.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/player_types.dart';
import '../../models/repos/abs/player_repository.dart';

part 'player_bloc.freezed.dart';
part 'player_event.dart';
part 'player_state.dart';

/// How often the card asks the player where it is, while it plays.
const Duration playerTick = Duration(milliseconds: 100);

/// Every this many ticks, a card waiting for bytes looks for them: two
/// seconds.
const int _recheckEvery = 20;

/// The recording player of one note (issue #38, step 5): the Rust core's
/// player behind a card in the note's preview. It finds the recording the
/// note holds, opens the speaker on the first Play (not when the note
/// opens), and reads the player's atomic position about ten times a second
/// while it plays, and not at all while it is paused.
///
/// One per open note: leaving the note closes it, which stops the sound. A
/// tap on a timestamp seeks the recording to that moment and plays.
///
/// Every event runs in turn, so a tap on a timestamp never races the
/// player still opening for the Play before it.
@injectable
class PlayerBloc extends Bloc<PlayerEvent, PlayerState> {
  PlayerBloc(
    this._repository, {
    // Named and public so tests can slow it down; not injected (no Duration
    // is registered, and asking for one throws).
    @ignoreParam Duration tick = playerTick,
    // ignore: prefer_initializing_formals
  }) : _tick = tick,
       super(const PlayerState()) {
    on<PlayerEvent>(_onEvent, transformer: sequential());
  }

  final PlayerRepository _repository;
  final Duration _tick;
  PlaybackSession? _session;
  Timer? _timer;
  int _ticks = 0;

  /// The volume before it was muted.
  double _unmuted = 1;

  Future<void> _onEvent(PlayerEvent event, Emitter<PlayerState> emit) async {
    switch (event) {
      case PlayerStartedEvent(:final noteId):
        emit(state.copyWith(noteId: noteId));
        await _refresh(emit);
      case PlayerRefreshedEvent():
        await _refresh(emit);
      case PlayerToggledEvent():
        await _toggle(emit);
      case PlayerSeekedEvent(:final ms):
        await _seek(ms, emit);
      case PlayerSkippedEvent(:final deltaMs):
        _session?.skipMs(deltaMs);
        _read(emit);
      case PlayerSpeedCycledEvent():
        _setSpeed(_nextSpeed(state.speed), emit);
      case PlayerVolumeChangedEvent(:final volume):
        _setVolume(volume, emit);
      case PlayerMuteToggledEvent():
        _setVolume(state.volume > 0 ? 0 : _unmuted, emit);
      case PlayerMomentTappedEvent(:final fileId, :final seconds):
        await _moment(fileId, seconds, emit);
      case PlayerRetriedEvent():
        _close();
        emit(state.copyWith(phase: PlayerPhase.idle, message: null));
        await _toggle(emit);
      case PlayerTickedEvent():
        await _onTick(emit);
    }
  }

  /// Looks at the note again: which recording it holds and whether its bytes
  /// are here. The same recording keeps its player.
  Future<void> _refresh(Emitter<PlayerState> emit) async {
    if (state.noteId.isEmpty) {
      return;
    }
    final found = await _repository.noteRecording(state.noteId);
    if (emit.isDone) {
      return;
    }
    final recording = found.fold((_) => null, (recording) => recording);
    if (recording == null) {
      _close();
      emit(PlayerState(noteId: state.noteId, speed: state.speed));
      return;
    }
    final same =
        _session != null && _sameParts(recording.fileIds, state.fileIds);
    if (same) {
      emit(
        state.copyWith(
          lengthMs: recording.lengthMs ?? state.lengthMs,
          phase: _phaseOf(recording.state, ready: true),
        ),
      );
      return;
    }
    _close();
    emit(
      state.copyWith(
        fileIds: recording.fileIds,
        lengthMs: recording.lengthMs ?? 0,
        positionMs: 0,
        playing: false,
        message: null,
        phase: _phaseOf(recording.state, ready: false),
      ),
    );
    _startTimerIfNeeded();
  }

  PlayerPhase _phaseOf(PartsState parts, {required bool ready}) =>
      switch (parts) {
        PartsState.ready => ready ? PlayerPhase.ready : PlayerPhase.idle,
        PartsState.notHere => PlayerPhase.notHere,
        PartsState.missing => PlayerPhase.missing,
      };

  bool _sameParts(List<String> a, List<String> b) {
    if (a.length != b.length) {
      return false;
    }
    for (var i = 0; i < a.length; i++) {
      if (a[i] != b[i]) {
        return false;
      }
    }
    return true;
  }

  /// Opens the speaker, then does `then` to it. Not while it is opening.
  Future<bool> _open(Emitter<PlayerState> emit) async {
    if (_session != null) {
      return true;
    }
    if (state.fileIds.isEmpty) {
      return false;
    }
    if (_repository.partsState(state.fileIds) != PartsState.ready) {
      await _refresh(emit);
      return false;
    }
    emit(state.copyWith(phase: PlayerPhase.opening, message: null));
    final opened = await _repository.open(state.fileIds);
    if (emit.isDone) {
      opened.fold((_) {}, (session) => session.dispose());
      return false;
    }
    return opened.fold(
      (error) {
        emit(
          state.copyWith(
            phase: PlayerPhase.failed,
            message: error.message,
          ),
        );
        return false;
      },
      (session) {
        _session = session;
        session
          ..setSpeed(state.speed)
          ..setVolume(state.volume);
        final status = session.status;
        emit(
          state.copyWith(
            phase: PlayerPhase.ready,
            lengthMs: status.lengthMs,
            positionMs: status.positionMs,
          ),
        );
        return true;
      },
    );
  }

  Future<void> _toggle(Emitter<PlayerState> emit) async {
    if (!await _open(emit)) {
      return;
    }
    _session?.toggle();
    _read(emit);
    _startTimerIfNeeded();
  }

  Future<void> _seek(int ms, Emitter<PlayerState> emit) async {
    if (!await _open(emit)) {
      return;
    }
    _session?.seekMs(ms);
    _read(emit);
  }

  Future<void> _moment(
    String fileId,
    int seconds,
    Emitter<PlayerState> emit,
  ) async {
    if (!state.fileIds.contains(fileId)) {
      // The text may be ahead of the card (a note just synced).
      await _refresh(emit);
      if (!state.fileIds.contains(fileId)) {
        return;
      }
    }
    // The card comes into view whatever it says.
    emit(state.copyWith(reveals: state.reveals + 1));
    if (!await _open(emit)) {
      return;
    }
    _session
      ?..seekMs(seconds * 1000)
      ..play();
    _read(emit);
    _startTimerIfNeeded();
  }

  void _setSpeed(double speed, Emitter<PlayerState> emit) {
    _session?.setSpeed(speed);
    emit(state.copyWith(speed: speed));
  }

  void _setVolume(double volume, Emitter<PlayerState> emit) {
    final v = volume.clamp(0.0, 1.0);
    if (v > 0) {
      _unmuted = v;
    }
    _session?.setVolume(v);
    emit(state.copyWith(volume: v));
  }

  /// Takes the player's position, if it moved.
  void _read(Emitter<PlayerState> emit) {
    final session = _session;
    if (session == null) {
      return;
    }
    final status = session.status;
    if (status.error != null) {
      emit(
        state.copyWith(
          phase: PlayerPhase.failed,
          message: status.error,
          playing: false,
        ),
      );
      return;
    }
    if (status.positionMs != state.positionMs ||
        status.playing != state.playing ||
        status.lengthMs != state.lengthMs) {
      emit(
        state.copyWith(
          positionMs: status.positionMs,
          lengthMs: status.lengthMs,
          playing: status.playing,
        ),
      );
    }
  }

  Future<void> _onTick(Emitter<PlayerState> emit) async {
    _ticks++;
    if (state.phase == PlayerPhase.notHere) {
      if (_ticks % _recheckEvery == 0 &&
          _repository.partsState(state.fileIds) != PartsState.notHere) {
        await _refresh(emit);
      }
      return;
    }
    _read(emit);
    if (!state.playing && state.phase != PlayerPhase.opening) {
      // Paused or finished: quiet until the next action.
      _stopTimer();
    }
  }

  void _startTimerIfNeeded() {
    if (_timer != null) {
      return;
    }
    final waiting = state.phase == PlayerPhase.notHere;
    if (!state.playing && !waiting && _session?.status.playing != true) {
      return;
    }
    _timer = Timer.periodic(_tick, (_) {
      if (!isClosed) {
        add(const PlayerEvent.ticked());
      }
    });
  }

  void _stopTimer() {
    _timer?.cancel();
    _timer = null;
  }

  void _close() {
    _stopTimer();
    _session?.dispose();
    _session = null;
  }

  @override
  Future<void> close() {
    _close();
    return super.close();
  }
}

/// The speed the button goes to after [current].
double _nextSpeed(double current) {
  for (final speed in playerSpeeds) {
    if (speed > current + 0.001) {
      return speed;
    }
  }
  return playerSpeeds.first;
}

/// The speeds the player offers.
const List<double> playerSpeeds = <double>[0.75, 1, 1.25, 1.5, 2];
