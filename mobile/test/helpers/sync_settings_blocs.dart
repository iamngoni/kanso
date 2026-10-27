// Kanso — sync_settings_blocs.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:kanso/settings/state/sync_settings/sync_settings_bloc.dart';

import 'close_bloc_later.dart';
import 'fixture_sync_repository.dart';

/// A `SyncSettingsBloc` over [sync] (by default Sync automatically on, every
/// five minutes), started; it has read once the test pumps. It is closed
/// when the test ends.
SyncSettingsBloc startedSyncSettingsBloc([FixtureSyncRepository? sync]) {
  final bloc = SyncSettingsBloc(sync ?? FixtureSyncRepository())
    ..add(const SyncSettingsEvent.started());
  closeBlocLater(bloc);
  return bloc;
}
