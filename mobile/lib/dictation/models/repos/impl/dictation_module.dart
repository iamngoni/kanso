// Kanso — dictation_module.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:injectable/injectable.dart';
import 'package:kanso_dictation/kanso_dictation.dart';

/// The plugin, once for the app: it owns the channel the native side sends
/// its events on.
@module
abstract class DictationModule {
  @lazySingleton
  KansoDictation get kansoDictation => KansoDictation();
}
