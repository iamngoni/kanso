// Kanso — feedback_labels.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../gen/l10n/app_localizations.dart';
import '../models/data/feedback_entry.dart';
import '../models/data/feedback_kind.dart';
import '../models/data/feedback_result.dart';

/// The words for feedback: the kinds, the chips, and how a send ended.
extension FeedbackLabels on AppLocalizations {
  /// A kind's name in the segmented control.
  String feedbackKindName(FeedbackKind kind) => switch (kind) {
    FeedbackKind.bug => feedbackKindBug,
    FeedbackKind.idea => feedbackKindIdea,
    FeedbackKind.question => feedbackKindQuestion,
  };

  /// The text box's hint for a kind of report.
  String feedbackHint(FeedbackKind kind) => switch (kind) {
    FeedbackKind.idea => feedbackHintIdea,
    FeedbackKind.bug || FeedbackKind.question => feedbackHintProblem,
  };

  /// A report's chip in words.
  String feedbackChipLabel(FeedbackEntry entry) => switch (entry.chip) {
    FeedbackChip.waitingToSend => feedbackChipWaiting,
    FeedbackChip.received => feedbackChipReceived,
    FeedbackChip.filedAs => feedbackChipFiled(entry.number ?? 0),
    FeedbackChip.building => feedbackChipBuilding,
    FeedbackChip.fixProposed => feedbackChipFixProposed,
    FeedbackChip.merged => feedbackChipMerged,
    FeedbackChip.releasedIn => feedbackChipReleased(entry.release ?? ''),
    FeedbackChip.notFiled => feedbackChipNotFiled,
    FeedbackChip.sameAs => feedbackChipSame(entry.number ?? 0),
    FeedbackChip.closed => feedbackChipClosed,
    FeedbackChip.deleting => feedbackChipDeleting,
  };

  /// A line under the chip, when there is more to say.
  String? feedbackChipDetail(FeedbackEntry entry) => switch (entry.chip) {
    FeedbackChip.notFiled => feedbackDetailNotFiled,
    FeedbackChip.sameAs => feedbackDetailSame(entry.number ?? 0),
    _ => null,
  };

  /// Why a report can't be sent as written.
  String feedbackProblemMessage(FeedbackProblem? problem) => switch (problem) {
    FeedbackProblem.tooShort => feedbackErrorTooShort,
    FeedbackProblem.tooLong => feedbackErrorTooLong,
    FeedbackProblem.badCharacter => feedbackErrorCharacter,
    FeedbackProblem.badCredit => feedbackErrorCredit,
    FeedbackProblem.badScreenshot => feedbackErrorScreenshot,
    FeedbackProblem.other || null => feedbackErrorOther,
  };

  /// What to tell the person once a send has ended: the toast of a report
  /// that went or was saved, or the line that keeps the page open.
  String feedbackOutcomeMessage(FeedbackSendResult result) =>
      switch (result.outcome) {
        FeedbackOutcome.sent => feedbackSentToast,
        FeedbackOutcome.queued => feedbackQueuedToast,
        FeedbackOutcome.paused => feedbackPausedToast,
        FeedbackOutcome.rateLimited => feedbackRateLimitedToast,
        FeedbackOutcome.rejected => feedbackRejectedMessage(result.field),
        FeedbackOutcome.invalid => feedbackProblemMessage(result.problem),
      };

  /// "Kanso couldn't send this report (field)…". The field is a JSON path
  /// from the server; anything else in it is dropped, and the value that
  /// failed is never shown.
  String feedbackRejectedMessage(String? field) {
    final clean = (field ?? '').replaceAll(RegExp(r'[^A-Za-z0-9._\[\]]'), '');
    final safe = clean.length > 40 ? clean.substring(0, 40) : clean;
    return safe.isEmpty
        ? feedbackErrorRejectedPlain
        : feedbackErrorRejected(safe);
  }
}

/// Whether a send ended with the report kept (sent or saved), so the page
/// closes and a toast says so.
bool feedbackClosesPage(FeedbackOutcome outcome) => switch (outcome) {
  FeedbackOutcome.sent ||
  FeedbackOutcome.queued ||
  FeedbackOutcome.paused ||
  FeedbackOutcome.rateLimited => true,
  FeedbackOutcome.rejected || FeedbackOutcome.invalid => false,
};
