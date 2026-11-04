// Kanso — player_event.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'player_bloc.dart';

@freezed
sealed class PlayerEvent with _$PlayerEvent {
  /// The note opened: look for the recording it holds.
  const factory PlayerEvent.started(String noteId) = PlayerStartedEvent;

  /// The note's text changed (or sync brought it): look again.
  const factory PlayerEvent.refreshed() = PlayerRefreshedEvent;

  /// Play or pause; the first Play opens the speaker.
  const factory PlayerEvent.toggled() = PlayerToggledEvent;

  /// Move to [ms] into the recording (the scrubber was released).
  const factory PlayerEvent.seeked(int ms) = PlayerSeekedEvent;

  /// Move by [deltaMs] (negative goes back).
  const factory PlayerEvent.skipped(int deltaMs) = PlayerSkippedEvent;

  /// The speed button: the next of 0.75×, 1×, 1.25×, 1.5×, 2×.
  const factory PlayerEvent.speedCycled() = PlayerSpeedCycledEvent;

  const factory PlayerEvent.volumeChanged(double volume) =
      PlayerVolumeChangedEvent;

  const factory PlayerEvent.muteToggled() = PlayerMuteToggledEvent;

  /// A timestamp in the note was tapped: seek to [seconds] of the recording
  /// [fileId] belongs to, play, and bring the card into view.
  const factory PlayerEvent.momentTapped(String fileId, int seconds) =
      PlayerMomentTappedEvent;

  /// Try Again after a failure.
  const factory PlayerEvent.retried() = PlayerRetriedEvent;

  /// The clock: read the player.
  const factory PlayerEvent.ticked() = PlayerTickedEvent;
}
