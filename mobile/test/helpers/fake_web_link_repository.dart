// Kanso — fake_web_link_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/editor/models/repos/abs/web_link_repository.dart';

/// A [WebLinkRepository] that opens nothing and remembers what it was asked
/// to open.
class FakeWebLinkRepository implements WebLinkRepository {
  final List<String> opened = <String>[];

  /// What opening answers.
  Either<ApplicationError, Unit> answer = const Right<ApplicationError, Unit>(
    unit,
  );

  @override
  Future<Either<ApplicationError, Unit>> open(String url) async {
    opened.add(url);
    return answer;
  }
}
