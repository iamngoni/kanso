// Kanso — package_info_about_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/foundation.dart';
import 'package:injectable/injectable.dart';
import 'package:package_info_plus/package_info_plus.dart';
import 'package:url_launcher/url_launcher.dart';

import '../../../configs/kanso_links.dart';
import '../../../configs/kanso_stores.dart';
import '../../data/app_version.dart';
import '../../data/application_error.dart';
import '../abs/about_repository.dart';
import '../abs/patch_repository.dart';
import 'shorebird_patch_repository.dart';

/// The app's version from the platform's package information, and the
/// website's pages opened in the browser.
@LazySingleton(as: AboutRepository)
class PackageInfoAboutRepository implements AboutRepository {
  PackageInfoAboutRepository([
    PatchRepository? patches,
    @ignoreParam this._platform,
  ]) : _patches = patches ?? ShorebirdPatchRepository();

  final PatchRepository _patches;

  /// The platform whose store to open; the running one when null.
  final TargetPlatform? _platform;

  @override
  Future<Either<ApplicationError, AppVersion>> version() async {
    try {
      final info = await PackageInfo.fromPlatform();
      return Right<ApplicationError, AppVersion>(
        AppVersion(
          version: info.version,
          build: info.buildNumber,
          patch: await _patches.currentPatch(),
        ),
      );
    } on Object catch (error) {
      return Left<ApplicationError, AppVersion>(
        ApplicationError.unexpected(message: '$error'),
      );
    }
  }

  @override
  Future<Either<ApplicationError, Unit>> openWebsite() =>
      _open(KansoLinks.website);

  @override
  Future<Either<ApplicationError, Unit>> openReleaseNotes() =>
      _open(KansoLinks.releaseNotes);

  @override
  Future<Either<ApplicationError, Unit>> openHelp() => _open(KansoLinks.help);

  @override
  Future<Either<ApplicationError, Unit>> openRatePage() async {
    final page = KansoStores.writeReviewFor(_platform ?? defaultTargetPlatform);
    if (page == null) {
      return const Left<ApplicationError, Unit>(
        ApplicationError.unexpected(message: 'No store page to rate the app'),
      );
    }
    return _open(page);
  }

  Future<Either<ApplicationError, Unit>> _open(Uri address) async {
    try {
      final opened = await launchUrl(
        address,
        mode: LaunchMode.externalApplication,
      );
      return opened
          ? const Right<ApplicationError, Unit>(unit)
          : const Left<ApplicationError, Unit>(
              ApplicationError.unexpected(message: 'The browser did not open'),
            );
    } on Object catch (error) {
      return Left<ApplicationError, Unit>(
        ApplicationError.unexpected(message: '$error'),
      );
    }
  }
}
