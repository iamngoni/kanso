// Kanso — push_token.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../../core/bridge/api/types.dart';

part 'push_token.freezed.dart';

/// The token the OS issued for waking this phone, and which push service and
/// environment it belongs to.
///
/// A token is how the push service finds this phone, so it is kept out of
/// logs: [toString] says that it is there, not what it is.
@Freezed(toStringOverride: false)
abstract class PushToken with _$PushToken {
  const factory PushToken({
    required PushPlatformKind platform,
    required String token,
    required PushEnvironmentKind environment,
  }) = _PushToken;

  const PushToken._();

  @override
  String toString() =>
      'PushToken(platform: ${platform.name}, '
      'environment: ${environment.name}, token: <hidden>)';
}
