// Kanso — widget_colors_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_colors.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/state/widget_colors/widget_colors_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/mock_repositories.dart';

void main() {
  late MockWidgetColorsRepository colors;

  setUpAll(() => registerFallbackValue(KansoColors.light));

  setUp(() {
    colors = MockWidgetColorsRepository();
  });

  WidgetColorsEvent kanso() => const WidgetColorsEvent.changed(
    light: KansoColors.light,
    dark: KansoColors.dark,
  );

  Future<Either<ApplicationError, bool>> answer({
    required bool written,
  }) async => Right<ApplicationError, bool>(written);

  group('WidgetColorsBloc', () {
    blocTest<WidgetColorsBloc, WidgetColorsState>(
      'publishes the colours it is told and counts what was written',
      setUp: () => when(
        () => colors.publish(
          light: any(named: 'light'),
          dark: any(named: 'dark'),
        ),
      ).thenAnswer((_) => answer(written: true)),
      build: () => WidgetColorsBloc(colors),
      act: (bloc) => bloc.add(kanso()),
      expect: () => const <WidgetColorsState>[WidgetColorsState(publishes: 1)],
      verify: (_) => verify(
        () => colors.publish(
          light: KansoColors.light,
          dark: KansoColors.dark,
        ),
      ).called(1),
    );

    blocTest<WidgetColorsBloc, WidgetColorsState>(
      'colours already written change nothing',
      setUp: () => when(
        () => colors.publish(
          light: any(named: 'light'),
          dark: any(named: 'dark'),
        ),
      ).thenAnswer((_) => answer(written: false)),
      build: () => WidgetColorsBloc(colors),
      act: (bloc) => bloc.add(kanso()),
      expect: () => const <WidgetColorsState>[WidgetColorsState()],
      verify: (bloc) => expect(bloc.state.publishes, 0),
    );

    blocTest<WidgetColorsBloc, WidgetColorsState>(
      'a failure is kept, and a later write clears it',
      setUp: () {
        final answers = <Either<ApplicationError, bool>>[
          const Left<ApplicationError, bool>(
            ApplicationError.storage(message: 'Disk full.'),
          ),
          const Right<ApplicationError, bool>(true),
        ];
        when(
          () => colors.publish(
            light: any(named: 'light'),
            dark: any(named: 'dark'),
          ),
        ).thenAnswer((_) async => answers.removeAt(0));
      },
      build: () => WidgetColorsBloc(colors),
      act: (bloc) => bloc
        ..add(kanso())
        ..add(kanso()),
      expect: () => const <WidgetColorsState>[
        WidgetColorsState(
          error: ApplicationError.storage(message: 'Disk full.'),
        ),
        WidgetColorsState(publishes: 1),
      ],
    );
  });
}
