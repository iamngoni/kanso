// Kanso — book_prompt.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../../core/bridge/api/types.dart';

part 'book_prompt.freezed.dart';

/// A question the Share notebook sheet asks before it invites someone.
@freezed
sealed class BookPrompt with _$BookPrompt {
  /// Someone new: compare [person]'s fingerprint before inviting them with
  /// [role].
  const factory BookPrompt.verify({
    required PersonRow person,
    required BookRole role,
  }) = BookPromptVerify;

  /// The keys the server gives for someone are not the ones pinned for them.
  const factory BookPrompt.keyChanged({
    required PersonRow offered,
    required BookRole role,
    PersonRow? pinned,
  }) = BookPromptKeyChanged;
}

/// What the Share notebook sheet just did, to say so.
@freezed
sealed class BookDone with _$BookDone {
  /// The notebook was shared with [username]: "Shared "Studio" with dan.
  /// It shows for them once they accept."
  const factory BookDone.shared(String username) = BookDoneShared;

  /// [username] was invited.
  const factory BookDone.invited(String username) = BookDoneInvited;

  /// [username] was removed: the notebook has a new key.
  const factory BookDone.removed(String username) = BookDoneRemoved;

  /// The notebook stopped being shared here, left or stopped.
  const factory BookDone.ended(BookEndedRow ended) = BookDoneEnded;
}
