// Kanso — register_page_blocs.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter_test/flutter_test.dart' show addTearDown;
import 'package:kanso/injection.dart';
import 'package:kanso/note/state/note/note_bloc.dart';
import 'package:kanso/note_list/state/note_list/note_list_bloc.dart';
import 'package:kanso/search/state/search/search_bloc.dart';
import 'package:kanso/settings/state/about/about_bloc.dart';
import 'package:kanso/settings/state/licences/licences_bloc.dart';
import 'package:kanso/themes/state/icon_themes/icon_themes_bloc.dart';
import 'package:kanso/themes/state/theme_picker/theme_picker_bloc.dart';

import 'browse_blocs.dart';
import 'fixture_about_repository.dart';
import 'fixture_file_picker_repository.dart';
import 'fixture_library_repository.dart';
import 'fixture_licences_repository.dart';
import 'fixture_renderer_repository.dart';
import 'fixture_themes_repository.dart';
import 'seeded_appearance_bloc.dart';

/// Registers, for a test that builds the app's pages, the BLoCs the pages
/// make for themselves, over [library]; they are taken out again when the
/// test ends.
///
/// The theme pages read [themes] (Kanso's own two by default) and import
/// from [files] (a picker that chooses nothing by default).
void registerPageBlocs(
  FixtureLibraryRepository library, [
  FixtureRendererRepository? renderer,
  FixtureThemesRepository? themes,
  FixtureFilePickerRepository? files,
]) {
  final haptics = RecordingHaptics();
  if (getIt.isRegistered<NoteListBloc>()) {
    getIt.unregister<NoteListBloc>();
  }
  getIt.registerFactory<NoteListBloc>(
    () => NoteListBloc(library, ForgetfulPreferencesRepository(), haptics),
  );
  if (getIt.isRegistered<NoteBloc>()) {
    getIt.unregister<NoteBloc>();
  }
  getIt.registerFactory<NoteBloc>(
    () => NoteBloc(library, renderer ?? FixtureRendererRepository()),
  );
  if (getIt.isRegistered<SearchBloc>()) {
    getIt.unregister<SearchBloc>();
  }
  getIt.registerFactory<SearchBloc>(() => SearchBloc(library));
  if (getIt.isRegistered<AboutBloc>()) {
    getIt.unregister<AboutBloc>();
  }
  getIt.registerFactory<AboutBloc>(() => AboutBloc(FixtureAboutRepository()));
  if (getIt.isRegistered<LicencesBloc>()) {
    getIt.unregister<LicencesBloc>();
  }
  getIt.registerFactory<LicencesBloc>(
    () => LicencesBloc(FixtureLicencesRepository()),
  );
  if (getIt.isRegistered<ThemePickerBloc>()) {
    getIt.unregister<ThemePickerBloc>();
  }
  final themeRepository = themes ?? FixtureThemesRepository();
  final filePicker = files ?? FixtureFilePickerRepository();
  getIt.registerFactory<ThemePickerBloc>(
    () => ThemePickerBloc(themeRepository, filePicker),
  );
  if (getIt.isRegistered<IconThemesBloc>()) {
    getIt.unregister<IconThemesBloc>();
  }
  getIt.registerFactory<IconThemesBloc>(
    () => IconThemesBloc(themeRepository, filePicker),
  );
  addTearDown(() => unawaited(getIt.reset()));
}
