// Kanso — push_setting_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/push/models/data/push_setting_status.dart';
import 'package:kanso/push/state/push_setting/push_setting_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/mock_repositories.dart';

const _network = ApplicationError.network(message: 'No connection.');

const _loadedOn = PushSettingState(status: PushSettingStatus.ready);

void main() {
  late MockPushRepository push;

  setUp(() {
    push = MockPushRepository();
    when(
      () => push.unavailableReason(),
    ).thenAnswer((_) async => const Right<ApplicationError, String?>(null));
    when(
      () => push.isEnabled(),
    ).thenAnswer((_) async => const Right<ApplicationError, bool>(true));
    when(
      () => push.setEnabled(enabled: any(named: 'enabled')),
    ).thenAnswer((_) async => const Right<ApplicationError, Unit>(unit));
  });

  PushSettingBloc build() => PushSettingBloc(push);

  group('PushSettingBloc', () {
    test('starts loading, on', () {
      final state = build().state;
      expect(state.status, PushSettingStatus.loading);
      expect(state.enabled, isTrue);
    });

    blocTest<PushSettingBloc, PushSettingState>(
      'started reads the setting',
      setUp: () {
        when(
          () => push.isEnabled(),
        ).thenAnswer((_) async => const Right<ApplicationError, bool>(false));
      },
      build: build,
      act: (bloc) => bloc.add(const PushSettingEvent.started()),
      expect: () => <PushSettingState>[
        const PushSettingState(
          status: PushSettingStatus.ready,
          enabled: false,
        ),
      ],
    );

    blocTest<PushSettingBloc, PushSettingState>(
      'started where push cannot work is unavailable and does not read',
      setUp: () {
        when(() => push.unavailableReason()).thenAnswer(
          (_) async => const Right<ApplicationError, String?>('No Firebase.'),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const PushSettingEvent.started()),
      expect: () => <PushSettingState>[
        const PushSettingState(status: PushSettingStatus.unavailable),
      ],
      verify: (_) => verifyNever(() => push.isEnabled()),
    );

    blocTest<PushSettingBloc, PushSettingState>(
      'a setting that cannot be read is on, and says why',
      setUp: () {
        when(() => push.isEnabled()).thenAnswer(
          (_) async => const Left<ApplicationError, bool>(_network),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const PushSettingEvent.started()),
      expect: () => <PushSettingState>[
        const PushSettingState(
          status: PushSettingStatus.ready,
          error: _network,
        ),
      ],
    );

    blocTest<PushSettingBloc, PushSettingState>(
      'turning it off shows at once and is saved',
      build: build,
      seed: () => _loadedOn,
      act: (bloc) => bloc.add(const PushSettingEvent.toggled(enabled: false)),
      expect: () => <PushSettingState>[
        const PushSettingState(
          status: PushSettingStatus.ready,
          enabled: false,
          saving: true,
        ),
        const PushSettingState(
          status: PushSettingStatus.ready,
          enabled: false,
        ),
      ],
      verify: (_) => verify(() => push.setEnabled(enabled: false)).called(1),
    );

    blocTest<PushSettingBloc, PushSettingState>(
      'a change that fails is put back, with the error',
      setUp: () {
        when(() => push.setEnabled(enabled: false)).thenAnswer(
          (_) async => const Left<ApplicationError, Unit>(_network),
        );
      },
      build: build,
      seed: () => _loadedOn,
      act: (bloc) => bloc.add(const PushSettingEvent.toggled(enabled: false)),
      expect: () => <PushSettingState>[
        const PushSettingState(
          status: PushSettingStatus.ready,
          enabled: false,
          saving: true,
        ),
        const PushSettingState(
          status: PushSettingStatus.ready,
          error: _network,
        ),
      ],
    );

    blocTest<PushSettingBloc, PushSettingState>(
      'a change before the setting is read, or where push cannot work, '
      'does nothing',
      build: build,
      act: (bloc) => bloc.add(const PushSettingEvent.toggled(enabled: false)),
      expect: () => <PushSettingState>[],
      verify: (_) =>
          verifyNever(() => push.setEnabled(enabled: any(named: 'enabled'))),
    );

    blocTest<PushSettingBloc, PushSettingState>(
      'reading the notice of a failure clears it',
      build: build,
      seed: () => const PushSettingState(
        status: PushSettingStatus.ready,
        error: _network,
      ),
      act: (bloc) => bloc.add(const PushSettingEvent.errorDismissed()),
      expect: () => <PushSettingState>[_loadedOn],
    );
  });
}
