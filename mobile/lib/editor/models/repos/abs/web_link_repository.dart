// Kanso — web_link_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/models/data/application_error.dart';

/// Opens an address from a note in the app that handles it: a web page in the
/// browser, a mail address in the mail app.
// One member, but a repository like the others.
// ignore: one_member_abstracts
abstract interface class WebLinkRepository {
  /// Opens [url]. An [InvalidError] for an address that is not a web page or
  /// a mail address: a note's text is not trusted to start other apps.
  Future<Either<ApplicationError, Unit>> open(String url);
}
