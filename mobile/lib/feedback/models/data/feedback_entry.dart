// Kanso — feedback_entry.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import 'feedback_kind.dart';

part 'feedback_entry.freezed.dart';

/// What a report's chip says; the screen words it from the ARB strings.
enum FeedbackChip {
  waitingToSend,
  received,
  filedAs,
  building,
  fixProposed,
  merged,
  releasedIn,
  notFiled,
  sameAs,
  closed,
  deleting,
}

/// One report in My feedback.
@freezed
abstract class FeedbackEntry with _$FeedbackEntry {
  const factory FeedbackEntry({
    required String id,
    required int createdMs,
    required FeedbackKind kind,

    /// The first line of the text.
    required String preview,
    required FeedbackChip chip,

    /// The issue number for [FeedbackChip.filedAs] and [FeedbackChip.sameAs].
    int? number,

    /// `0.9.2` for [FeedbackChip.releasedIn].
    String? release,

    /// The pull request that fixes it, once there is one.
    int? pr,

    /// Where the issue is on GitHub, once it is filed.
    String? issueUrl,

    /// A delete is waiting for the server.
    @Default(false) bool deleting,

    /// Released, and not yet looked at.
    @Default(false) bool fresh,

    /// Whether the server has it (not still waiting on this phone).
    @Default(true) bool onServer,
  }) = _FeedbackEntry;
}
