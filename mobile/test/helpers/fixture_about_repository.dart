// Kanso — fixture_about_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso/core/models/data/app_version.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/about_repository.dart';

/// An About repository for widget tests: version 0.1.0 build 3, and a
/// website, release notes, help page and store page that record how often
/// they were opened.
class FixtureAboutRepository implements AboutRepository {
  /// How many times the website was opened.
  int websiteOpens = 0;

  /// How many times the release notes were opened.
  int releaseNotesOpens = 0;

  /// How many times the help page was opened.
  int helpOpens = 0;

  /// How many times the store's page to rate the app was opened.
  int rateOpens = 0;

  @override
  Future<Either<ApplicationError, AppVersion>> version() async =>
      const Right<ApplicationError, AppVersion>(
        AppVersion(version: '0.1.0', build: '3'),
      );

  @override
  Future<Either<ApplicationError, Unit>> openWebsite() async {
    websiteOpens++;
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, Unit>> openReleaseNotes() async {
    releaseNotesOpens++;
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, Unit>> openHelp() async {
    helpOpens++;
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, Unit>> openRatePage() async {
    rateOpens++;
    return const Right<ApplicationError, Unit>(unit);
  }
}
