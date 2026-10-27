// Kanso — editor_typing_performance_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:kanso/core/models/data/typing_report.dart';
import 'package:kanso/core/utils/typing_report.dart';

import 'helpers/flow_driver.dart';
import 'helpers/long_note.dart';
import 'helpers/typing_script.dart';
import 'helpers/typing_session.dart';

/// The lowest refresh rate the display may report, from
/// `--dart-define=KANSO_MIN_REFRESH_RATE=119`; see the scrolling test.
const int minRefreshRate = int.fromEnvironment('KANSO_MIN_REFRESH_RATE');

/// Whether to type the short script, whatever the mode, from
/// `--dart-define=KANSO_TYPING_QUICK=true`: to see how the numbers move with
/// the size of the note without waiting for the whole script.
const bool quickDefined = bool.fromEnvironment('KANSO_TYPING_QUICK');

/// Whether to record the timeline too (`--dart-define=KANSO_TYPING_TRACE=true`,
/// profile mode, with `flutter drive`): the driver writes it to
/// `build/editor_typing_timeline.json` for the slowest frames to be read.
const bool traceDefined = bool.fromEnvironment('KANSO_TYPING_TRACE');

/// How many words the note typed into has, from
/// `--dart-define=KANSO_TYPING_WORDS=20000`. Without it, 50,000 in profile
/// mode and 2,000 elsewhere.
const int wordsDefined = int.fromEnvironment('KANSO_TYPING_WORDS');

/// How typing into a long note keeps up with the display: the editor's
/// performance gate.
///
/// A note of 50,000 words is opened in Edit and a person's typing is
/// replayed into it (`TypingScript`): at the end and in the middle, steady
/// and in bursts, Return in a list, a paste, undos, and two minutes of steady
/// typing for the memory measure. The report (`TypingReport`) goes to
/// `flutter drive` through `test_driver/typing_driver.dart`, and to the log.
///
/// Only in profile mode, on a device, are the numbers held to the limits:
/// debug mode is slow on purpose and a simulator or emulator is not a phone.
/// There a short script runs on a small note, and the test only checks that
/// the report was made.
///
/// ```sh
/// flutter drive --profile \
///   --driver=test_driver/typing_driver.dart \
///   --target=integration_test/editor_typing_performance_test.dart \
///   -d <device> [--dart-define=KANSO_TYPING_WORDS=20000] \
///   [--dart-define=KANSO_MIN_REFRESH_RATE=119]
/// ```
void main() {
  final binding = IntegrationTestWidgetsFlutterBinding.ensureInitialized()
    ..framePolicy = LiveTestWidgetsFlutterBindingFramePolicy.fullyLive;

  testWidgets(
    'typing into a long note keeps up with the display',
    timeout: const Timeout(Duration(minutes: 30)),
    (tester) async {
      const words = wordsDefined > 0
          ? wordsDefined
          : (kProfileMode ? 50000 : 2000);
      const script = kProfileMode && !quickDefined
          ? TypingScript.full
          : TypingScript.quick;

      await startApp(tester);
      final id = await createLongNote(words);
      await openForTyping(id);
      late final TypingReport report;
      if (traceDefined) {
        await binding.traceAction(() async {
          report = await script.run(words: words);
        }, reportKey: 'typing_timeline');
      } else {
        report = await script.run(words: words);
      }

      final problems = typingProblems(
        report,
        minRefreshRate: minRefreshRate.toDouble(),
      );
      binding.reportData = <String, dynamic>{
        ...?binding.reportData,
        'typing_report': <String, Object>{
          ...report.toJson(),
          'mode': kProfileMode ? 'profile' : (kDebugMode ? 'debug' : 'release'),
          'fixtureHash': fixtureHash(longNote(words)),
          'os': Platform.operatingSystemVersion,
          'problems': problems,
        },
      };
      // ignore: avoid_print
      print(
        'Typing report: $words words, ${report.keystrokes} keystrokes, '
        '${report.frames.frames} frames at ${report.frames.refreshRate} Hz; '
        'keystroke to frame p50/p95/p99 ${report.latencyP50}/'
        '${report.latencyP95}/${report.latencyP99} ms; fresh highlight '
        'p50/p95 ${report.freshRunsP50}/${report.freshRunsP95} ms '
        '(${report.freshRunsTimed} timed); problems: $problems',
      );

      // The report is made in every mode.
      expect(report.keystrokes, greaterThan(0));
      expect(report.frames.frames, greaterThan(0));
      if (kProfileMode && !quickDefined) {
        expect(problems, isEmpty, reason: 'Typing did not keep up.');
      }
    },
  );
}
