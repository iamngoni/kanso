// Kanso — palette_repository.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/bridge/api/linking_types.dart';
import '../../../../core/models/data/application_error.dart';

/// The command palette: the commands the app can run, the places of the
/// library and, once something is typed, the notes Open Note would find.
// A one-call repository is what this feature needs; the shape is the app's.
// ignore: one_member_abstracts
abstract interface class PaletteRepository {
  /// What [query] matches, in the sections the palette shows them in.
  ///
  /// [context] says what the app can do now: which commands have a handler,
  /// the note that is open and the place being listed.
  Future<Either<ApplicationError, List<PaletteSection>>> search({
    required String query,
    required PaletteContext context,
  });
}
