// Kanso — url_launcher_web_link_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';
import 'package:url_launcher/url_launcher.dart';

import '../../../../core/models/data/application_error.dart';
import '../abs/web_link_repository.dart';

/// [WebLinkRepository] over the platform's `url_launcher`.
@LazySingleton(as: WebLinkRepository)
class UrlLauncherWebLinkRepository implements WebLinkRepository {
  /// The schemes a note may open.
  static const Set<String> schemes = <String>{'http', 'https', 'mailto'};

  @override
  Future<Either<ApplicationError, Unit>> open(String url) async {
    final uri = Uri.tryParse(url.trim());
    if (uri == null || !schemes.contains(uri.scheme.toLowerCase())) {
      return const Left<ApplicationError, Unit>(
        ApplicationError.invalid(message: 'Not a web or mail address.'),
      );
    }
    try {
      final opened = await launchUrl(uri, mode: LaunchMode.externalApplication);
      return opened
          ? const Right<ApplicationError, Unit>(unit)
          : const Left<ApplicationError, Unit>(
              ApplicationError.unexpected(message: 'Nothing opened it.'),
            );
    } on Object catch (error) {
      return Left<ApplicationError, Unit>(
        ApplicationError.unexpected(message: '$error'),
      );
    }
  }
}
