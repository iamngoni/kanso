// Kanso — pump_routed.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:kanso/core/configs/kanso_theme.dart';
import 'package:kanso/core/utils/kanso_scroll_behavior.dart';
import 'package:kanso/gen/l10n/app_localizations.dart';

/// Pumps [child] as the page at `/start` of a router that has the app's
/// addresses, each an empty page, and returns the router so a test can see
/// where a tap went.
Future<GoRouter> pumpRouted(
  WidgetTester tester,
  Widget child, {
  Size size = const Size(393, 852),
  Brightness brightness = Brightness.light,
  Widget Function(Widget page)? wrap,
}) async {
  tester.view
    ..physicalSize = size
    ..devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  Widget empty(BuildContext context, GoRouterState state) =>
      const SizedBox.shrink();
  final router = GoRouter(
    initialLocation: '/start',
    routes: <RouteBase>[
      GoRoute(
        path: '/start',
        builder: (context, state) => Scaffold(
          resizeToAvoidBottomInset: false,
          body: wrap == null ? child : wrap(child),
        ),
      ),
      GoRoute(path: '/notes', builder: empty),
      GoRoute(path: '/notes/note/:id', builder: empty),
      GoRoute(path: '/pinned/note/:id', builder: empty),
      GoRoute(path: '/notebooks/:id/note/:noteId', builder: empty),
      GoRoute(path: '/tags/:id/note/:noteId', builder: empty),
      GoRoute(path: '/search/note/:id', builder: empty),
      GoRoute(path: '/tasks', builder: empty),
      GoRoute(path: '/tasks/note/:id', builder: empty),
      GoRoute(path: '/pinned', builder: empty),
      GoRoute(path: '/recent', builder: empty),
      GoRoute(path: '/trash', builder: empty),
      GoRoute(path: '/notebooks/:id', builder: empty),
      GoRoute(path: '/tags/:id', builder: empty),
      GoRoute(path: '/search', builder: empty),
      GoRoute(
        path: '/settings',
        builder: empty,
        routes: <RouteBase>[
          GoRoute(path: 'notebook', builder: empty),
          GoRoute(path: 'theme', builder: empty),
          GoRoute(path: 'icons', builder: empty),
          GoRoute(path: 'fonts', builder: empty),
          GoRoute(path: 'licences', builder: empty),
          GoRoute(path: 'account', builder: empty),
          GoRoute(path: 'account/sign-in', builder: empty),
          GoRoute(path: 'account/unlock', builder: empty),
          GoRoute(path: 'account/recover', builder: empty),
          GoRoute(path: 'account/recovery-key', builder: empty),
          GoRoute(path: 'crash-reports', builder: empty),
        ],
      ),
      GoRoute(
        path: '/shared',
        builder: empty,
        routes: <RouteBase>[GoRoute(path: 'join', builder: empty)],
      ),
      GoRoute(path: '/design', builder: empty),
    ],
  );
  addTearDown(router.dispose);
  await tester.pumpWidget(
    MaterialApp.router(
      theme: KansoTheme.light,
      darkTheme: KansoTheme.dark,
      themeMode: brightness == Brightness.dark
          ? ThemeMode.dark
          : ThemeMode.light,
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      scrollBehavior: const KansoScrollBehavior(),
      routerConfig: router,
    ),
  );
  await tester.pumpAndSettle();
  return router;
}

/// Where [router] is, as an address.
String whereIs(GoRouter router) =>
    router.routerDelegate.currentConfiguration.uri.toString();

/// The address of the page on top of [router]'s stack, which is not
/// [whereIs] after a `push`.
String topOf(GoRouter router) =>
    router.routerDelegate.currentConfiguration.matches.last.matchedLocation;
