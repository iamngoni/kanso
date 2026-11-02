// Kanso — triage_event.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'triage_bloc.dart';

@freezed
sealed class TriageEvent with _$TriageEvent {
  /// The pager opened: read the cards and keep them up to date.
  const factory TriageEvent.started() = TriageStartedEvent;

  /// The library changed these notes. A card among them says the note
  /// changed on another device and File waits.
  const factory TriageEvent.changesReceived(List<String> ids) =
      TriageChangesReceivedEvent;

  /// Read the cards again, keeping what the person did to those still there.
  const factory TriageEvent.reloaded() = TriageReloadedEvent;

  /// Accept chip [index] of a card, or lift its acceptance.
  const factory TriageEvent.chipAccepted(String noteId, int index) =
      TriageChipAcceptedEvent;

  /// Reject chip [index] of a card, or lift its rejection.
  const factory TriageEvent.chipRejected(String noteId, int index) =
      TriageChipRejectedEvent;

  /// Change chip [index] to what [text] makes for its field.
  const factory TriageEvent.chipEdited(
    String noteId,
    int index,
    String text,
  ) = TriageChipEditedEvent;

  /// Accept every chip on a card ("Accept all").
  const factory TriageEvent.cardAcceptedAll(String noteId) =
      TriageCardAcceptedAllEvent;

  /// Accept every suggested notebook (or tag) on every card.
  const factory TriageEvent.fieldAcceptedEverywhere(TriageField field) =
      TriageFieldAcceptedEverywhereEvent;

  /// Hide a card for this visit.
  const factory TriageEvent.cardSkipped(String noteId) = TriageCardSkippedEvent;

  /// File a card with what is accepted on it.
  const factory TriageEvent.cardFiled(String noteId) = TriageCardFiledEvent;

  /// "File all accepted": plan it, so the question can say what it does.
  const factory TriageEvent.fileAllRequested() = TriageFileAllRequestedEvent;

  /// The question was answered yes.
  const factory TriageEvent.fileAllConfirmed() = TriageFileAllConfirmedEvent;

  /// The question was answered no.
  const factory TriageEvent.planDismissed() = TriagePlanDismissedEvent;

  /// Ask the person's AI to rank a card's candidates.
  const factory TriageEvent.refineRequested(String noteId) =
      TriageRefineRequestedEvent;

  /// Turn Related notes on, for notebook and tag suggestions.
  const factory TriageEvent.relatedTurnedOn() = TriageRelatedTurnedOnEvent;

  /// The person has read the notice.
  const factory TriageEvent.noticeDismissed() = TriageNoticeDismissedEvent;
}
