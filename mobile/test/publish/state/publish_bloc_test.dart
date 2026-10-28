// Kanso — publish_bloc_test.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/publish.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/publish/models/repos/abs/publish_repository.dart';
import 'package:kanso/publish/state/publish/publish_bloc.dart';
import 'package:mocktail/mocktail.dart';

class MockPublishRepository extends Mock implements PublishRepository {}

const PaletteDto _palette = PaletteDto(
  dark: false,
  background: 0xfcfaf4,
  text: 0x26241f,
  muted: 0x857c6e,
  accent: 0x4d6684,
  border: 0xddd6c9,
  surface: 0xf4f1ea,
  quote: 0x655f55,
  taskDone: 0x655f55,
  inlineCodeBackground: 0xf4f1ea,
);

const PreviewOptions _options = PreviewOptions(
  size: 16,
  utcOffsetSecs: 0,
  code: <SyntaxStyleRow>[],
);

const List<PublishExpiryOption> _expiryOptions = <PublishExpiryOption>[
  PublishExpiryOption(expiry: PublishExpiry.never, label: 'Never'),
  PublishExpiryOption(expiry: PublishExpiry.day, label: '1 day'),
  PublishExpiryOption(expiry: PublishExpiry.week, label: '7 days'),
  PublishExpiryOption(expiry: PublishExpiry.month, label: '30 days'),
];

const PublishLink _link = PublishLink(
  url: 'https://kanso.example/p/abc#key',
  expiry: PublishExpiry.never,
  publishedAtMs: 1,
);

void main() {
  late MockPublishRepository repository;

  setUp(() {
    repository = MockPublishRepository();
    when(() => repository.expiryOptions()).thenAnswer(
      (_) async => right(_expiryOptions),
    );
    when(() => repository.note('note:one')).thenAnswer(
      (_) async => right(
        const PublishNoteInfo(
          id: 'note:one',
          title: 'Field notes',
          body: 'A real note body.',
          rev: '2-a',
          needsUpdate: false,
        ),
      ),
    );
    when(() => repository.notebook('book:one')).thenAnswer(
      (_) async => right(
        const PublishNotebookInfo(
          id: 'book:one',
          name: 'Research',
          contents: '2 notes and 1 image.',
          locked: 'Left out because it is locked: Private.',
          needsUpdate: false,
          noteCount: 2,
          fileCount: 1,
        ),
      ),
    );
    when(
      () => repository.publishNote(
        id: 'note:one',
        expiry: PublishExpiry.never,
        palette: _palette,
        options: _options,
      ),
    ).thenAnswer((_) async => right(_link));
    when(
      () => repository.publishNotebook(
        id: 'book:one',
        expiry: PublishExpiry.never,
        palette: _palette,
        options: _options,
      ),
    ).thenAnswer((_) async => right(_link));
    when(() => repository.unpublishNote('note:one')).thenAnswer(
      (_) async => right(unit),
    );
    when(() => repository.unpublishNotebook('book:one')).thenAnswer(
      (_) async => right(unit),
    );
  });

  blocTest<PublishBloc, PublishState>(
    'note moves idle to publishing to published, and unpublishes to idle',
    build: () => PublishBloc(repository),
    act: (bloc) async {
      bloc.add(
        const PublishEvent.openNote(
          id: 'note:one',
          palette: _palette,
          options: _options,
        ),
      );
      await Future<void>.delayed(const Duration(milliseconds: 10));
      bloc.add(const PublishEvent.publish(PublishExpiry.never));
      await Future<void>.delayed(const Duration(milliseconds: 10));
      bloc.add(const PublishEvent.unpublish());
    },
    expect: () => <Matcher>[
      isA<PublishIdleState>(),
      isA<PublishPublishingState>(),
      isA<PublishPublishedState>(),
      isA<PublishPublishingState>(),
      isA<PublishIdleState>(),
    ],
  );

  blocTest<PublishBloc, PublishState>(
    'notebook uses the same publishing and published transitions',
    build: () => PublishBloc(repository),
    act: (bloc) async {
      bloc.add(
        const PublishEvent.openNotebook(
          id: 'book:one',
          palette: _palette,
          options: _options,
        ),
      );
      await Future<void>.delayed(const Duration(milliseconds: 10));
      bloc.add(const PublishEvent.publish(PublishExpiry.never));
    },
    expect: () => <Matcher>[
      isA<PublishIdleState>(),
      isA<PublishPublishingState>(),
      isA<PublishPublishedState>(),
    ],
  );

  blocTest<PublishBloc, PublishState>(
    'a signed-out publishing attempt ends failed with its actionable message',
    setUp: () {
      when(
        () => repository.publishNote(
          id: 'note:one',
          expiry: PublishExpiry.never,
          palette: _palette,
          options: _options,
        ),
      ).thenAnswer(
        (_) async => left(
          const ApplicationError.signedOut(message: 'Sign in to publish.'),
        ),
      );
    },
    build: () => PublishBloc(repository),
    act: (bloc) async {
      bloc.add(
        const PublishEvent.openNote(
          id: 'note:one',
          palette: _palette,
          options: _options,
        ),
      );
      await Future<void>.delayed(const Duration(milliseconds: 10));
      bloc.add(const PublishEvent.publish(PublishExpiry.never));
    },
    expect: () => <Matcher>[
      isA<PublishIdleState>(),
      isA<PublishPublishingState>(),
      isA<PublishFailedState>().having(
        (state) => state.text,
        'error text',
        'Sign in to publish.',
      ),
    ],
  );
}
