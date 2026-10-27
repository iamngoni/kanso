// Kanso — push_module.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:injectable/injectable.dart';
import 'package:kanso_push/kanso_push.dart';

/// The native push plugin's side of the channel.
@module
abstract class PushModule {
  @lazySingleton
  KansoPushHostApi hostApi() => KansoPushHostApi();
}
