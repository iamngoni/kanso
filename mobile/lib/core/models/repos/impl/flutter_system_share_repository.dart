// Kanso — flutter_system_share_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/services.dart';
import 'package:injectable/injectable.dart';
import 'package:share_plus/share_plus.dart';

import '../../data/application_error.dart';
import '../abs/system_share_repository.dart';

/// [SystemShareRepository] over Flutter's clipboard and `share_plus`.
@LazySingleton(as: SystemShareRepository)
class FlutterSystemShareRepository implements SystemShareRepository {
  @override
  Future<Either<ApplicationError, Unit>> copy(String text) async {
    try {
      await Clipboard.setData(ClipboardData(text: text));
      return const Right<ApplicationError, Unit>(unit);
    } on Object catch (error) {
      return Left<ApplicationError, Unit>(
        ApplicationError.unexpected(message: '$error'),
      );
    }
  }

  @override
  Future<Either<ApplicationError, Unit>> send(
    String text, {
    Rect? origin,
  }) async {
    try {
      await SharePlus.instance.share(
        ShareParams(text: text, sharePositionOrigin: origin),
      );
      return const Right<ApplicationError, Unit>(unit);
    } on Object catch (error) {
      return Left<ApplicationError, Unit>(
        ApplicationError.unexpected(message: '$error'),
      );
    }
  }
}
