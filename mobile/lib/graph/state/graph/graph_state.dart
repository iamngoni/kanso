// Kanso — graph_state.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'graph_bloc.dart';

@freezed
abstract class GraphState with _$GraphState {
  const factory GraphState({
    @Default(GraphRequest.library_(filter: GraphFilter.all()))
    GraphRequest request,
    GraphLayout? layout,
    @Default(false) bool loading,
    @Default(false) bool failed,
  }) = _GraphState;
}
