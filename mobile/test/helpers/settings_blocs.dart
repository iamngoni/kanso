// Kanso — settings_blocs.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart' show addTearDown;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/library_index_status.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/state/library_index/library_index_bloc.dart';
import 'package:kanso/core/state/sync_status/sync_status_bloc.dart';
import 'package:kanso/settings/state/about/about_bloc.dart';
import 'package:kanso/settings/state/default_notebook/default_notebook_bloc.dart';

import 'close_bloc_later.dart';
import 'fixture_about_repository.dart';
import 'fixture_library_repository.dart';
import 'fixture_sync_repository.dart';
import 'seeded_appearance_bloc.dart';

/// The BLoCs the settings screens read, read: the version of the app
/// (0.1.0, build 3) and a sync status, by default no account. The
/// appearance is the golden helper's.
class SettingsBlocs {
  SettingsBlocs._(this.about, this.sync, this.index, this.defaultNotebook);

  final AboutBloc about;
  final SyncStatusBloc sync;

  /// The notebooks of the mock library.
  final LibraryIndexBloc index;
  final DefaultNotebookBloc defaultNotebook;

  /// Makes them and waits until they have read.
  static Future<SettingsBlocs> start({
    SyncStatus status = signedOutStatus,
    FixtureLibraryRepository? library,
    Preferences preferences = const Preferences(),
  }) async {
    final about = AboutBloc(FixtureAboutRepository())
      ..add(const AboutEvent.started());
    closeBlocLater(about);
    final sync = SyncStatusBloc(FixtureSyncRepository(current: status))
      ..add(const SyncStatusEvent.started());
    closeBlocLater(sync);
    final fixture = library ?? FixtureLibraryRepository();
    addTearDown(() => unawaited(fixture.close()));
    final index = LibraryIndexBloc(fixture)
      ..add(const LibraryIndexEvent.started());
    closeBlocLater(index);
    final defaultNotebook = DefaultNotebookBloc(
      SeededPreferencesRepository(preferences),
    )..add(const DefaultNotebookEvent.started());
    closeBlocLater(defaultNotebook);
    if (about.state.status != LoadStatus.ready) {
      await about.stream.firstWhere(
        (state) => state.status == LoadStatus.ready,
      );
    }
    if (sync.state.status == null) {
      await sync.stream.firstWhere((state) => state.status != null);
    }
    if (index.state.status == LibraryIndexStatus.loading) {
      await index.stream.firstWhere(
        (state) => state.status != LibraryIndexStatus.loading,
      );
    }
    // Its reading is a few turns of the event loop; a seed that is the
    // default emits nothing to wait for.
    for (var turn = 0; turn < 10; turn++) {
      await Future<void>.value();
    }
    return SettingsBlocs._(about, sync, index, defaultNotebook);
  }

  /// Providers of them, to put above a screen, including one that is a
  /// route pushed later.
  List<BlocProvider<dynamic>> get providers => <BlocProvider<dynamic>>[
    BlocProvider<AboutBloc>.value(value: about),
    BlocProvider<SyncStatusBloc>.value(value: sync),
    BlocProvider<LibraryIndexBloc>.value(value: index),
    BlocProvider<DefaultNotebookBloc>.value(value: defaultNotebook),
  ];

  /// [child] with them above it.
  Widget provide(Widget child) =>
      MultiBlocProvider(providers: providers, child: child);
}
