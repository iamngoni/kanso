// Kanso — sync_settings_bloc_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, IntAnys, ListAnys, any;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/settings/state/sync_settings/sync_settings_bloc.dart';

import '../../../helpers/fixture_sync_repository.dart';
import '../../../helpers/property.dart';

/// A change the person makes: Sync automatically on or off, or one of the
/// intervals.
SyncSettingsEvent _change(int code) => code < 2
    ? SyncSettingsEvent.enabledChanged(enabled: code == 1)
    : SyncSettingsEvent.intervalChosen(fixtureIntervals[code - 2].minutes);

void main() {
  late FixtureSyncRepository sync;

  setUp(() => sync = FixtureSyncRepository());

  SyncSettingsBloc build() => SyncSettingsBloc(sync);

  group('SyncSettingsBloc', () {
    blocTest<SyncSettingsBloc, SyncSettingsState>(
      'started reads the settings and the choices',
      build: build,
      act: (bloc) => bloc.add(const SyncSettingsEvent.started()),
      expect: () => const <SyncSettingsState>[
        SyncSettingsState(
          status: LoadStatus.ready,
          settings: SyncSettingsRow(enabled: true, intervalSecs: 300),
          options: fixtureIntervals,
        ),
      ],
      verify: (bloc) => expect(bloc.state.interval?.label, '5 minutes'),
    );

    blocTest<SyncSettingsBloc, SyncSettingsState>(
      'turning it off shows at once and is saved',
      build: build,
      act: (bloc) async {
        bloc.add(const SyncSettingsEvent.started());
        await bloc.stream.first;
        bloc.add(const SyncSettingsEvent.enabledChanged(enabled: false));
      },
      skip: 1,
      expect: () => const <SyncSettingsState>[
        SyncSettingsState(
          status: LoadStatus.ready,
          settings: SyncSettingsRow(enabled: false, intervalSecs: 300),
          options: fixtureIntervals,
        ),
      ],
      verify: (_) => expect(sync.syncSettings.enabled, isFalse),
    );

    blocTest<SyncSettingsBloc, SyncSettingsState>(
      'an interval is saved in seconds',
      build: build,
      act: (bloc) async {
        bloc.add(const SyncSettingsEvent.started());
        await bloc.stream.first;
        bloc.add(const SyncSettingsEvent.intervalChosen(60));
      },
      verify: (bloc) {
        expect(sync.syncSettings.intervalSecs, 3600);
        expect(bloc.state.interval?.label, '1 hour');
      },
    );

    blocTest<SyncSettingsBloc, SyncSettingsState>(
      'a change that cannot be saved goes back, and says why',
      build: build,
      act: (bloc) async {
        bloc.add(const SyncSettingsEvent.started());
        await bloc.stream.first;
        sync.settingsError = const ApplicationError.storage(
          message: 'Unwritable.',
        );
        bloc.add(const SyncSettingsEvent.enabledChanged(enabled: false));
      },
      skip: 1,
      expect: () => const <SyncSettingsState>[
        SyncSettingsState(
          status: LoadStatus.ready,
          settings: SyncSettingsRow(enabled: false, intervalSecs: 300),
          options: fixtureIntervals,
        ),
        SyncSettingsState(
          status: LoadStatus.ready,
          settings: SyncSettingsRow(enabled: true, intervalSecs: 300),
          options: fixtureIntervals,
          error: ApplicationError.storage(message: 'Unwritable.'),
        ),
      ],
    );

    blocTest<SyncSettingsBloc, SyncSettingsState>(
      'a change before the settings are read does nothing',
      build: build,
      act: (bloc) =>
          bloc.add(const SyncSettingsEvent.enabledChanged(enabled: false)),
      expect: () => const <SyncSettingsState>[],
    );

    test(
      'the status line is the repository’s, once the settings are read',
      () async {
        final bloc = build();
        expect(
          bloc.statusLine(signedInStatus, when: 'just now', resume: ''),
          isNull,
        );
        bloc.add(const SyncSettingsEvent.started());
        await bloc.stream.first;
        expect(
          bloc.statusLine(signedInStatus, when: 'just now', resume: '')?.label,
          'Synced just now',
        );
        await bloc.close();
      },
    );

    Glados<List<int>>(
      any.listWithLengthInRange(0, 12, any.intInRange(0, 8)),
      kansoExploreConfig(),
    ).test('whatever changes are made, what is shown is what is saved', (
      codes,
    ) async {
      final repository = FixtureSyncRepository();
      final bloc = SyncSettingsBloc(repository)
        ..add(const SyncSettingsEvent.started());
      for (final code in codes) {
        bloc.add(_change(code));
      }
      // Each change is handled in turn: wait for the last.
      await pumpEventQueue();
      await bloc.close();
      expect(bloc.state.settings, repository.syncSettings);
      expect(bloc.state.interval, isNotNull);
    });
  });
}
