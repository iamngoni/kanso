// Kanso — fixture_system_share_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import 'package:dartz/dartz.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/system_share_repository.dart';

/// The clipboard and the system's share sheet for widget tests: they keep
/// what they were given.
class FixtureSystemShareRepository implements SystemShareRepository {
  /// What was copied, in order.
  final List<String> copied = <String>[];

  /// What was sent to the share sheet, in order, and where it pointed.
  final List<String> sent = <String>[];
  final List<Rect?> origins = <Rect?>[];

  @override
  Future<Either<ApplicationError, Unit>> copy(String text) async {
    copied.add(text);
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, Unit>> send(
    String text, {
    Rect? origin,
  }) async {
    sent.add(text);
    origins.add(origin);
    return const Right<ApplicationError, Unit>(unit);
  }
}
