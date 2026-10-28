// Kanso — main_production.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'bootstrap.dart';
import 'core/configs/app_environment.dart';

// A note's own window starts at this entry point (see note_window_main.dart);
// exporting it keeps it in the program the native sides start an engine from.
export 'note_window_main.dart' show noteWindowMain;

Future<void> main() => bootstrap(AppEnvironment.production);
