// Kanso — graph_bloc.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../discovery/models/repos/abs/discovery_repository.dart';

part 'graph_bloc.freezed.dart';
part 'graph_event.dart';
part 'graph_state.dart';

/// Reads the selected library or local graph through the discovery bridge.
@injectable
class GraphBloc extends Bloc<GraphEvent, GraphState> {
  GraphBloc(this._discovery) : super(const GraphState()) {
    on<GraphStartedEvent>(_onStarted);
    on<GraphFilterSelectedEvent>(_onFilterSelected);
  }

  final DiscoveryRepository _discovery;

  Future<void> _onStarted(
    GraphStartedEvent event,
    Emitter<GraphState> emit,
  ) async {
    emit(state.copyWith(request: event.request, loading: true));
    final result = await _discovery.graph(event.request);
    emit(
      state.copyWith(
        loading: false,
        layout: result.fold((_) => null, (layout) => layout),
        failed: result.isLeft(),
      ),
    );
  }

  Future<void> _onFilterSelected(
    GraphFilterSelectedEvent event,
    Emitter<GraphState> emit,
  ) async {
    final request = switch (state.request) {
      GraphRequest_Local(:final center, :final hops) => GraphRequest.local(
        center: center,
        hops: hops,
      ),
      GraphRequest_Library() => GraphRequest.library_(filter: event.filter),
    };
    await _onStarted(GraphStartedEvent(request), emit);
  }
}
