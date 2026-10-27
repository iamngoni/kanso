// Kanso — bootstrap.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import 'app.dart';
import 'core/bridge/frb_generated.dart';
import 'core/configs/app_environment.dart';
import 'core/models/data/appearance_status.dart';
import 'core/router/kanso_router.dart';
import 'core/state/appearance/appearance_bloc.dart';
import 'core/utils/license_registration.dart';
import 'crash/models/repos/abs/crash_report_repository.dart';
import 'crash/utils/crash_capture.dart';
import 'injection.dart';
import 'providers.dart';

/// Prepares the app and runs it for [environment].
///
/// [beforeRun], for the performance bench, is awaited once the app's parts
/// are ready and before the first frame (to set up a library, say).
Future<void> bootstrap(
  AppEnvironment environment, {
  Future<void> Function()? beforeRun,
}) async {
  WidgetsFlutterBinding.ensureInitialized();
  registerBundledLicenses();
  // The Rust core: its library is loaded (the build puts it in the app)
  // before anything opens a note library.
  await KansoCore.init();
  await configureDependencies(environment.injectableName);
  // Dart errors are kept as scrubbed crash reports; integration tests keep
  // the framework's own handlers so a failure still fails the test.
  if (environment != AppEnvironment.integration) {
    installCrashCapture(getIt<CrashReportRepository>());
  }
  // The saved themes and fonts are read before the first frame, so the app
  // never flashes the default look.
  final appearance = getIt<AppearanceBloc>()
    ..add(const AppearanceEvent.started());
  await appearance.stream.firstWhere(
    (state) => state.status != AppearanceStatus.loading,
  );
  await beforeRun?.call();
  runApp(
    AppProviders(
      appearance: appearance,
      child: App(router: getIt<KansoRouter>()),
    ),
  );
}
