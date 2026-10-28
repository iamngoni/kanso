// Kanso — plugin_auth_session_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';
import 'package:kanso_auth_session/kanso_auth_session.dart';

import '../../../utils/kanso_error_mapping.dart';
import '../../data/application_error.dart';
import '../abs/auth_session_repository.dart';

/// [AuthSessionRepository] over the `kanso_auth_session` plugin:
/// `ASWebAuthenticationSession` on iOS, a Custom Tab on Android.
@LazySingleton(as: AuthSessionRepository)
class PluginAuthSessionRepository implements AuthSessionRepository {
  @override
  Future<Either<ApplicationError, String?>> open(
    String url, {
    String? callbackScheme,
  }) =>
      attempt(() => KansoAuthSession.open(url, callbackScheme: callbackScheme));

  @override
  Future<Either<ApplicationError, Unit>> close() => attempt(() async {
    await KansoAuthSession.close();
    return unit;
  });
}
