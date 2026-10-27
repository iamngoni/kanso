// Kanso — fixture_push_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/push/models/data/push_token.dart';
import 'package:kanso/push/models/repos/abs/push_repository.dart';

/// A push service for widget tests: push works unless a test says why not,
/// the setting is what the test gives it, and nothing reaches an OS.
class FixturePushRepository implements PushRepository {
  FixturePushRepository({this.unavailable, this.enabled = true});

  /// Why push can't work here; null when it can.
  String? unavailable;

  /// Whether the person wants this phone woken.
  bool enabled;

  /// When set, the next change of the setting fails with it.
  ApplicationError? failure;

  final StreamController<PushToken> _tokens =
      StreamController<PushToken>.broadcast();

  @override
  Future<Either<ApplicationError, String?>> unavailableReason() async =>
      Right<ApplicationError, String?>(unavailable);

  @override
  Stream<PushToken> tokens() => _tokens.stream;

  @override
  Future<Either<ApplicationError, Unit>> register() async =>
      const Right<ApplicationError, Unit>(unit);

  @override
  Future<Either<ApplicationError, Unit>> setToken(PushToken token) async =>
      const Right<ApplicationError, Unit>(unit);

  @override
  Future<Either<ApplicationError, Unit>> refresh() async =>
      const Right<ApplicationError, Unit>(unit);

  @override
  Future<Either<ApplicationError, bool>> isEnabled() async =>
      Right<ApplicationError, bool>(enabled);

  @override
  Future<Either<ApplicationError, Unit>> setEnabled({
    required bool enabled,
  }) async {
    final error = failure;
    if (error != null) {
      return Left<ApplicationError, Unit>(error);
    }
    this.enabled = enabled;
    return const Right<ApplicationError, Unit>(unit);
  }
}
