// Kanso — editor_bench_app.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

// This is a bench tool; its output is printing.
// ignore_for_file: avoid_print

import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:kanso/bootstrap.dart';
import 'package:kanso/core/configs/app_environment.dart';
import 'package:kanso/core/models/repos/abs/preferences_repository.dart';
import 'package:kanso/core/utils/typing_report.dart';
import 'package:kanso/injection.dart';
import 'package:path_provider/path_provider.dart';

import 'helpers/bench_results.dart';
import 'helpers/long_note.dart';
import 'helpers/typing_script.dart';
import 'helpers/typing_session.dart';

/// How many words the note typed into has, from
/// `--dart-define=KANSO_TYPING_WORDS=20000`.
const int wordsDefined = int.fromEnvironment(
  'KANSO_TYPING_WORDS',
  defaultValue: 50000,
);

/// Whether to type the short script instead of the gate's, from
/// `--dart-define=KANSO_TYPING_QUICK=true`.
const bool quick = bool.fromEnvironment('KANSO_TYPING_QUICK');

/// The editor's typing check as an app of its own, for a phone that
/// `flutter drive` cannot reach (it needs the Dart VM service, which a phone
/// on a network may not offer): the same script as
/// `editor_typing_performance_test.dart`, on a library in a new temporary
/// folder.
///
/// It shows the numbers on screen when done, copies the JSON to the
/// clipboard, writes it to `editor_typing.json` in the app's documents
/// folder and prints it between `KANSO_TYPING_JSON` markers for
/// `flutter run` to show.
///
/// ```sh
/// flutter run --profile -t integration_test/editor_bench_app.dart \
///   -d <device> [--dart-define=KANSO_TYPING_WORDS=20000]
/// ```
Future<void> main() async {
  await bootstrap(
    AppEnvironment.integration,
    beforeRun: () async {
      // The notice of the first launch is not the bench's business.
      await getIt<PreferencesRepository>().update(
        (current) => current.copyWith(backupNoticeSeen: true),
      );
    },
  );
  // Let the app draw its first frames.
  await Future<void>.delayed(const Duration(seconds: 2));
  final id = await createLongNote(wordsDefined);
  await openForTyping(id);
  final report = await (quick ? TypingScript.quick : TypingScript.full).run(
    words: wordsDefined,
  );
  final problems = typingProblems(report);
  final json = const JsonEncoder.withIndent('  ').convert(<String, Object>{
    ...report.toJson(),
    'mode': kProfileMode ? 'profile' : (kDebugMode ? 'debug' : 'release'),
    'fixtureHash': fixtureHash(longNote(wordsDefined)),
    'os': Platform.operatingSystemVersion,
    'problems': problems,
  });
  print('KANSO_TYPING_JSON');
  print(json);
  print('KANSO_TYPING_JSON_END');
  try {
    final documents = await getApplicationDocumentsDirectory();
    await File('${documents.path}/editor_typing.json').writeAsString(json);
  } on Object catch (error) {
    print('The report could not be written: $error');
  }
  try {
    await Clipboard.setData(ClipboardData(text: json));
  } on Object catch (error) {
    print('The report could not be copied: $error');
  }
  runApp(BenchResults(report: report, problems: problems));
}
