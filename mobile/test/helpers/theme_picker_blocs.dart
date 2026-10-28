// Kanso — theme_picker_blocs.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:kanso/core/bridge/api/types.dart' show GalleryItem;
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/core/models/repos/abs/file_picker_repository.dart';
import 'package:kanso/themes/state/theme_gallery/theme_gallery_bloc.dart';
import 'package:kanso/themes/state/theme_picker/theme_picker_bloc.dart';

import 'close_bloc_later.dart';
import 'fixture_file_picker_repository.dart';
import 'fixture_gallery_repository.dart';
import 'fixture_themes_repository.dart';
import 'real_themes.dart';

/// A fixture gallery with nothing in it, for tests of the picker that are
/// not about the gallery.
FixtureGalleryRepository emptyGallery() =>
    FixtureGalleryRepository(items: <GalleryItem>[]);

/// A `ThemePickerBloc` that has read the themes in [themes], and the file
/// picker [files] will answer to imports.
Future<ThemePickerBloc> startThemePicker({
  FixtureThemesRepository? themes,
  FilePickerRepository? files,
}) async {
  final picker = ThemePickerBloc(
    themes ?? realThemesRepository(),
    files ?? FixtureFilePickerRepository(),
  )..add(const ThemePickerEvent.started());
  closeBlocLater(picker);
  await picker.stream.firstWhere((state) => state.status == LoadStatus.ready);
  return picker;
}

/// A `ThemeGalleryBloc` over [gallery], started; and, unless [wait] is
/// false, past loading.
Future<ThemeGalleryBloc> startThemeGallery(
  FixtureGalleryRepository gallery, {
  bool wait = true,
}) async {
  final bloc = ThemeGalleryBloc(gallery)
    ..add(const ThemeGalleryEvent.started());
  closeBlocLater(bloc);
  if (wait) {
    await bloc.stream.firstWhere(
      (state) =>
          state.status != LoadStatus.loading &&
          !state.refreshing &&
          state.working.isEmpty,
    );
  }
  return bloc;
}

/// [child] with a picker and a gallery above it, over [gallery] (which has
/// nothing in it when a test gives none). With [waitForGallery] false the
/// gallery is left as it is while it is read.
Future<Widget> withThemePicker(
  Widget child, {
  FixtureGalleryRepository? gallery,
  FixtureThemesRepository? themes,
  FilePickerRepository? files,
  bool waitForGallery = true,
}) async {
  final galleryRepository = gallery ?? emptyGallery();
  return MultiBlocProvider(
    providers: <BlocProvider<dynamic>>[
      BlocProvider<ThemePickerBloc>.value(
        value: await startThemePicker(themes: themes, files: files),
      ),
      BlocProvider<ThemeGalleryBloc>.value(
        value: await startThemeGallery(
          galleryRepository,
          wait: waitForGallery,
        ),
      ),
    ],
    child: child,
  );
}
