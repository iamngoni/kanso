// Kanso — graph_event.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'graph_bloc.dart';

@freezed
sealed class GraphEvent with _$GraphEvent {
  const factory GraphEvent.started(GraphRequest request) = GraphStartedEvent;
  const factory GraphEvent.filterSelected(GraphFilter filter) =
      GraphFilterSelectedEvent;
}
