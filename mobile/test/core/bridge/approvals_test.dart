// Kanso — approvals_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';

import '../../helpers/core_library.dart';
import 'suites/approvals_suite.dart';

void main() {
  setUpAll(initCoreForTests);
  approvalsSuite(openTestLibrary);
}
