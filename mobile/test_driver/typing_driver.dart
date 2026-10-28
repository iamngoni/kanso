// Kanso — typing_driver.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

// This is a command-line tool; its output is printing.
// ignore_for_file: avoid_print

import 'dart:convert';
import 'dart:io';

import 'package:integration_test/integration_test_driver.dart';

/// Where the typing report is written, from `mobile/`.
const String reportPath = 'build/editor_typing.json';

/// Where the timeline is written when the test recorded one.
const String timelinePath = 'build/editor_typing_timeline.json';

/// The driver of `integration_test/editor_typing_performance_test.dart` for
/// `flutter drive`: it prints the test's typing report and writes it to
/// [reportPath]. The test itself decides whether the numbers pass.
///
/// ```sh
/// flutter drive --profile \
///   --driver=test_driver/typing_driver.dart \
///   --target=integration_test/editor_typing_performance_test.dart \
///   -d <device>
/// ```
Future<void> main() {
  return integrationDriver(
    writeResponseOnFailure: true,
    responseDataCallback: (data) async {
      final timeline = data?['typing_timeline'];
      if (timeline != null) {
        File(timelinePath)
          ..createSync(recursive: true)
          ..writeAsStringSync(jsonEncode(timeline));
        print('timeline written to $timelinePath');
      }
      final report = data?['typing_report'];
      if (report is! Map<String, dynamic>) {
        print('The test did not report any typing.');
        return;
      }
      File(reportPath)
        ..createSync(recursive: true)
        ..writeAsStringSync(const JsonEncoder.withIndent('  ').convert(report));
      print(const JsonEncoder.withIndent('  ').convert(report));
      print('written to $reportPath');
    },
  );
}
