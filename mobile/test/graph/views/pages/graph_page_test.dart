// Kanso — graph_page_test.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/router/kanso_routes.dart';
import 'package:kanso/graph/state/graph/graph_bloc.dart';
import 'package:kanso/graph/views/pages/graph_page.dart';
import 'package:kanso/graph/views/widgets/graph_canvas.dart';
import 'package:kanso/injection.dart';

import '../../../helpers/fixture_discovery_repository.dart';
import '../../../helpers/golden.dart';
import '../../../helpers/pump_routed.dart';

const GraphLayout _layout = GraphLayout(
  nodes: <GraphNodeRow>[
    GraphNodeRow(
      id: 'note-a',
      title: 'Seed note',
      notebookId: 'book-a',
      degree: 1,
      x: -24,
      y: -10,
      radius: 10,
      colorLight: 0xff7389a6,
      colorDark: 0xff607b9e,
      isCenter: false,
    ),
    GraphNodeRow(
      id: 'note-b',
      title: 'Open note',
      notebookId: 'book-a',
      degree: 2,
      x: 0,
      y: 0,
      radius: 14,
      colorLight: 0xff647d9c,
      colorDark: 0xff4e6d94,
      isCenter: true,
    ),
    GraphNodeRow(
      id: 'note-c',
      title: 'Related note',
      notebookId: 'book-a',
      degree: 1,
      x: 24,
      y: 10,
      radius: 10,
      colorLight: 0xff967d61,
      colorDark: 0xffa68c6d,
      isCenter: false,
    ),
  ],
  edges: <GraphEdgeRow>[
    GraphEdgeRow(from: 0, to: 1),
    GraphEdgeRow(from: 1, to: 2),
  ],
  linkCount: 2,
);

class _GraphRepository extends FixtureDiscoveryRepository {
  @override
  Future<Either<ApplicationError, GraphLayout>> graph(
    GraphRequest request,
  ) async => const Right<ApplicationError, GraphLayout>(_layout);
}

void _registerGraphBloc() {
  getIt.registerFactory<GraphBloc>(
    () => GraphBloc(_GraphRepository()),
  );
  addTearDown(() async => getIt.reset());
}

void main() {
  for (final brightness in Brightness.values) {
    testWidgets('local graph page, ${brightness.name}', (tester) async {
      _registerGraphBloc();
      await expectGoldenOnBothPlatforms(
        tester,
        const GraphPage(centerId: 'note-b'),
        name: 'phone_graph_page_${brightness.name}',
        size: phoneSize,
        brightness: brightness,
      );
      expect(find.text('Graph'), findsOneWidget);
      expect(find.byType(GraphCanvas), findsOneWidget);
    });
  }

  testWidgets('tapping a local graph node opens that note', (tester) async {
    _registerGraphBloc();
    final router = await pumpRouted(
      tester,
      const GraphPage(centerId: 'note-b'),
    );
    final canvas = tester.getRect(
      find.descendant(
        of: find.byType(GraphCanvas),
        matching: find.byType(CustomPaint),
      ),
    );
    await tester.tapAt(canvas.center);
    await tester.pumpAndSettle();
    expect(topOf(router), '${KansoRoutes.noteById}/note-b');
  });
}
