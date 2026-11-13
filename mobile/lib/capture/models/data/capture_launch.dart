// Kanso — capture_launch.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';

/// What the capture sheet over another app is started with on Android
/// (`CaptureEngines.kt`): the environment the app runs in and the library
/// folder, so that its engine opens the one library the app's does. The mode
/// is asked of the activity once it is on screen, as an engine started
/// ahead does not know it yet.
@immutable
class CaptureLaunch {
  const CaptureLaunch({
    this.environment = productionName,
    this.libraryFolder = '',
  });

  /// Reads the entry point's arguments: the environment, then the folder.
  /// Missing or empty ones are production and the app's own folder.
  factory CaptureLaunch.fromArgs(List<String> args) {
    final environment = args.isNotEmpty && args.first.isNotEmpty
        ? args.first
        : productionName;
    return CaptureLaunch(
      environment: environment,
      libraryFolder: args.length > 1 ? args[1] : '',
    );
  }

  /// The name of the production `AppEnvironment`.
  static const String productionName = 'production';

  /// The name of the `AppEnvironment` to run in.
  final String environment;

  /// The library folder; empty for the app's own.
  final String libraryFolder;

  /// The entry point's arguments, as [CaptureLaunch.fromArgs] reads them.
  List<String> get args => <String>[environment, libraryFolder];

  @override
  bool operator ==(Object other) =>
      other is CaptureLaunch &&
      other.environment == environment &&
      other.libraryFolder == libraryFolder;

  @override
  int get hashCode => Object.hash(environment, libraryFolder);

  @override
  String toString() =>
      'CaptureLaunch(environment: $environment, '
      'libraryFolder: $libraryFolder)';
}
