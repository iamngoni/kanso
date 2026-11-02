// Kanso — review_prompt_record.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'review_prompt_record.freezed.dart';
part 'review_prompt_record.g.dart';

/// What the phone counts to decide whether to ask for a review of Kanso
/// Pocket: kept in the preferences, on this phone only, and never sent.
@freezed
abstract class ReviewPromptRecord with _$ReviewPromptRecord {
  const factory ReviewPromptRecord({
    /// When the app was first opened with this record (the install, or the
    /// update that brought it). Null until then.
    DateTime? firstOpenedAt,

    /// On how many different days the app was opened.
    @Default(0) int activeDays,

    /// The last day the app was opened, `YYYY-MM-DD` in local time.
    String? lastActiveDay,

    /// How many times a note was let go after it was written in.
    @Default(0) int notesEdited,

    /// When the app last crashed, or sync or signing in last failed.
    DateTime? lastTroubleAt,

    /// When the app last asked the system for a review.
    DateTime? lastAskedAt,

    /// The versions of the app that have asked, oldest first; one is never
    /// asked from twice.
    @Default(<String>[]) List<String> askedVersions,

    /// Whether a sync has completed on this phone.
    @Default(false) bool firstSyncSeen,
  }) = _ReviewPromptRecord;

  factory ReviewPromptRecord.fromJson(Map<String, dynamic> json) =>
      _$ReviewPromptRecordFromJson(json);
}
