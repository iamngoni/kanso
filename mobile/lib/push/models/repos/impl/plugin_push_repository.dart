// Kanso — plugin_push_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';
import 'package:kanso_push/kanso_push.dart';

import '../../../../core/bridge/api/sync.dart';
import '../../../../core/bridge/api/types.dart';
import '../../../../core/models/data/application_error.dart';
import '../../../../core/utils/kanso_error_mapping.dart';
import '../../data/push_token.dart';
import '../abs/push_repository.dart';

/// [PushRepository] over the `kanso_push` plugin, which asks the OS for the
/// token and tells Dart of it, and the Rust core, which keeps the token and
/// registers it with the server.
@LazySingleton(as: PushRepository)
class PluginPushRepository implements PushRepository, KansoPushFlutterApi {
  PluginPushRepository(this._host, this._sync);

  final KansoPushHostApi _host;
  final KansoSync _sync;
  final StreamController<PushToken> _tokens =
      StreamController<PushToken>.broadcast();

  @override
  Stream<PushToken> tokens() => _tokens.stream;

  /// The plugin's call: the OS issued [token] (or a new one). A service or
  /// environment this app doesn't know is dropped.
  @override
  void onToken(String platform, String token, String environment) {
    final service = switch (platform) {
      'apns' => PushPlatformKind.apns,
      'fcm' => PushPlatformKind.fcm,
      _ => null,
    };
    final sandbox = switch (environment) {
      'production' => PushEnvironmentKind.production,
      'development' => PushEnvironmentKind.development,
      _ => null,
    };
    if (service == null || sandbox == null || token.isEmpty) {
      return;
    }
    _tokens.add(
      PushToken(platform: service, token: token, environment: sandbox),
    );
  }

  @override
  Future<Either<ApplicationError, String?>> unavailableReason() =>
      attempt(() async => _host.unavailableReason());

  @override
  Future<Either<ApplicationError, Unit>> register() => attempt(() async {
    // Listening comes before asking, so no answer is missed.
    KansoPushFlutterApi.setUp(this);
    await _host.register();
    return unit;
  });

  @override
  Future<Either<ApplicationError, Unit>> setToken(PushToken token) =>
      attempt(() async {
        await _sync.setPushToken(
          platform: token.platform,
          token: token.token,
          environment: token.environment,
        );
        return unit;
      });

  @override
  Future<Either<ApplicationError, Unit>> refresh() => attempt(() async {
    await _sync.refreshPush();
    return unit;
  });

  @override
  Future<Either<ApplicationError, bool>> isEnabled() =>
      attempt(_sync.pushEnabled);

  @override
  Future<Either<ApplicationError, Unit>> setEnabled({
    required bool enabled,
  }) => attempt(() async {
    await _sync.setPushEnabled(enabled: enabled);
    return unit;
  });
}
