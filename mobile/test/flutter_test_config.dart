// Kanso — flutter_test_config.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';
import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'helpers/core_library.dart';
import 'helpers/kanso_golden_comparator.dart';

/// Loads the bundled fonts (the variable ones draw every weight from one
/// file) from their files before any test runs, so text
/// in widget tests and goldens is drawn in Kanso's fonts rather than the
/// test harness's placeholder font.
Future<void> testExecutable(FutureOr<void> Function() testMain) async {
  TestWidgetsFlutterBinding.ensureInitialized();
  await initCoreForTests();
  final comparator = goldenFileComparator;
  if (comparator is LocalFileComparator) {
    goldenFileComparator = KansoGoldenComparator(
      comparator.basedir.resolve('test.dart'),
    );
  }
  await _loadFamily('Instrument Sans', <String>[
    'InstrumentSans-Regular.ttf',
    'InstrumentSans-Medium.ttf',
    'InstrumentSans-SemiBold.ttf',
    'InstrumentSans-Bold.ttf',
    'InstrumentSans-Italic.ttf',
  ]);
  await _loadFamily('JetBrains Mono', <String>[
    'JetBrainsMono-Regular.ttf',
    'JetBrainsMono-Bold.ttf',
  ]);
  await _loadFamily('Inter', <String>[
    'Inter-Variable.ttf',
    'Inter-VariableItalic.ttf',
  ]);
  await _loadFamily('Source Serif 4', <String>[
    'SourceSerif4-Variable.ttf',
    'SourceSerif4-VariableItalic.ttf',
  ]);
  await _loadFamily('Literata', <String>[
    'Literata-Variable.ttf',
    'Literata-VariableItalic.ttf',
  ]);
  await _loadFamily('IBM Plex Mono', <String>[
    'IBMPlexMono-Regular.ttf',
    'IBMPlexMono-Medium.ttf',
    'IBMPlexMono-SemiBold.ttf',
    'IBMPlexMono-Bold.ttf',
    'IBMPlexMono-Italic.ttf',
  ]);
  await testMain();
}

Future<void> _loadFamily(String family, List<String> files) async {
  final loader = FontLoader(family);
  for (final file in files) {
    final bytes = await File('assets/fonts/$file').readAsBytes();
    loader.addFont(Future<ByteData>.value(ByteData.sublistView(bytes)));
  }
  await loader.load();
}
