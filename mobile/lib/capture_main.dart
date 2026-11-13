// Kanso — capture_main.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import 'capture/models/data/capture_launch.dart';
import 'capture/models/repos/abs/capture_window_repository.dart';
import 'capture/models/repos/abs/media_picker_repository.dart';
import 'capture/models/repos/impl/platform_capture_window_repository.dart';
import 'capture/state/capture_over_app/capture_over_app_cubit.dart';
import 'capture/state/capture_task/capture_task_cubit.dart';
import 'capture/views/widgets/capture_over_app.dart';
import 'core/bridge/frb_generated.dart';
import 'core/configs/app_environment.dart';
import 'core/configs/core_module.dart';
import 'core/models/data/appearance_status.dart';
import 'core/models/repos/abs/capture_repository.dart';
import 'core/state/appearance/appearance_bloc.dart';
import 'crash/models/repos/abs/crash_report_repository.dart';
import 'crash/utils/crash_capture.dart';
import 'injection.dart';

/// The entry point of the capture sheet over another app on Android: a
/// Flutter engine of this process that `CaptureActivity` shows in a
/// see-through window (the Add task tile, the Capture and Add task
/// shortcuts), started with [args] saying the environment and library
/// folder (see [CaptureLaunch]).
///
/// It runs the same app services on the same library folder as the app, so
/// both engines open the one library the Rust core keeps open once per
/// process, and a task written here is in the app's lists at once.
@pragma('vm:entry-point')
Future<void> captureMain(List<String> args) =>
    bootstrapCapture(CaptureLaunch.fromArgs(args));

/// Prepares the sheet's services and runs it. [window] stands in for the
/// activity in tests; [beforeRun] is awaited once the services are ready
/// and before the first frame.
Future<void> bootstrapCapture(
  CaptureLaunch launch, {
  CaptureWindowRepository? window,
  Future<void> Function()? beforeRun,
}) async {
  WidgetsFlutterBinding.ensureInitialized();
  await KansoCore.init();
  final environment = AppEnvironment.values.firstWhere(
    (value) => value.name == launch.environment,
    orElse: () => AppEnvironment.production,
  );
  libraryFolderOverride = launch.libraryFolder.isEmpty
      ? null
      : launch.libraryFolder;
  await configureDependencies(environment.injectableName);
  if (environment != AppEnvironment.integration) {
    installCrashCapture(getIt<CrashReportRepository>());
  }
  final appearance = getIt<AppearanceBloc>()
    ..add(const AppearanceEvent.started());
  await appearance.stream.firstWhere(
    (state) => state.status != AppearanceStatus.loading,
  );
  await beforeRun?.call();
  runApp(
    CaptureOverApp(
      window: window ?? PlatformCaptureWindowRepository(),
      appearance: appearance,
      tasks: getIt.call<CaptureTaskCubit>,
      media: CaptureOverAppCubit(
        getIt<CaptureRepository>(),
        getIt<MediaPickerRepository>(),
      ),
    ),
  );
}
