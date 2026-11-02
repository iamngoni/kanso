// Kanso — about_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../data/app_version.dart';
import '../../data/application_error.dart';

/// Facts about the app and the ways out of it that the About section needs.
abstract interface class AboutRepository {
  /// The version and build of the running app.
  Future<Either<ApplicationError, AppVersion>> version();

  /// Opens Kanso's website in the browser.
  Future<Either<ApplicationError, Unit>> openWebsite();

  /// Opens the website's release notes in the browser.
  Future<Either<ApplicationError, Unit>> openReleaseNotes();

  /// Opens the website's help page in the browser.
  Future<Either<ApplicationError, Unit>> openHelp();

  /// Opens the store's page to write a review of the app (the App Store,
  /// or Google Play once the listing is live). Fails where there is none.
  Future<Either<ApplicationError, Unit>> openRatePage();
}
