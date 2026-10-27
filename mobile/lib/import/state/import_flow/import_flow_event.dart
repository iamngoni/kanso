// Kanso: import_flow_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'import_flow_bloc.dart';

@freezed
sealed class ImportFlowEvent with _$ImportFlowEvent {
  /// The flow opened. With a [path] (a file or folder another part of the
  /// app already has, such as an export that was shared), the flow reads
  /// that instead of asking for one; [kind] says which source it is, when
  /// that is known.
  const factory ImportFlowEvent.started({String? path, String? kind}) =
      ImportFlowStartedEvent;

  /// A source was chosen from the list. Without a path already given, this
  /// asks the system for a file, or for a folder when [folder] is set.
  const factory ImportFlowEvent.sourceChosen(
    String kind, {
    @Default(false) bool folder,
  }) = ImportFlowSourceChosenEvent;

  /// Import was pressed on the summary.
  const factory ImportFlowEvent.confirmed() = ImportFlowConfirmedEvent;

  /// Reading or importing again after it failed.
  const factory ImportFlowEvent.retried() = ImportFlowRetriedEvent;

  /// Back to the list of sources.
  const factory ImportFlowEvent.restarted() = ImportFlowRestartedEvent;
}
