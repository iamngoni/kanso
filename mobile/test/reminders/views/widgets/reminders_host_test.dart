// Kanso — reminders_host_test.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/reminders/utils/reminders_coordinator.dart';
import 'package:kanso/reminders/views/widgets/reminders_host.dart';
import 'package:mocktail/mocktail.dart';

class _MockRemindersCoordinator extends Mock implements RemindersCoordinator {}

void main() {
  testWidgets('Open routes to the note and its task line', (tester) async {
    final opened = StreamController<TaskPlace>();
    final coordinator = _MockRemindersCoordinator();
    when(() => coordinator.opened).thenAnswer((_) => opened.stream);
    when(coordinator.start).thenAnswer((_) async {});

    final router = GoRouter(
      initialLocation: '/notes',
      routes: <RouteBase>[
        GoRoute(
          path: '/notes',
          builder: (context, state) => const Text('All Notes'),
          routes: <RouteBase>[
            GoRoute(
              path: 'note/:noteId',
              builder: (context, state) => Text(
                '${state.pathParameters['noteId']}: '
                '${state.uri.queryParameters['line']}',
              ),
            ),
          ],
        ),
      ],
    );
    addTearDown(router.dispose);
    addTearDown(opened.close);

    await tester.pumpWidget(
      MaterialApp.router(
        routerConfig: router,
        builder: (context, child) => RemindersHost(
          coordinator: coordinator,
          router: router,
          child: child!,
        ),
      ),
    );
    await tester.pumpAndSettle();

    opened.add(const TaskPlace(noteId: 'note:plan', line: 7));
    await tester.pumpAndSettle();

    expect(find.text('note:plan: 7'), findsOneWidget);
  });
}
