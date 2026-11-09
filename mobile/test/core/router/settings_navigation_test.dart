// Kanso — settings_navigation_test.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:kanso/core/router/kanso_routes.dart';
import 'package:kanso/core/router/kanso_sheet_page.dart';
import 'package:kanso/core/router/settings_navigation.dart';
import 'package:kanso/core/views/widgets/kanso_page_route.dart';

/// Settings as the router has it (a shell: a sheet on a tablet, a page on a
/// phone) with the account's sign-in page on top of it. Signing in is a
/// button that changes what Settings says and then does what the sign-in
/// page does, `popToSettings`.
GoRouter _router({
  required bool tablet,
  required ValueNotifier<bool> signedIn,
}) {
  return GoRouter(
    initialLocation: KansoRoutes.notes,
    routes: <RouteBase>[
      GoRoute(
        path: KansoRoutes.notes,
        builder: (context, state) => const Text('notes'),
      ),
      ShellRoute(
        pageBuilder: (context, state, child) => tablet
            ? KansoSheetPage<void>(key: state.pageKey, child: child)
            : kansoPage<void>(key: state.pageKey, child: child),
        routes: <RouteBase>[
          GoRoute(
            path: KansoRoutes.settings,
            pageBuilder: (context, state) => kansoPage<void>(
              key: state.pageKey,
              child: ValueListenableBuilder<bool>(
                valueListenable: signedIn,
                builder: (context, on, _) =>
                    Text(on ? 'Settings: signed in' : 'Settings: signed out'),
              ),
            ),
            routes: <RouteBase>[
              GoRoute(
                path: 'account/sign-in',
                pageBuilder: (context, state) => kansoPage<void>(
                  key: state.pageKey,
                  child: Builder(
                    builder: (context) => TextButton(
                      onPressed: () {
                        signedIn.value = true;
                        GoRouter.of(context).popToSettings();
                      },
                      child: const Text('Sign in'),
                    ),
                  ),
                ),
              ),
            ],
          ),
        ],
      ),
    ],
  );
}

void main() {
  for (final tablet in <bool>[true, false]) {
    for (final pushed in <bool>[true, false]) {
      testWidgets(
        'popToSettings leaves the sign-in page '
        '(${tablet ? 'tablet sheet' : 'phone'}, '
        '${pushed ? 'pushed' : 'gone to'})',
        (tester) async {
          tester.view
            ..physicalSize = tablet
                ? const Size(1024, 1366)
                : const Size(393, 852)
            ..devicePixelRatio = 1;
          addTearDown(tester.view.reset);
          final signedIn = ValueNotifier<bool>(false);
          addTearDown(signedIn.dispose);
          final router = _router(tablet: tablet, signedIn: signedIn);
          addTearDown(router.dispose);
          await tester.pumpWidget(MaterialApp.router(routerConfig: router));
          router.go(KansoRoutes.settings);
          await tester.pumpAndSettle();
          expect(find.text('Settings: signed out'), findsOneWidget);

          if (pushed) {
            unawaited(router.push<void>(KansoRoutes.settingsSignIn));
          } else {
            router.go(KansoRoutes.settingsSignIn);
          }
          await tester.pumpAndSettle();
          expect(router.topLocation, KansoRoutes.settingsSignIn);

          await tester.tap(find.text('Sign in'));
          await tester.pumpAndSettle();

          // The page that is seen is Settings, and says it is signed in.
          expect(find.text('Sign in'), findsNothing);
          expect(find.text('Settings: signed in'), findsOneWidget);
          expect(router.topLocation, KansoRoutes.settings);
        },
      );
    }
  }
}
