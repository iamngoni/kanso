// Kanso — locks_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ffi';

import 'package:flutter_test/flutter_test.dart';

import '../../helpers/biometric_answer.dart';
import '../../helpers/core_library.dart';
import 'suites/locks_suite.dart';

void main() {
  setUpAll(initCoreForTests);
  locksSuite(
    openTestLibrary,
    answer: (ticket, status, [secret]) => biometricAnswerOf(
      DynamicLibrary.open(builtCoreLibrary().path),
    )(ticket, status, secret),
  );
}
