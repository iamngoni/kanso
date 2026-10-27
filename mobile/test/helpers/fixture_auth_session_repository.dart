// Kanso — fixture_auth_session_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/auth_session_repository.dart';

/// A sign-in session service for widget tests: the pages it was asked to
/// open, and how often it was closed. It answers every page with null, as if
/// the person had dismissed it.
class FixtureAuthSessionRepository implements AuthSessionRepository {
  /// The pages opened, in order.
  final List<String> opened = <String>[];

  /// How many times the session was closed.
  int closes = 0;

  @override
  Future<Either<ApplicationError, String?>> open(
    String url, {
    String? callbackScheme,
  }) async {
    opened.add(url);
    return const Right<ApplicationError, String?>(null);
  }

  @override
  Future<Either<ApplicationError, Unit>> close() async {
    closes++;
    return const Right<ApplicationError, Unit>(unit);
  }
}
