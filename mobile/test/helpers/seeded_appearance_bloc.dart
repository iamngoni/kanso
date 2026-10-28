// Kanso — seeded_appearance_bloc.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/models/repos/abs/preferences_repository.dart';
import 'package:kanso/core/models/repos/abs/renderer_repository.dart';
import 'package:kanso/core/models/repos/abs/themes_repository.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/settings/state/backup_notice/backup_notice_bloc.dart';
import 'package:mocktail/mocktail.dart';

import 'fixture_gallery_repository.dart';
import 'fixture_themes_repository.dart';

class MockPreferencesRepository extends Mock implements PreferencesRepository {}

class MockRendererRepository extends Mock implements RendererRepository {}

class MockThemesRepository extends Mock implements ThemesRepository {}

/// A preferences repository that remembers nothing it is given back: loads
/// are the defaults and every update succeeds.
class ForgetfulPreferencesRepository implements PreferencesRepository {
  @override
  Future<Either<ApplicationError, Preferences>> load() async =>
      const Right<ApplicationError, Preferences>(Preferences());

  @override
  Future<Either<ApplicationError, Preferences>> update(
    Preferences Function(Preferences current) change,
  ) async => Right<ApplicationError, Preferences>(change(const Preferences()));
}

/// A preferences repository that starts with its seed and keeps what is saved
/// for the run.
class SeededPreferencesRepository implements PreferencesRepository {
  SeededPreferencesRepository(this.current);

  Preferences current;

  @override
  Future<Either<ApplicationError, Preferences>> load() async =>
      Right<ApplicationError, Preferences>(current);

  @override
  Future<Either<ApplicationError, Preferences>> update(
    Preferences Function(Preferences current) change,
  ) async {
    current = change(current);
    return Right<ApplicationError, Preferences>(current);
  }
}

/// An `AppearanceBloc` that starts in `seed` rather than the defaults, for
/// pages whose picture depends on the person's choices. It saves nothing and
/// has no themes to read besides Kanso's (see [FixtureThemesRepository]).
class SeededAppearanceBloc extends AppearanceBloc {
  SeededAppearanceBloc(AppearanceState seed)
    : super(
        ForgetfulPreferencesRepository(),
        MockRendererRepository(),
        FixtureGalleryRepository(),
        FixtureThemesRepository(),
      ) {
    emit(seed);
  }
}

/// A backup notice bloc that is not shown: it was not started, so it has
/// not read the preferences, and nothing covers the app in a test.
BackupNoticeBloc hiddenBackupNoticeBloc() =>
    BackupNoticeBloc(ForgetfulPreferencesRepository());
