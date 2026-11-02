// Kanso — about_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/app_version.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/settings/state/about/about_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/mock_repositories.dart';

const _version = AppVersion(version: '0.1.0', build: '7');

void main() {
  late MockAboutRepository about;

  setUp(() {
    about = MockAboutRepository();
    when(() => about.version()).thenAnswer(
      (_) async => const Right<ApplicationError, AppVersion>(_version),
    );
    when(
      () => about.openWebsite(),
    ).thenAnswer((_) async => const Right<ApplicationError, Unit>(unit));
    when(
      () => about.openReleaseNotes(),
    ).thenAnswer((_) async => const Right<ApplicationError, Unit>(unit));
    when(
      () => about.openHelp(),
    ).thenAnswer((_) async => const Right<ApplicationError, Unit>(unit));
    when(
      () => about.openRatePage(),
    ).thenAnswer((_) async => const Right<ApplicationError, Unit>(unit));
  });

  AboutBloc build() => AboutBloc(about);

  group('AboutBloc', () {
    test('starts loading, with no version', () {
      final state = build().state;
      expect(state.status, LoadStatus.loading);
      expect(state.version, isNull);
    });

    blocTest<AboutBloc, AboutState>(
      'started reads the version',
      build: build,
      act: (bloc) => bloc.add(const AboutEvent.started()),
      expect: () => <AboutState>[
        const AboutState(),
        const AboutState(status: LoadStatus.ready, version: _version),
      ],
    );

    blocTest<AboutBloc, AboutState>(
      'started fails when the version cannot be read, and again reads it',
      setUp: () {
        when(() => about.version()).thenAnswer(
          (_) async => const Left<ApplicationError, AppVersion>(
            ApplicationError.unexpected(message: 'No package information.'),
          ),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const AboutEvent.started()),
      expect: () => <AboutState>[
        const AboutState(),
        const AboutState(status: LoadStatus.failed),
      ],
    );

    blocTest<AboutBloc, AboutState>(
      'rateOpened opens the store page and changes nothing shown',
      build: build,
      act: (bloc) => bloc.add(const AboutEvent.rateOpened()),
      expect: () => <AboutState>[],
      verify: (_) {
        verify(() => about.openRatePage()).called(1);
        verifyNever(() => about.openWebsite());
      },
    );

    blocTest<AboutBloc, AboutState>(
      'websiteOpened opens the website and changes nothing shown',
      build: build,
      act: (bloc) => bloc.add(const AboutEvent.websiteOpened()),
      expect: () => <AboutState>[],
      verify: (_) => verify(() => about.openWebsite()).called(1),
    );

    blocTest<AboutBloc, AboutState>(
      'websiteOpened that fails changes nothing shown either',
      setUp: () {
        when(() => about.openWebsite()).thenAnswer(
          (_) async => const Left<ApplicationError, Unit>(
            ApplicationError.unexpected(message: 'No browser.'),
          ),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const AboutEvent.websiteOpened()),
      expect: () => <AboutState>[],
    );

    blocTest<AboutBloc, AboutState>(
      'releaseNotesOpened opens the release notes and nothing else',
      build: build,
      act: (bloc) => bloc.add(const AboutEvent.releaseNotesOpened()),
      expect: () => <AboutState>[],
      verify: (_) {
        verify(() => about.openReleaseNotes()).called(1);
        verifyNever(() => about.openWebsite());
        verifyNever(() => about.openHelp());
      },
    );

    blocTest<AboutBloc, AboutState>(
      'helpOpened opens the help page and nothing else',
      build: build,
      act: (bloc) => bloc.add(const AboutEvent.helpOpened()),
      expect: () => <AboutState>[],
      verify: (_) {
        verify(() => about.openHelp()).called(1);
        verifyNever(() => about.openWebsite());
        verifyNever(() => about.openReleaseNotes());
      },
    );

    blocTest<AboutBloc, AboutState>(
      'releaseNotesOpened and helpOpened that fail change nothing shown',
      setUp: () {
        when(() => about.openReleaseNotes()).thenAnswer(
          (_) async => const Left<ApplicationError, Unit>(
            ApplicationError.unexpected(message: 'No browser.'),
          ),
        );
        when(() => about.openHelp()).thenAnswer(
          (_) async => const Left<ApplicationError, Unit>(
            ApplicationError.unexpected(message: 'No browser.'),
          ),
        );
      },
      build: build,
      act: (bloc) => bloc
        ..add(const AboutEvent.releaseNotesOpened())
        ..add(const AboutEvent.helpOpened()),
      expect: () => <AboutState>[],
    );
  });
}
