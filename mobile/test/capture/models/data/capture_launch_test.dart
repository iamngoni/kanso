// Kanso — capture_launch_test.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/capture/models/data/capture_launch.dart';
import 'package:kanso/capture_main.dart' as capture;
import 'package:kanso/main_development.dart' as development;
import 'package:kanso/main_production.dart' as production;

void main() {
  group('CaptureLaunch.fromArgs', () {
    test('no arguments are production in the app’s own library', () {
      expect(CaptureLaunch.fromArgs(const <String>[]), const CaptureLaunch());
      expect(
        CaptureLaunch.fromArgs(const <String>['', '']),
        const CaptureLaunch(),
      );
    });

    test('reads the environment, then the folder', () {
      final launch = CaptureLaunch.fromArgs(const <String>[
        'integration',
        '/data/tmp/library',
      ]);
      expect(launch.environment, 'integration');
      expect(launch.libraryFolder, '/data/tmp/library');
    });

    test('an environment alone keeps the app’s own folder', () {
      expect(
        CaptureLaunch.fromArgs(const <String>['development']),
        const CaptureLaunch(environment: 'development'),
      );
    });

    test('args are what fromArgs reads back', () {
      const launch = CaptureLaunch(
        environment: 'integration',
        libraryFolder: '/x',
      );
      expect(CaptureLaunch.fromArgs(launch.args), launch);
      expect(launch.hashCode, CaptureLaunch.fromArgs(launch.args).hashCode);
    });
  });

  test(
    'captureMain is the entry point both programs keep for the native side',
    () {
      // CaptureEngines.kt starts `captureMain` in
      // `package:kanso/capture_main.dart`; the exports keep it in the build.
      expect(production.captureMain, same(capture.captureMain));
      expect(development.captureMain, same(capture.captureMain));
    },
  );
}
