// Kanso — system_share_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import 'package:dartz/dartz.dart';

import '../../data/application_error.dart';

/// Putting text where the person can use it elsewhere: the clipboard and
/// the system's share sheet.
abstract interface class SystemShareRepository {
  /// Copies [text] to the clipboard.
  Future<Either<ApplicationError, Unit>> copy(String text);

  /// Offers [text] to the apps the system's share sheet lists. On an iPad
  /// the sheet is a popover that points at [origin], the button that was
  /// pressed, in the screen's coordinates.
  Future<Either<ApplicationError, Unit>> send(String text, {Rect? origin});
}
