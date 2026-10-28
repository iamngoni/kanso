// Kanso — rust_templates_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/library.dart';
import '../../../bridge/api/types.dart';
import '../../../utils/kanso_error_mapping.dart';
import '../../data/application_error.dart';
import '../abs/templates_repository.dart';

/// [TemplatesRepository] over the Rust core's library.
@LazySingleton(as: TemplatesRepository)
class RustTemplatesRepository implements TemplatesRepository {
  RustTemplatesRepository(this._library);

  final KansoLibrary _library;

  @override
  Future<Either<ApplicationError, List<TemplateRow>>> templates() =>
      attempt(_library.templates);

  @override
  Future<Either<ApplicationError, MadeNote>> newFromTemplate({
    required String templateId,
    required String? bookId,
    required LocalTime now,
  }) => attempt(
    () => _library.newFromTemplate(
      templateId: templateId,
      bookId: bookId,
      now: now,
    ),
  );

  @override
  Future<Either<ApplicationError, MadeNote>> newTemplate() =>
      attempt(_library.newTemplate);

  @override
  Future<Either<ApplicationError, String?>> dailyTemplate() =>
      attempt(_library.dailyTemplate);

  @override
  Future<Either<ApplicationError, Unit>> setDailyTemplate(
    String? templateId,
  ) => attempt(() async {
    await _library.setDailyTemplate(templateId: templateId);
    return unit;
  });
}
