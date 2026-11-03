// Kanso — flutter_test_config.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';
import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/notebook_home/models/repos/abs/notebook_home_repository.dart';

import 'helpers/core_library.dart';
import 'helpers/fake_notebook_home_repository.dart';
import 'helpers/kanso_golden_comparator.dart';

/// Loads the bundled fonts (the variable ones draw every weight from one
/// file) from their files before any test runs, so text
/// in widget tests and goldens is drawn in Kanso's fonts rather than the
/// test harness's placeholder font.
///
/// It also cleans up after a failed widget test, so that its tear-downs end
/// and the next test runs; see [_afterFailure].
Future<void> testExecutable(FutureOr<void> Function() testMain) async {
  TestWidgetsFlutterBinding.ensureInitialized();
  final report = reportTestException;
  reportTestException = (details, description) {
    report(details, description);
    _afterFailure();
  };
  // Every test's tear-downs are over by now.
  tearDown(_stopDraining);
  // A notebook's list asks for its homepage card: unless a test says what the
  // card is, there is none.
  setUp(() {
    if (!getIt.isRegistered<NotebookHomeRepository>()) {
      getIt.registerSingleton<NotebookHomeRepository>(
        FakeNotebookHomeRepository(),
      );
    }
  });
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

/// Cleans up after a failed widget test, so that its tear-downs end.
///
/// A widget test runs in a fake-async zone: a timer, a microtask, and the
/// callbacks of a future made there (even one already complete) are queued
/// on that zone's fake clock, which only the test advances. The framework
/// runs the test's tear-downs after the body, outside the zone, with the
/// clock no longer advanced. When a test passes, that is harmless: the
/// framework first unmounts its widgets while the clock still runs, so the
/// blocs they own are closed and their listeners cancelled there, and a
/// tear-down finds nothing left waiting on the clock.
///
/// When a test fails, the framework leaves everything as it was. A
/// tear-down that awaits `bloc.close()` then waits on work queued on the
/// stopped clock: an event handler still running, a stream's done event
/// for a listener a widget still holds, a future made in the zone. It never
/// returns, the next test never starts, and the run hangs instead of
/// reporting the failure.
///
/// So when a failure is reported (still inside the zone), the widgets are
/// unmounted, as for a test that passes, and the fake clock's queue is
/// drained from outside the zone until the tear-downs are over.
void _afterFailure() {
  _unmountWidgets();
  _drainFakeClock();
}

/// Unmounts the widgets of the test, while the fake clock runs. An error
/// while they are taken down is printed, not raised: the test has already
/// failed, with its own error.
void _unmountWidgets() {
  final binding = TestWidgetsFlutterBinding.instance;
  if (binding.rootElement == null) {
    return;
  }
  final onError = FlutterError.onError;
  FlutterError.onError = (details) => debugPrint(
    'While removing the widgets of a failed test: ${details.exception}',
  );
  try {
    binding.attachRootWidget(
      binding.wrapWithDefaultView(const SizedBox.shrink()),
    );
    binding.buildOwner!
      ..buildScope(binding.rootElement!)
      ..finalizeTree();
  } on Object catch (error) {
    debugPrint('While removing the widgets of a failed test: $error');
  } finally {
    FlutterError.onError = onError;
  }
}

/// The real timer that drains the fake clock of a failed test.
Timer? _draining;

/// Runs what is due on the test's fake clock, from a real timer, until the
/// test's tear-downs are over ([_stopDraining]) or the clock is gone.
void _drainFakeClock() {
  final binding = TestWidgetsFlutterBinding.instance;
  _draining?.cancel();
  _draining = Zone.root.createPeriodicTimer(
    const Duration(milliseconds: 1),
    (timer) {
      try {
        // Runs the microtasks and the timers due now, in their zone.
        unawaited(binding.delayed(Duration.zero));
      } on Object {
        // The clock is gone with the test.
        timer.cancel();
      }
    },
  );
}

void _stopDraining() {
  _draining?.cancel();
  _draining = null;
}
