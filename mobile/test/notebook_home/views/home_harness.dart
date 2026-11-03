// Kanso — home_harness.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/notebook_home/models/repos/abs/notebook_home_repository.dart';
import 'package:kanso/notebook_home/state/notebook_home/notebook_home_bloc.dart';

import '../../helpers/browse_blocs.dart';
import '../../helpers/fake_notebook_home_repository.dart';
import '../../helpers/kanso_test_app.dart';

/// Makes [fake] the app's notebook homepage repository for a test, with a
/// bloc over it.
void useHomeRepository(FakeNotebookHomeRepository fake) {
  void swap<T extends Object>(T Function() make) {
    if (getIt.isRegistered<T>()) {
      getIt.unregister<T>();
    }
    getIt.registerFactory<T>(make);
  }

  swap<NotebookHomeRepository>(() => fake);
  swap<NotebookHomeBloc>(() => NotebookHomeBloc(fake));
  addTearDown(() {
    if (getIt.isRegistered<NotebookHomeBloc>()) {
      getIt.unregister<NotebookHomeBloc>();
    }
    if (getIt.isRegistered<NotebookHomeRepository>()) {
      getIt.unregister<NotebookHomeRepository>();
    }
  });
}

/// Pumps [build]'s page inside the app's BLoCs and a router that has the
/// addresses the homepage goes to; each of those is an empty page. Returns
/// the router so the test can see where a tap went.
Future<GoRouter> pumpHome(
  WidgetTester tester,
  BrowseBlocs blocs,
  Widget Function() build, {
  Size size = const Size(393, 852),
  List<Uri>? visits,
}) async {
  tester.view
    ..physicalSize = size
    ..devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  Widget empty(BuildContext context, GoRouterState state) {
    visits?.add(state.uri);
    return const SizedBox.shrink();
  }

  final router = GoRouter(
    initialLocation: '/start',
    routes: <RouteBase>[
      GoRoute(
        path: '/start',
        builder: (context, state) => Scaffold(
          resizeToAvoidBottomInset: false,
          body: blocs.provide(build()),
        ),
      ),
      GoRoute(path: '/note/:id', builder: empty),
      GoRoute(path: '/graph', builder: empty),
      GoRoute(path: '/ask', builder: empty),
      GoRoute(path: '/notebooks/:id', builder: empty),
      GoRoute(path: '/notebooks/:id/home', builder: empty),
      GoRoute(path: '/settings/ai', builder: empty),
    ],
  );
  addTearDown(router.dispose);
  await tester.pumpWidget(KansoTestRouterApp(routerConfig: router));
  await tester.pumpAndSettle();
  return router;
}
