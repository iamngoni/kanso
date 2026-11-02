// Kanso — brief_test.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';

import '../../helpers/core_library.dart';
import 'suites/brief_suite.dart';

void main() {
  setUpAll(initCoreForTests);
  briefSuite(openTestLibrary);
}
