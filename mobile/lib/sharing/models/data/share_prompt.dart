// Kanso — share_prompt.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../../core/bridge/api/types.dart';

part 'share_prompt.freezed.dart';

/// A question the share sheet asks before it trusts someone's keys: the
/// person compares a fingerprint out loud, then answers.
@freezed
sealed class SharePrompt with _$SharePrompt {
  /// The first share with [person]: compare their fingerprint, then invite
  /// them as [role].
  const factory SharePrompt.verify({
    required PersonRow person,
    required ShareRole role,
  }) = SharePromptVerify;

  /// The server gives other keys for [offered] than the ones pinned
  /// ([pinned], when there are any): compare the new fingerprint, then trust
  /// them. [role] is the role to invite them as afterwards; null when the
  /// prompt came from a member of the note, who is already in it.
  const factory SharePrompt.keyChanged({
    required PersonRow offered,
    PersonRow? pinned,
    ShareRole? role,
  }) = SharePromptKeyChanged;
}
