// Kanso — main_development.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'bootstrap.dart';
import 'core/configs/app_environment.dart';

// The capture sheet over another app on Android starts at this entry point
// (see capture_main.dart), and a note's own window at the next (see
// note_window_main.dart); exporting them keeps them in the program the
// native sides start an engine from.
export 'capture_main.dart' show captureMain;
export 'note_window_main.dart' show noteWindowMain;

Future<void> main() => bootstrap(AppEnvironment.development);
