// Kanso — note_creation_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:typed_data';

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/capture/models/data/media_source.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/models/repos/abs/preferences_repository.dart';
import 'package:kanso/note/models/data/new_note_start.dart';
import 'package:kanso/note/models/repos/impl/memory_note_place_repository.dart';
import 'package:kanso/shell/models/data/note_creation_failure.dart';
import 'package:kanso/shell/models/data/note_opening.dart';
import 'package:kanso/shell/state/note_creation/note_creation_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/fake_media_picker_repository.dart';
import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/mock_repositories.dart';

class MockPreferences extends Mock implements PreferencesRepository {}

ShareFile _file(String name) => ShareFile(
  name: name,
  bytes: Uint8List.fromList(<int>[1]),
);

void main() {
  late FixtureLibraryRepository library;
  late MemoryNotePlaceRepository places;
  late MockPreferences preferences;
  late MockCaptureRepository capture;
  late FakeMediaPickerRepository picker;
  late Preferences saved;

  setUp(() {
    library = FixtureLibraryRepository();
    addTearDown(library.close);
    places = MemoryNotePlaceRepository();
    preferences = MockPreferences();
    capture = MockCaptureRepository();
    picker = FakeMediaPickerRepository();
    saved = const Preferences();
    registerFallbackValue(const Share(files: <ShareFile>[]));
    when(
      preferences.load,
    ).thenAnswer((_) async => Right<ApplicationError, Preferences>(saved));
  });

  NoteCreationBloc build() => NoteCreationBloc(
    library,
    places,
    preferences,
    capture,
    picker,
  );

  group('NoteCreationBloc requested', () {
    blocTest<NoteCreationBloc, NoteCreationState>(
      'makes an untitled note from the place, and opens it there ready to '
      'write in',
      build: build,
      act: (bloc) => bloc.add(
        const NoteCreationEvent.requested(
          ListPlace.notebook(id: MockIds.kanso),
        ),
      ),
      expect: () => <NoteCreationState>[
        const NoteCreationState(creating: true),
        const NoteCreationState(
          opening: NoteOpening(
            noteId: 'note:new-1',
            place: ListPlace.notebook(id: MockIds.kanso),
          ),
        ),
      ],
      verify: (_) {
        final made = library.created.single;
        expect(made.title, '');
        expect(made.place, const ListPlace.notebook(id: MockIds.kanso));
        expect(made.pinned, isFalse);
        expect(made.notebookId, isNull);
        expect(places.takeOpenedNew('note:new-1'), NewNoteStart.title);
      },
    );

    blocTest<NoteCreationBloc, NoteCreationState>(
      'asks for the default notebook the person chose, and for a pin when the '
      'list shows pinned notes only',
      setUp: () => saved = const Preferences(defaultNotebookId: 'book:chosen'),
      build: build,
      act: (bloc) => bloc.add(
        const NoteCreationEvent.requested(ListPlace.all(), pinned: true),
      ),
      verify: (_) {
        final made = library.created.single;
        expect(made.preferredNotebookId, 'book:chosen');
        expect(made.pinned, isTrue);
      },
    );

    blocTest<NoteCreationBloc, NoteCreationState>(
      'asks for no preference when the preferences cannot be read',
      setUp: () {
        when(preferences.load).thenAnswer(
          (_) async => const Left<ApplicationError, Preferences>(
            ApplicationError.storage(message: 'Unreadable.'),
          ),
        );
      },
      build: build,
      act: (bloc) =>
          bloc.add(const NoteCreationEvent.requested(ListPlace.all())),
      verify: (_) => expect(library.created.single.preferredNotebookId, isNull),
    );

    blocTest<NoteCreationBloc, NoteCreationState>(
      'from the trash opens the note in the notebook it went into',
      setUp: () => library.notes['note:new-1'] = mockNote(
        id: 'note:new-1',
        notebookId: MockIds.inbox,
      ),
      build: build,
      act: (bloc) =>
          bloc.add(const NoteCreationEvent.requested(ListPlace.trash())),
      expect: () => <NoteCreationState>[
        const NoteCreationState(creating: true),
        const NoteCreationState(
          opening: NoteOpening(
            noteId: 'note:new-1',
            place: ListPlace.notebook(id: MockIds.inbox),
          ),
        ),
      ],
    );

    blocTest<NoteCreationBloc, NoteCreationState>(
      'says a locked notebook has to be unlocked, and makes no note to open',
      setUp: () => library.createResult = const Left<ApplicationError, Saved>(
        ApplicationError.locked(message: 'locked'),
      ),
      build: build,
      act: (bloc) => bloc.add(
        const NoteCreationEvent.requested(
          ListPlace.notebook(id: MockIds.kanso),
        ),
      ),
      expect: () => const <NoteCreationState>[
        NoteCreationState(creating: true),
        NoteCreationState(failure: NoteCreationFailure.locked),
      ],
      verify: (_) => expect(places.takeOpenedNew('note:new-1'), isNull),
    );

    blocTest<NoteCreationBloc, NoteCreationState>(
      'says any other failure, and a dismissed notice is gone',
      setUp: () => library.createResult = const Left<ApplicationError, Saved>(
        ApplicationError.storage(message: 'disk'),
      ),
      build: build,
      act: (bloc) async {
        bloc.add(const NoteCreationEvent.requested(ListPlace.all()));
        await bloc.stream.firstWhere((s) => s.failure != null);
        bloc.add(const NoteCreationEvent.noticeDismissed());
      },
      expect: () => const <NoteCreationState>[
        NoteCreationState(creating: true),
        NoteCreationState(failure: NoteCreationFailure.failed),
        NoteCreationState(),
      ],
    );

    blocTest<NoteCreationBloc, NoteCreationState>(
      'a second request while one is being made is ignored',
      build: build,
      act: (bloc) => bloc
        ..add(const NoteCreationEvent.requested(ListPlace.all()))
        ..add(const NoteCreationEvent.requested(ListPlace.all())),
      verify: (_) => expect(library.created, hasLength(1)),
    );

    blocTest<NoteCreationBloc, NoteCreationState>(
      'an opening that was handled is cleared',
      build: build,
      act: (bloc) async {
        bloc.add(const NoteCreationEvent.requested(ListPlace.all()));
        await bloc.stream.firstWhere((s) => s.opening != null);
        bloc.add(const NoteCreationEvent.openingHandled());
      },
      skip: 2,
      expect: () => const <NoteCreationState>[NoteCreationState()],
    );

    blocTest<NoteCreationBloc, NoteCreationState>(
      'opens at the text when the request asks for it',
      build: build,
      act: (bloc) => bloc.add(
        const NoteCreationEvent.requested(
          ListPlace.all(),
          start: NewNoteStart.text,
        ),
      ),
      expect: () => const <NoteCreationState>[
        NoteCreationState(creating: true),
        NoteCreationState(
          opening: NoteOpening(noteId: 'note:new-1', place: ListPlace.all()),
        ),
      ],
      verify: (_) => expect(
        places.takeOpenedNew('note:new-1'),
        NewNoteStart.text,
      ),
    );
  });

  group('NoteCreationBloc captured', () {
    blocTest<NoteCreationBloc, NoteCreationState>(
      'a picker failure says the note was not made',
      setUp: () =>
          picker.answer = const Left<ApplicationError, List<ShareFile>>(
            ApplicationError.storage(message: 'No picker.'),
          ),
      build: build,
      act: (bloc) async {
        bloc.add(const NoteCreationEvent.captured(MediaSource.camera));
        await bloc.stream.firstWhere((state) => state.failure != null);
      },
      expect: () => const <NoteCreationState>[
        NoteCreationState(failure: NoteCreationFailure.failed),
      ],
      verify: (_) {
        expect(picker.sources, <MediaSource>[MediaSource.camera]);
        verifyNever(() => capture.capture(any()));
      },
    );

    blocTest<NoteCreationBloc, NoteCreationState>(
      'picking nothing makes no note',
      build: build,
      act: (bloc) async {
        bloc.add(const NoteCreationEvent.captured(MediaSource.photos));
        await pumpEventQueue();
      },
      expect: () => const <NoteCreationState>[],
      verify: (_) {
        expect(picker.sources, <MediaSource>[MediaSource.photos]);
        verifyNever(() => capture.capture(any()));
      },
    );

    blocTest<NoteCreationBloc, NoteCreationState>(
      'captures the files into one note and opens it with what was left out',
      setUp: () {
        picker.answer = Right<ApplicationError, List<ShareFile>>(
          <ShareFile>[_file('photo.png'), _file('scan.pdf')],
        );
        when(() => capture.capture(any())).thenAnswer(
          (_) async => const Right<ApplicationError, CaptureOutcome>(
            CaptureOutcome(
              noteId: 'note:capture-1',
              problems: <String>['scan.pdf: Not an image or a PDF.'],
            ),
          ),
        );
      },
      build: build,
      act: (bloc) => bloc.add(
        const NoteCreationEvent.captured(MediaSource.photos),
      ),
      expect: () => const <NoteCreationState>[
        NoteCreationState(creating: true),
        NoteCreationState(
          opening: NoteOpening(
            noteId: 'note:capture-1',
            place: ListPlace.all(),
          ),
          problems: <String>['scan.pdf: Not an image or a PDF.'],
        ),
      ],
      verify: (_) {
        expect(picker.sources, <MediaSource>[MediaSource.photos]);
        final share =
            verify(() => capture.capture(captureAny())).captured.single
                as Share;
        expect(
          share.files.map((file) => file.name).toList(),
          <String>['photo.png', 'scan.pdf'],
        );
      },
    );

    blocTest<NoteCreationBloc, NoteCreationState>(
      'a locked Inbox says it has to be unlocked',
      setUp: () {
        picker.answer = Right<ApplicationError, List<ShareFile>>(
          <ShareFile>[_file('photo.png')],
        );
        when(() => capture.capture(any())).thenAnswer(
          (_) async => const Left<ApplicationError, CaptureOutcome>(
            ApplicationError.locked(message: 'Locked.'),
          ),
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const NoteCreationEvent.captured(MediaSource.pdf));
        await bloc.stream.firstWhere((state) => state.failure != null);
      },
      expect: () => const <NoteCreationState>[
        NoteCreationState(creating: true),
        NoteCreationState(failure: NoteCreationFailure.locked),
      ],
    );

    blocTest<NoteCreationBloc, NoteCreationState>(
      'any other capture failure says it failed',
      setUp: () {
        picker.answer = Right<ApplicationError, List<ShareFile>>(
          <ShareFile>[_file('scan.pdf')],
        );
        when(() => capture.capture(any())).thenAnswer(
          (_) async => const Left<ApplicationError, CaptureOutcome>(
            ApplicationError.storage(message: 'Disk full.'),
          ),
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const NoteCreationEvent.captured(MediaSource.pdf));
        await bloc.stream.firstWhere((state) => state.failure != null);
      },
      expect: () => const <NoteCreationState>[
        NoteCreationState(creating: true),
        NoteCreationState(failure: NoteCreationFailure.failed),
      ],
    );

    blocTest<NoteCreationBloc, NoteCreationState>(
      'dismissing the notice clears the problems and keeps the opening',
      setUp: () {
        picker.answer = Right<ApplicationError, List<ShareFile>>(
          <ShareFile>[_file('scan.pdf')],
        );
        when(() => capture.capture(any())).thenAnswer(
          (_) async => const Right<ApplicationError, CaptureOutcome>(
            CaptureOutcome(
              noteId: 'note:capture-1',
              problems: <String>['scan.pdf: Not an image or a PDF.'],
            ),
          ),
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const NoteCreationEvent.captured(MediaSource.pdf));
        await bloc.stream.firstWhere((state) => state.problems.isNotEmpty);
        bloc.add(const NoteCreationEvent.noticeDismissed());
      },
      expect: () => const <NoteCreationState>[
        NoteCreationState(creating: true),
        NoteCreationState(
          opening: NoteOpening(
            noteId: 'note:capture-1',
            place: ListPlace.all(),
          ),
          problems: <String>['scan.pdf: Not an image or a PDF.'],
        ),
        NoteCreationState(
          opening: NoteOpening(
            noteId: 'note:capture-1',
            place: ListPlace.all(),
          ),
        ),
      ],
    );
  });

  group('NoteCreationBloc titled', () {
    blocTest<NoteCreationBloc, NoteCreationState>(
      'makes the note a link names, in the preferred notebook, and opens it '
      'in All Notes',
      setUp: () => saved = const Preferences(defaultNotebookId: 'book:chosen'),
      build: build,
      act: (bloc) => bloc.add(const NoteCreationEvent.titled('Not yet')),
      expect: () => const <NoteCreationState>[
        NoteCreationState(creating: true),
        NoteCreationState(
          opening: NoteOpening(noteId: 'note:new-1', place: ListPlace.all()),
        ),
      ],
      verify: (_) {
        final made = library.created.single;
        expect(made.title, 'Not yet');
        expect(made.place, isNull);
        expect(made.preferredNotebookId, 'book:chosen');
        expect(places.takeOpenedNew('note:new-1'), NewNoteStart.title);
      },
    );

    blocTest<NoteCreationBloc, NoteCreationState>(
      'a note that could not be made says so and goes nowhere',
      setUp: () => library.createResult = const Left<ApplicationError, Saved>(
        ApplicationError.storage(message: 'disk'),
      ),
      build: build,
      act: (bloc) => bloc.add(const NoteCreationEvent.titled('Not yet')),
      expect: () => const <NoteCreationState>[
        NoteCreationState(creating: true),
        NoteCreationState(failure: NoteCreationFailure.failed),
      ],
    );
  });
}
