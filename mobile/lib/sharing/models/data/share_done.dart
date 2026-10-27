// Kanso — share_done.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'share_done.freezed.dart';

/// What the last step of the share sheet did, when it is worth saying.
@freezed
sealed class ShareDone with _$ShareDone {
  /// [username] was invited.
  const factory ShareDone.invited(String username) = ShareDoneInvited;

  /// The owner stopped sharing.
  const factory ShareDone.stopped() = ShareDoneStopped;

  /// The person left the note.
  const factory ShareDone.left() = ShareDoneLeft;
}
