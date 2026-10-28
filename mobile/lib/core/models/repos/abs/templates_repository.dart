// Kanso — templates_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../bridge/api/types.dart';
import '../../data/application_error.dart';

/// The notes in the Templates notebook, and the template new daily notes
/// start from.
abstract interface class TemplatesRepository {
  Future<Either<ApplicationError, List<TemplateRow>>> templates();

  /// A note from the template [templateId] in the notebook [bookId]; the
  /// core picks the Inbox when it is null.
  Future<Either<ApplicationError, MadeNote>> newFromTemplate({
    required String templateId,
    required String? bookId,
    required LocalTime now,
  });

  /// An empty note in the Templates notebook (made if needed), to write a
  /// template in.
  Future<Either<ApplicationError, MadeNote>> newTemplate();

  /// The template daily notes start from; null for none.
  Future<Either<ApplicationError, String?>> dailyTemplate();

  Future<Either<ApplicationError, Unit>> setDailyTemplate(String? templateId);
}
