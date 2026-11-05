// Kanso — pdf_test.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';

import '../../helpers/core_library.dart';
import 'suites/pdf_suite.dart';

void main() {
  setUpAll(initCoreForTests);
  pdfSuite(openTestLibrary);
}
