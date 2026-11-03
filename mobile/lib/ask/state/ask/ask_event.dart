// Kanso — ask_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'ask_bloc.dart';

@freezed
sealed class AskEvent with _$AskEvent {
  /// The screen opened: see whether AI is on and where questions go. [scope]
  /// is what the first question searches, when it was opened from a
  /// notebook; all notes when null.
  const factory AskEvent.started({AskScope? scope}) = AskStartedEvent;

  /// The settings changed while this conversation was open.
  const factory AskEvent.settingsChanged(AiSettings settings) =
      AskSettingsChangedEvent;

  /// The person asked [question] of their notes.
  const factory AskEvent.questionAsked(String question) = AskQuestionAskedEvent;

  /// The core told how the question is going.
  const factory AskEvent.updateReceived(AskUpdate update) =
      AskUpdateReceivedEvent;

  /// The core ended its updates.
  const factory AskEvent.streamEnded() = AskStreamEndedEvent;

  /// The person pressed Stop.
  const factory AskEvent.stopped() = AskStoppedEvent;

  /// The person cleared the conversation.
  const factory AskEvent.cleared() = AskClearedEvent;

  /// The person asked to try the failed question again.
  const factory AskEvent.retried() = AskRetriedEvent;

  /// The person chose what the next question searches.
  const factory AskEvent.scopeChosen(AskScope scope) = AskScopeChosenEvent;

  /// The scope sheet opened: list what a question can be limited to.
  const factory AskEvent.scopesRequested() = AskScopesRequestedEvent;

  /// The person dropped the time range the question was understood to have.
  const factory AskEvent.timeDropped() = AskTimeDroppedEvent;

  /// Nothing matched in a narrow scope; the person asked all notes.
  const factory AskEvent.searchedAll() = AskSearchedAllEvent;

  /// The person pressed Save as note on the newest answer.
  const factory AskEvent.answerSaved() = AskAnswerSavedEvent;
}
