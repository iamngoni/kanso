// Kanso — player_state.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'player_bloc.dart';

/// Where the card stands.
enum PlayerPhase {
  /// The note holds no recording: no card.
  none,

  /// A recording whose bytes are here; the speaker opens on Play.
  idle,

  /// Reading the parts and opening the speaker.
  opening,

  /// Open: it plays.
  ready,

  /// A part's bytes haven't arrived by sync.
  notHere,

  /// A part is gone, or sealed under a lock.
  missing,

  /// The file or the speaker failed; [PlayerState.message] says how.
  failed,
}

@freezed
abstract class PlayerState with _$PlayerState {
  const factory PlayerState({
    @Default(PlayerPhase.none) PlayerPhase phase,
    @Default('') String noteId,

    /// The parts of the recording, in playing order. The first is the file
    /// every timestamp in the note names.
    @Default(<String>[]) List<String> fileIds,
    @Default(0) int positionMs,
    @Default(0) int lengthMs,
    @Default(false) bool playing,
    @Default(1.0) double speed,
    @Default(1.0) double volume,

    /// Counts the timestamp taps: each asks the card to scroll into view.
    @Default(0) int reveals,
    String? message,
  }) = _PlayerState;

  const PlayerState._();

  /// Whether the note has a card to draw.
  bool get shown => phase != PlayerPhase.none;
}
