// Kanso — widget_colors_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../configs/kanso_colors.dart';
import '../../data/application_error.dart';

/// The colours the home-screen widgets draw with.
// ignore: one_member_abstracts
abstract interface class WidgetColorsRepository {
  /// Writes `widget.json` for the [light] and [dark] colours and asks for the
  /// widgets to be drawn again. Colours the file already holds change
  /// nothing. True when the file was written.
  Future<Either<ApplicationError, bool>> publish({
    required KansoColors light,
    required KansoColors dark,
  });
}
