// Kanso — rust_palette_repository.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/library.dart';
import '../../../../core/bridge/api/linking_types.dart';
import '../../../../core/models/data/application_error.dart';
import '../../../../core/utils/kanso_error_mapping.dart';
import '../abs/palette_repository.dart';

/// [PaletteRepository] over the Rust core's linking.
@LazySingleton(as: PaletteRepository)
class RustPaletteRepository implements PaletteRepository {
  RustPaletteRepository(this._library);

  final KansoLibrary _library;

  @override
  Future<Either<ApplicationError, List<PaletteSection>>> search({
    required String query,
    required PaletteContext context,
  }) => attempt(
    () => _library.linking().palette(query: query, context: context),
  );
}
