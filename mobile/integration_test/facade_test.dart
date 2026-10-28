// Kanso — facade_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ffi';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:kanso/core/bridge/api/library.dart';
import 'package:kanso/core/bridge/api/sync.dart';
import 'package:kanso/core/bridge/frb_generated.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';

import '../test/core/bridge/suites/account_settings_suite.dart';
import '../test/core/bridge/suites/account_suite.dart';
import '../test/core/bridge/suites/ai_suite.dart';
import '../test/core/bridge/suites/approvals_suite.dart';
import '../test/core/bridge/suites/boards_suite.dart';
import '../test/core/bridge/suites/diagnostics_suite.dart';
import '../test/core/bridge/suites/discovery_suite.dart';
import '../test/core/bridge/suites/editor_suite.dart';
import '../test/core/bridge/suites/gallery_suite.dart';
import '../test/core/bridge/suites/journal_suite.dart';
import '../test/core/bridge/suites/library_suite.dart';
import '../test/core/bridge/suites/linking_suite.dart';
import '../test/core/bridge/suites/locks_suite.dart';
import '../test/core/bridge/suites/publish_suite.dart';
import '../test/core/bridge/suites/reminders_suite.dart';
import '../test/core/bridge/suites/renderer_suite.dart';
import '../test/core/bridge/suites/shared_suite.dart';
import '../test/core/bridge/suites/shortcuts_suite.dart';
import '../test/core/bridge/suites/sync_suite.dart';
import '../test/core/bridge/suites/templates_suite.dart';
import '../test/core/bridge/suites/themes_suite.dart';
import '../test/core/bridge/suites/voice_suite.dart';
import '../test/helpers/biometric_answer.dart';

/// Every call of the bridge, on a simulator or an emulator: the library the
/// app ships, loaded the way the app loads it, with the platform's own secret
/// store (the Keychain, the Android Keystore through JNI) and certificate
/// verifier.
///
/// Run it with `flutter test integration_test/facade_test.dart -d <device>`.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  // The library is in the app: the default loader finds it. On Android the
  // `Application` has already handed over its Context (KansoApplication).
  setUpAll(KansoCore.init);

  Future<KansoLibrary> openDeviceLibrary() async {
    final root = await getTemporaryDirectory();
    final folder = await root.createTemp('kanso_facade_');
    addTearDown(() => folder.delete(recursive: true));
    return KansoLibrary.open(path: p.join(folder.path, 'library.db'));
  }

  librarySuite(openDeviceLibrary);
  linkingSuite(openDeviceLibrary);
  locksSuite(
    openDeviceLibrary,
    answer: biometricAnswerOf(
      Platform.isAndroid
          ? DynamicLibrary.open('libkanso_mobile_bridge.so')
          : DynamicLibrary.process(),
    ),
  );
  syncSuite(openDeviceLibrary);
  rendererSuite(openDeviceLibrary);
  themesSuite(openDeviceLibrary);
  diagnosticsSuite(openDeviceLibrary);
  accountSuite(openDeviceLibrary);
  accountSettingsSuite(openDeviceLibrary);
  aiSuite(openDeviceLibrary);
  approvalsSuite(openDeviceLibrary);
  sharedSuite(openDeviceLibrary);
  gallerySuite(openDeviceLibrary);
  editorSuite(openDeviceLibrary);
  boardsSuite(openDeviceLibrary);
  discoverySuite(openDeviceLibrary);
  journalSuite(openDeviceLibrary);
  templatesSuite(openDeviceLibrary);
  voiceSuite(openDeviceLibrary);
  publishSuite(openDeviceLibrary);
  remindersSuite(openDeviceLibrary);
  shortcutsSuite(openDeviceLibrary);

  group('the platform verifier', () {
    test(
      "Kanso's default sync server answers over https",
      () async {
        final sync = (await openDeviceLibrary()).syncing();
        final server = await KansoSync.defaultServer();
        expect(server, startsWith('https://'));
        // A real request: the server's certificate is checked by the
        // platform's verifier (on Android, through JNI).
        final info = await sync.checkServer(url: server);
        expect(info.signupsOpen, isA<bool>());
      },
      skip: Platform.isAndroid
          ? false
          : 'Checked on Android, where the verifier is Kotlin reached by JNI.',
    );
  });
}
