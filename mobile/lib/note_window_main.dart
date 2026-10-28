// Kanso — note_window_main.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import 'core/bridge/frb_generated.dart';
import 'core/configs/app_environment.dart';
import 'core/configs/core_module.dart';
import 'core/models/data/appearance_status.dart';
import 'core/state/appearance/appearance_bloc.dart';
import 'crash/models/repos/abs/crash_report_repository.dart';
import 'crash/utils/crash_capture.dart';
import 'injection.dart';
import 'window/models/data/note_window_launch.dart';
import 'window/models/repos/abs/note_window_repository.dart';
import 'window/views/widgets/note_window_app.dart';

/// The entry point of a note's own window: a second Flutter engine of this
/// process, started by the native side (an iPadOS scene, an Android task)
/// with [args] saying which note, and where the library is (see
/// [NoteWindowLaunch]).
///
/// It runs the same app services on the same library folder as the main
/// window, so both engines open the one library the Rust core keeps
/// open once per process: they share the lock session, and what is typed in
/// one shows in the other.
@pragma('vm:entry-point')
Future<void> noteWindowMain(List<String> args) async {
  final launch = NoteWindowLaunch.fromArgs(args);
  await bootstrapNoteWindow(launch);
}

/// Prepares the services of a note's window and runs it. With no note in
/// [launch] there is nothing to show, and the window closes.
///
/// [beforeRun] is awaited once the services are ready and before the first
/// frame.
Future<void> bootstrapNoteWindow(
  NoteWindowLaunch? launch, {
  Future<void> Function()? beforeRun,
}) async {
  WidgetsFlutterBinding.ensureInitialized();
  await KansoCore.init();
  final environment = AppEnvironment.values.firstWhere(
    (value) => value.name == launch?.environment,
    orElse: () => AppEnvironment.production,
  );
  final folder = launch?.libraryFolder ?? '';
  libraryFolderOverride = folder.isEmpty ? null : folder;
  await configureDependencies(environment.injectableName);
  if (environment != AppEnvironment.integration) {
    installCrashCapture(getIt<CrashReportRepository>());
  }
  final windows = getIt<NoteWindowRepository>();
  if (launch == null) {
    runApp(const SizedBox.shrink());
    await windows.closeThisWindow();
    return;
  }
  final appearance = getIt<AppearanceBloc>()
    ..add(const AppearanceEvent.started());
  await appearance.stream.firstWhere(
    (state) => state.status != AppearanceStatus.loading,
  );
  await beforeRun?.call();
  runApp(
    NoteWindowApp(
      noteId: launch.noteId,
      windows: windows,
      appearance: appearance,
    ),
  );
}
