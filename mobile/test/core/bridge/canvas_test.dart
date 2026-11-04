// Kanso — canvas_test.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';

import '../../helpers/core_library.dart';
import 'suites/canvas_suite.dart';

void main() {
  setUpAll(initCoreForTests);
  canvasSuite(openTestLibrary);
}
