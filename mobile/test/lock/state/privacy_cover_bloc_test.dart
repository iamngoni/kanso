// Kanso — privacy_cover_bloc_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/lock/state/privacy_cover/privacy_cover_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../helpers/mock_repositories.dart';

void main() {
  late MockDeviceLockRepository device;

  setUp(() {
    device = MockDeviceLockRepository();
    when(() => device.setSecure(on: any(named: 'on'))).thenAnswer(
      (_) async => const Right<ApplicationError, Unit>(unit),
    );
  });

  PrivacyCoverBloc build() => PrivacyCoverBloc(device);

  group('PrivacyCoverBloc', () {
    blocTest<PrivacyCoverBloc, PrivacyCoverState>(
      'turns the cover on and off, in order',
      build: build,
      act: (bloc) => bloc
        ..add(const PrivacyCoverEvent.changed(secure: true))
        ..add(const PrivacyCoverEvent.changed(secure: false)),
      expect: () => const <PrivacyCoverState>[
        PrivacyCoverState(secure: true),
        PrivacyCoverState(),
      ],
      verify: (_) {
        verifyInOrder(<void Function()>[
          () => device.setSecure(on: true),
          () => device.setSecure(on: false),
        ]);
      },
    );

    blocTest<PrivacyCoverBloc, PrivacyCoverState>(
      'asking for what is already so asks the phone nothing',
      build: build,
      act: (bloc) => bloc
        ..add(const PrivacyCoverEvent.changed(secure: false))
        ..add(const PrivacyCoverEvent.changed(secure: true))
        ..add(const PrivacyCoverEvent.changed(secure: true)),
      verify: (_) {
        verify(() => device.setSecure(on: true)).called(1);
        verifyNever(() => device.setSecure(on: false));
      },
    );

    blocTest<PrivacyCoverBloc, PrivacyCoverState>(
      'a phone that refuses leaves the state as it was',
      setUp: () {
        when(() => device.setSecure(on: true)).thenAnswer(
          (_) async => const Left<ApplicationError, Unit>(
            ApplicationError.unexpected(message: 'No window.'),
          ),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const PrivacyCoverEvent.changed(secure: true)),
      expect: () => const <PrivacyCoverState>[],
    );
  });
}
