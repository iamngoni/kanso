// Kanso — theme_picker_blocs.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/themes/state/theme_gallery/theme_gallery_bloc.dart';
import 'package:kanso/themes/state/theme_picker/theme_picker_bloc.dart';

import 'close_bloc_later.dart';
import 'fixture_gallery_repository.dart';
import 'real_themes.dart';

/// A fixture gallery with nothing in it, for tests of the picker that are
/// not about the gallery.
FixtureGalleryRepository emptyGallery() =>
    FixtureGalleryRepository(items: <GalleryItem>[]);

/// A `ThemePickerBloc` that has read the real built-in themes (see
/// [loadRealThemes]) and the themes installed in [gallery].
Future<ThemePickerBloc> startThemePicker([
  FixtureGalleryRepository? gallery,
]) async {
  final picker = ThemePickerBloc(
    RealThemesRenderer(),
    gallery ?? emptyGallery(),
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
  bool waitForGallery = true,
}) async {
  final themes = gallery ?? emptyGallery();
  return MultiBlocProvider(
    providers: <BlocProvider<dynamic>>[
      BlocProvider<ThemePickerBloc>.value(
        value: await startThemePicker(themes),
      ),
      BlocProvider<ThemeGalleryBloc>.value(
        value: await startThemeGallery(themes, wait: waitForGallery),
      ),
    ],
    child: child,
  );
}
