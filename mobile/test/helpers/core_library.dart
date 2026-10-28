// Kanso — core_library.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart'
    show ExternalLibrary;
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/library.dart';
import 'package:kanso/core/bridge/frb_generated.dart';
import 'package:path/path.dart' as p;

/// Loads the Rust library for a test that runs on this computer.
///
/// `flutter test` runs the build hook (`hook/build.dart`) before the tests, but
/// does not bundle its output the way an app build does, so the default
/// loader can't find it. This opens the library the hook built, the newest
/// if there are several, or the file named by `KANSO_CORE_LIBRARY`.
///
/// On a phone, a simulator or an emulator (an integration test) the library
/// is in the app, and the default loader finds it.
Future<void> initCoreForTests() async {
  if (KansoCore.instance.initialized) {
    return;
  }
  if (Platform.isIOS || Platform.isAndroid) {
    await KansoCore.init();
    return;
  }
  await KansoCore.init(
    externalLibrary: ExternalLibrary.open(_builtLibrary().path),
  );
}

/// The file [initCoreForTests] loads, for a test that calls the library's C
/// entries (`kanso_biometric_answer`) directly.
File builtCoreLibrary() => _builtLibrary();

/// A library of its own, empty, in a new temporary folder that is removed
/// when the test ends. On a computer its secrets are in memory: never the
/// developer's own Keychain.
Future<KansoLibrary> openTestLibrary() async {
  final folder = await Directory.systemTemp.createTemp('kanso_core_');
  addTearDown(() => folder.delete(recursive: true));
  return KansoLibrary.open(path: p.join(folder.path, 'library.db'));
}

File _builtLibrary() {
  final named = Platform.environment['KANSO_CORE_LIBRARY'];
  if (named != null && named.isNotEmpty) {
    return File(named);
  }
  final extension = Platform.isMacOS
      ? 'dylib'
      : Platform.isWindows
      ? 'dll'
      : 'so';
  final prefix = Platform.isWindows ? '' : 'lib';
  final name = '${prefix}kanso_mobile_bridge.$extension';
  final hooks = Directory('.dart_tool/hooks_runner/shared/kanso/build');
  final found = <File>[
    if (hooks.existsSync())
      for (final entity in hooks.listSync(recursive: true))
        if (entity is File &&
            entity.uri.pathSegments.last == name &&
            !_isPhoneBuild(entity))
          entity,
  ]..sort((a, b) => b.lastModifiedSync().compareTo(a.lastModifiedSync()));
  if (found.isEmpty) {
    throw StateError(
      'The Rust library was not built: run `flutter test` from mobile/, '
      'which runs hook/build.dart, or set KANSO_CORE_LIBRARY.',
    );
  }
  return found.first;
}

/// Whether [file] was built for a phone or its simulator (an app build puts
/// those beside the computer's), which this computer can't load.
bool _isPhoneBuild(File file) => file.uri.pathSegments.any(
  (segment) => segment.contains('-ios') || segment.contains('-android'),
);
