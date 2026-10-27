// Kanso — editor_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'editor_bloc.dart';

// Events are never copied; the copy would have to reach into the signal.
@Freezed(copyWith: false)
sealed class EditorEvent with _$EditorEvent {
  /// The note [noteId] was opened for editing: open the session on it.
  const factory EditorEvent.started(String noteId) = EditorStartedEvent;

  /// The session said something it was not asked: a save, a change made
  /// elsewhere, a reload, a failure, its end.
  const factory EditorEvent.signalReceived(EditorSignal signal) =
      EditorSignalReceivedEvent;

  /// The text, title or properties were changed, so there is something
  /// unsaved.
  const factory EditorEvent.edited() = EditorEditedEvent;

  /// The caret moved, or the text under it changed, and this is what is
  /// under it. Asks for completions when there are some to offer.
  const factory EditorEvent.caretMoved(EditorContext context) =
      EditorCaretMovedEvent;

  /// The person dismissed the completions.
  const factory EditorEvent.completionDismissed() =
      EditorCompletionDismissedEvent;

  /// The properties were opened or closed.
  const factory EditorEvent.propertiesToggled() = EditorPropertiesToggledEvent;

  /// The value of the property in row [index] was set to [value]; or, with
  /// no [index], the property called [key] was (added when there is none).
  const factory EditorEvent.propertySet({
    required String key,
    required String value,
    int? index,
  }) = EditorPropertySetEvent;

  /// The property in row [index] was removed.
  const factory EditorEvent.propertyRemoved(int index) =
      EditorPropertyRemovedEvent;

  /// The person chose to keep what they typed over a change made elsewhere.
  const factory EditorEvent.keepMineRequested() = EditorKeepMineRequestedEvent;

  /// The person chose the change made elsewhere over what they typed.
  const factory EditorEvent.useTheirsRequested() =
      EditorUseTheirsRequestedEvent;

  /// Save now, for [reason].
  const factory EditorEvent.flushRequested(FlushReason reason) =
      EditorFlushRequestedEvent;

  /// The app moved to [lifecycle]; going to the background saves.
  const factory EditorEvent.lifecycleChanged(AppLifecycleState lifecycle) =
      EditorLifecycleChangedEvent;

  /// The person asked to open where the link [target] leads.
  const factory EditorEvent.linkOpenRequested(LinkTarget target) =
      EditorLinkOpenRequestedEvent;

  /// The screen has gone to the destination the state named.
  const factory EditorEvent.destinationHandled() =
      EditorDestinationHandledEvent;

  /// The editor is left: save, keeping what was typed, and let go.
  const factory EditorEvent.closed() = EditorClosedEvent;
}
