// Kanso — graph_bloc_test.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/discovery/models/repos/abs/discovery_repository.dart';
import 'package:kanso/graph/state/graph/graph_bloc.dart';
import 'package:mocktail/mocktail.dart';

class _MockDiscoveryRepository extends Mock implements DiscoveryRepository {}

void main() {
  late _MockDiscoveryRepository discovery;
  const empty = GraphLayout(
    nodes: <GraphNodeRow>[],
    edges: <GraphEdgeRow>[],
    linkCount: 0,
  );

  setUpAll(() {
    registerFallbackValue(
      const GraphRequest.library_(filter: GraphFilter.all()),
    );
  });

  setUp(() {
    discovery = _MockDiscoveryRepository();
    when(() => discovery.graph(any())).thenAnswer(
      (_) async => const Right<ApplicationError, GraphLayout>(empty),
    );
  });

  blocTest<GraphBloc, GraphState>(
    'loads the requested local graph around the open note',
    build: () => GraphBloc(discovery),
    act: (bloc) => bloc.add(
      const GraphEvent.started(GraphRequest.local(center: 'note-a', hops: 2)),
    ),
    expect: () => <Matcher>[
      isA<GraphState>().having((state) => state.loading, 'loading', true),
      isA<GraphState>()
          .having((state) => state.loading, 'loading', false)
          .having((state) => state.layout, 'layout', empty),
    ],
    verify: (_) => verify(
      () => discovery.graph(
        const GraphRequest.local(center: 'note-a', hops: 2),
      ),
    ).called(1),
  );

  blocTest<GraphBloc, GraphState>(
    'applies a library notebook filter',
    build: () => GraphBloc(discovery),
    act: (bloc) async {
      bloc.add(
        const GraphEvent.started(
          GraphRequest.library_(filter: GraphFilter.all()),
        ),
      );
      await Future<void>.delayed(Duration.zero);
      bloc.add(const GraphEvent.filterSelected(GraphFilter.book(id: 'book-a')));
    },
    wait: const Duration(milliseconds: 20),
    verify: (_) => verify(
      () => discovery.graph(
        const GraphRequest.library_(filter: GraphFilter.book(id: 'book-a')),
      ),
    ).called(1),
  );
}
