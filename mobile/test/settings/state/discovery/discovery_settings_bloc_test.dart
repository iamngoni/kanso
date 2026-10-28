// Kanso — discovery_settings_bloc_test.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/models/repos/abs/preferences_repository.dart';
import 'package:kanso/discovery/models/repos/abs/discovery_repository.dart';
import 'package:kanso/settings/state/discovery/discovery_settings_bloc.dart';
import 'package:mocktail/mocktail.dart';

class MockDiscoveryRepository extends Mock implements DiscoveryRepository {}

class MockPreferencesRepository extends Mock implements PreferencesRepository {}

void main() {
  late MockDiscoveryRepository discovery;
  late MockPreferencesRepository preferences;

  setUp(() {
    discovery = MockDiscoveryRepository();
    preferences = MockPreferencesRepository();
    when(
      () => discovery.relatedEnabled(),
    ).thenAnswer((_) async => right(false));
    when(() => discovery.relatedState()).thenAnswer(
      (_) async => right(const RelatedState(status: RelatedStatus.off())),
    );
    when(
      () => discovery.relatedStream(),
    ).thenAnswer((_) => const Stream.empty());
    when(
      () => discovery.changes(),
    ).thenAnswer((_) => const Stream<List<String>>.empty());
    when(() => discovery.ocrWaiting()).thenAnswer((_) async => right(0));
    when(
      () => preferences.load(),
    ).thenAnswer((_) async => right(const Preferences()));
    when(() => preferences.update(any())).thenAnswer((invocation) async {
      final change =
          invocation.positionalArguments.single
              as Preferences Function(Preferences);
      return right(change(const Preferences()));
    });
  });

  blocTest<DiscoverySettingsBloc, DiscoverySettingsState>(
    'loads the live related setting and persisted image text choice',
    build: () => DiscoverySettingsBloc(discovery, preferences),
    act: (bloc) => bloc.add(const DiscoverySettingsEvent.started()),
    expect: () => <DiscoverySettingsState>[
      const DiscoverySettingsState(
        related: RelatedState(status: RelatedStatus.off()),
      ),
    ],
    verify: (_) {
      verify(() => discovery.relatedEnabled()).called(1);
      verify(() => discovery.relatedStream()).called(1);
      verify(() => preferences.load()).called(1);
    },
  );

  blocTest<DiscoverySettingsBloc, DiscoverySettingsState>(
    'persists the image text choice before showing it enabled',
    build: () => DiscoverySettingsBloc(discovery, preferences),
    act: (bloc) async {
      bloc.add(const DiscoverySettingsEvent.started());
      await Future<void>.delayed(Duration.zero);
      bloc.add(const DiscoverySettingsEvent.ocrChanged(enabled: true));
    },
    expect: () => <DiscoverySettingsState>[
      const DiscoverySettingsState(
        related: RelatedState(status: RelatedStatus.off()),
      ),
      const DiscoverySettingsState(
        related: RelatedState(status: RelatedStatus.off()),
        ocrEnabled: true,
      ),
    ],
    verify: (_) => verify(() => preferences.update(any())).called(1),
  );
}
