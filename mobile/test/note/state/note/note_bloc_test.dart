// Kanso — note_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/utils/source_lines.dart';
import 'package:kanso/note/models/data/new_note_start.dart';
import 'package:kanso/note/models/data/note_mode.dart';
import 'package:kanso/note/models/data/note_place.dart';
import 'package:kanso/note/models/data/note_screen_status.dart';
import 'package:kanso/note/models/repos/impl/memory_note_place_repository.dart';
import 'package:kanso/note/state/note/note_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/fixture_renderer_repository.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/mock_repositories.dart';
import '../../../helpers/seeded_appearance_bloc.dart';

const _failure = ApplicationError.storage(message: 'Disk error.');

void main() {
  late MockLibraryRepository library;
  late MockRendererRepository renderer;
  late StreamController<List<String>> changes;
  late MemoryNotePlaceRepository places;
  var note = mockOpenNote;

  setUp(() {
    note = mockOpenNote;
    places = MemoryNotePlaceRepository();
    changes = StreamController<List<String>>.broadcast();
    addTearDown(changes.close);
    library = MockLibraryRepository();
    renderer = MockRendererRepository();
    when(() => library.changes()).thenAnswer((_) => changes.stream);
    when(() => library.note(any())).thenAnswer(
      (_) async => Right<ApplicationError, NoteView>(note),
    );
    when(
      () => renderer.styledRuns(
        any(),
        start: any(named: 'start'),
        end: any(named: 'end'),
        themeId: any(named: 'themeId'),
      ),
    ).thenAnswer(
      (invocation) async => Right<ApplicationError, List<StyledRun>>(
        fixtureRuns(invocation.positionalArguments.first as String),
      ),
    );
  });

  NoteBloc build() => NoteBloc(library, renderer, places);

  void expectNoHighlighting() {
    verifyNever(
      () => renderer.styledRuns(
        any(),
        start: any(named: 'start'),
        end: any(named: 'end'),
        themeId: any(named: 'themeId'),
      ),
    );
  }

  group('NoteBloc', () {
    test('starts loading, with nothing', () {
      final state = build().state;
      expect(state.status, NoteScreenStatus.loading);
      expect(state.note, isNull);
      expect(state.lines, isEmpty);
    });

    blocTest<NoteBloc, NoteState>(
      'started reads the note and highlights its text in the theme',
      build: build,
      act: (bloc) => bloc.add(
        const NoteEvent.started(MockIds.syncDesign, 'kanso-dark'),
      ),
      expect: () => <Matcher>[
        isA<NoteState>()
            .having((s) => s.noteId, 'noteId', MockIds.syncDesign)
            .having((s) => s.themeId, 'themeId', 'kanso-dark')
            .having((s) => s.status, 'status', NoteScreenStatus.loading),
        isA<NoteState>()
            .having((s) => s.status, 'status', NoteScreenStatus.ready)
            .having((s) => s.note, 'note', mockOpenNote),
      ],
      verify: (bloc) {
        verify(() => library.note(MockIds.syncDesign)).called(1);
        final call = verify(
          () => renderer.styledRuns(
            captureAny(),
            start: captureAny(named: 'start'),
            end: captureAny(named: 'end'),
            themeId: captureAny(named: 'themeId'),
          ),
        ).captured;
        expect(call, <Object?>[
          mockNoteBody,
          0,
          mockNoteBody.length,
          'kanso-dark',
        ]);
        expect(
          bloc.state.lines,
          sourceLines(mockNoteBody, fixtureRuns(mockNoteBody)),
        );
        // The first line is the heading, styled.
        expect(bloc.state.lines.first.runs, isNotEmpty);
      },
    );

    blocTest<NoteBloc, NoteState>(
      'a note that is locked and closed is its title and its lock',
      setUp: () {
        note = mockLockedNote;
      },
      build: build,
      act: (bloc) => bloc.add(
        const NoteEvent.started(MockIds.privateJournal, 'kanso-light'),
      ),
      verify: (bloc) {
        expect(bloc.state.status, NoteScreenStatus.locked);
        expect(bloc.state.note?.title, 'Private journal');
        expect(bloc.state.note?.body, isNull);
        expect(bloc.state.lines, isEmpty);
        expectNoHighlighting();
      },
    );

    blocTest<NoteBloc, NoteState>(
      "never shows a closed note's text, even if it were handed over",
      setUp: () {
        // The library never does this; the screen must not depend on it.
        note = const NoteView(
          id: MockIds.privateJournal,
          rev: '1',
          title: 'Private journal',
          body: mockSecret,
          lock: LockState.closed(),
          notebookId: MockIds.journal,
          tags: <String>[],
          status: 'none',
          pinned: false,
          createdMs: 1,
          updatedMs: 1,
        );
      },
      build: build,
      act: (bloc) => bloc.add(
        const NoteEvent.started(MockIds.privateJournal, 'kanso-light'),
      ),
      verify: (bloc) {
        expect(bloc.state.status, NoteScreenStatus.locked);
        expect(bloc.state.lines, isEmpty);
        expectNoHighlighting();
      },
    );

    blocTest<NoteBloc, NoteState>(
      'a note that is not there is not found',
      setUp: () {
        when(() => library.note(any())).thenAnswer(
          (_) async => const Left<ApplicationError, NoteView>(
            ApplicationError.notFound(message: 'Not found.'),
          ),
        );
      },
      build: build,
      act: (bloc) =>
          bloc.add(const NoteEvent.started('note:gone', 'kanso-light')),
      verify: (bloc) {
        expect(bloc.state.status, NoteScreenStatus.notFound);
        expect(bloc.state.note, isNull);
        expectNoHighlighting();
      },
    );

    blocTest<NoteBloc, NoteState>(
      'a failed reading says why, and retry reads again',
      setUp: () {
        var calls = 0;
        when(() => library.note(any())).thenAnswer((_) async {
          calls++;
          return calls == 1
              ? const Left<ApplicationError, NoteView>(_failure)
              : Right<ApplicationError, NoteView>(note);
        });
      },
      build: build,
      act: (bloc) async {
        bloc.add(const NoteEvent.started(MockIds.syncDesign, 'kanso-light'));
        await bloc.stream.firstWhere(
          (state) => state.status == NoteScreenStatus.failed,
        );
        expect(bloc.state.error, _failure);
        bloc.add(const NoteEvent.retried());
      },
      verify: (bloc) {
        expect(bloc.state.status, NoteScreenStatus.ready);
        expect(bloc.state.error, isNull);
      },
    );

    blocTest<NoteBloc, NoteState>(
      'shows the text plain when it cannot be highlighted',
      setUp: () {
        when(
          () => renderer.styledRuns(
            any(),
            start: any(named: 'start'),
            end: any(named: 'end'),
            themeId: any(named: 'themeId'),
          ),
        ).thenAnswer(
          (_) async => const Left<ApplicationError, List<StyledRun>>(
            ApplicationError.notFound(message: 'No such theme.'),
          ),
        );
      },
      build: build,
      act: (bloc) => bloc.add(
        const NoteEvent.started(MockIds.syncDesign, 'no-such-theme'),
      ),
      verify: (bloc) {
        expect(bloc.state.status, NoteScreenStatus.ready);
        expect(bloc.state.lines, isNotEmpty);
        expect(bloc.state.lines.every((line) => line.runs.isEmpty), isTrue);
      },
    );

    group('the library changing', () {
      blocTest<NoteBloc, NoteState>(
        'reads the note again when it is written',
        build: build,
        act: (bloc) async {
          bloc.add(const NoteEvent.started(MockIds.syncDesign, 'kanso-light'));
          await bloc.stream.firstWhere(
            (state) => state.status == NoteScreenStatus.ready,
          );
          note = NoteView(
            id: mockOpenNote.id,
            rev: '2-note:sync',
            title: 'Sync design notes, revised',
            body: '# Changed\n',
            lock: const LockState.plain(),
            notebookId: mockOpenNote.notebookId,
            tags: const <String>[],
            status: 'none',
            pinned: false,
            createdMs: mockOpenNote.createdMs,
            updatedMs: mockOpenNote.updatedMs,
          );
          changes
            ..add(<String>[MockIds.syncDesign, 'note:other'])
            ..add(<String>[MockIds.syncDesign]);
        },
        wait: const Duration(milliseconds: 300),
        verify: (bloc) {
          verify(() => library.note(any())).called(2);
          expect(bloc.state.note?.title, 'Sync design notes, revised');
          expect(bloc.state.lines, hasLength(2));
        },
      );

      blocTest<NoteBloc, NoteState>(
        'leaves it alone when other notes are written',
        build: build,
        act: (bloc) async {
          bloc.add(const NoteEvent.started(MockIds.syncDesign, 'kanso-light'));
          await bloc.stream.firstWhere(
            (state) => state.status == NoteScreenStatus.ready,
          );
          changes.add(<String>['note:other', 'book:work']);
        },
        wait: const Duration(milliseconds: 300),
        verify: (bloc) {
          verify(() => library.note(any())).called(1);
        },
      );

      blocTest<NoteBloc, NoteState>(
        'is not found once the note is gone',
        build: build,
        act: (bloc) async {
          bloc.add(const NoteEvent.started(MockIds.syncDesign, 'kanso-light'));
          await bloc.stream.firstWhere(
            (state) => state.status == NoteScreenStatus.ready,
          );
          when(() => library.note(any())).thenAnswer(
            (_) async => const Left<ApplicationError, NoteView>(
              ApplicationError.notFound(message: 'Not found.'),
            ),
          );
          changes.add(<String>[MockIds.syncDesign]);
        },
        wait: const Duration(milliseconds: 300),
        verify: (bloc) {
          expect(bloc.state.status, NoteScreenStatus.notFound);
          expect(bloc.state.lines, isEmpty);
        },
      );

      blocTest<NoteBloc, NoteState>(
        'becomes locked when the note is locked meanwhile',
        build: build,
        act: (bloc) async {
          bloc.add(const NoteEvent.started(MockIds.syncDesign, 'kanso-light'));
          await bloc.stream.firstWhere(
            (state) => state.status == NoteScreenStatus.ready,
          );
          note = mockLockedNote;
          changes.add(<String>[MockIds.syncDesign]);
        },
        wait: const Duration(milliseconds: 300),
        verify: (bloc) {
          expect(bloc.state.status, NoteScreenStatus.locked);
          expect(bloc.state.lines, isEmpty);
          expect(bloc.state.note?.body, isNull);
        },
      );
    });

    group('the theme', () {
      blocTest<NoteBloc, NoteState>(
        'highlights the text again in the new theme, without reading it',
        build: build,
        act: (bloc) async {
          bloc.add(const NoteEvent.started(MockIds.syncDesign, 'kanso-light'));
          await bloc.stream.firstWhere(
            (state) => state.status == NoteScreenStatus.ready,
          );
          bloc.add(const NoteEvent.syntaxThemeChanged('kanso-dark'));
        },
        verify: (bloc) {
          expect(bloc.state.themeId, 'kanso-dark');
          verify(() => library.note(any())).called(1);
          final themes = verify(
            () => renderer.styledRuns(
              any(),
              start: any(named: 'start'),
              end: any(named: 'end'),
              themeId: captureAny(named: 'themeId'),
            ),
          ).captured;
          expect(themes, <Object?>['kanso-light', 'kanso-dark']);
        },
      );

      blocTest<NoteBloc, NoteState>(
        'does nothing for the theme it has',
        build: build,
        act: (bloc) async {
          bloc.add(const NoteEvent.started(MockIds.syncDesign, 'kanso-light'));
          await bloc.stream.firstWhere(
            (state) => state.status == NoteScreenStatus.ready,
          );
          bloc.add(const NoteEvent.syntaxThemeChanged('kanso-light'));
        },
        verify: (bloc) {
          verify(
            () => renderer.styledRuns(
              any(),
              start: any(named: 'start'),
              end: any(named: 'end'),
              themeId: any(named: 'themeId'),
            ),
          ).called(1);
        },
      );

      blocTest<NoteBloc, NoteState>(
        'keeps a locked note unhighlighted when the theme changes',
        setUp: () {
          note = mockLockedNote;
        },
        build: build,
        act: (bloc) async {
          bloc.add(
            const NoteEvent.started(MockIds.privateJournal, 'kanso-light'),
          );
          await bloc.stream.firstWhere(
            (state) => state.status == NoteScreenStatus.locked,
          );
          bloc.add(const NoteEvent.syntaxThemeChanged('kanso-dark'));
        },
        verify: (bloc) {
          expect(bloc.state.themeId, 'kanso-dark');
          expect(bloc.state.lines, isEmpty);
          expectNoHighlighting();
        },
      );
    });

    blocTest<NoteBloc, NoteState>(
      'another note starts over, and a reading it overtook is dropped',
      setUp: () {
        final slow = Completer<Either<ApplicationError, NoteView>>();
        when(() => library.note('slow')).thenAnswer((_) => slow.future);
        when(() => library.note('fast')).thenAnswer((_) async {
          Timer(const Duration(milliseconds: 20), () {
            slow.complete(Right<ApplicationError, NoteView>(mockOpenNote));
          });
          return Right<ApplicationError, NoteView>(mockLockedNote);
        });
      },
      build: build,
      act: (bloc) async {
        bloc
          ..add(const NoteEvent.started('slow', 'kanso-light'))
          ..add(const NoteEvent.started('fast', 'kanso-light'));
        await Future<void>.delayed(const Duration(milliseconds: 100));
      },
      verify: (bloc) {
        expect(bloc.state.noteId, 'fast');
        expect(bloc.state.status, NoteScreenStatus.locked);
      },
    );

    group('the mode', () {
      Future<NoteBloc> started() async {
        final bloc = build()
          ..add(const NoteEvent.started(MockIds.syncDesign, 'kanso-light'));
        addTearDown(bloc.close);
        await bloc.stream.firstWhere(
          (state) => state.status == NoteScreenStatus.ready,
        );
        return bloc;
      }

      test('a note opens to read', () async {
        final bloc = await started();
        expect(bloc.state.mode, NoteMode.preview);
        expect(bloc.state.caret, isNull);
      });

      test('a note just made opens to edit, once', () async {
        places.markOpenedNew(MockIds.syncDesign);
        final bloc = await started();
        expect(bloc.state.mode, NoteMode.edit);
        expect(bloc.state.openedNew, isTrue);
        expect(bloc.state.startInText, isFalse);
        final again = build()
          ..add(const NoteEvent.started(MockIds.syncDesign, 'kanso-light'));
        addTearDown(again.close);
        await again.stream.firstWhere(
          (state) => state.status == NoteScreenStatus.ready,
        );
        expect(again.state.mode, NoteMode.preview);
        expect(again.state.openedNew, isFalse);
      });

      test('a note made for its text opens in the body', () async {
        places.markOpenedNew(MockIds.syncDesign, start: NewNoteStart.text);
        final bloc = await started();
        expect(bloc.state.mode, NoteMode.edit);
        expect(bloc.state.openedNew, isTrue);
        expect(bloc.state.startInText, isTrue);
      });

      test('edit with a tap names the caret in the whole body', () async {
        final bloc = await started();
        bloc.add(const NoteEvent.modeChanged(NoteMode.edit, bodyCaret: 42));
        await bloc.stream.firstWhere((state) => state.mode == NoteMode.edit);
        expect(bloc.state.caret, 42);
        expect(bloc.state.caretInBody, isTrue);
        expect(bloc.state.scroll, 0);
        bloc.add(const NoteEvent.modeChanged(NoteMode.preview));
        await bloc.stream.firstWhere((state) => state.mode == NoteMode.preview);
        expect(bloc.state.caret, isNull);
        expect(bloc.state.caretInBody, isFalse);
      });

      test('edit with no tap comes back to where it was left', () async {
        final bloc = await started();
        bloc
          ..add(const NoteEvent.modeChanged(NoteMode.edit))
          ..add(const NoteEvent.placeKept(17, 240.5))
          ..add(const NoteEvent.modeChanged(NoteMode.preview));
        await bloc.stream.firstWhere((state) => state.mode == NoteMode.preview);
        expect(
          places.placeOf(MockIds.syncDesign),
          const NotePlace(caret: 17, scroll: 240.5),
        );
        bloc.add(const NoteEvent.modeChanged(NoteMode.edit));
        final state = await bloc.stream.firstWhere(
          (state) => state.mode == NoteMode.edit,
        );
        expect(state.caret, 17);
        expect(state.caretInBody, isFalse);
        expect(state.scroll, 240.5);
      });

      test('writes while editing are read once, on turning to read', () async {
        final bloc = await started();
        clearInteractions(library);
        bloc.add(const NoteEvent.modeChanged(NoteMode.edit));
        await bloc.stream.firstWhere((state) => state.mode == NoteMode.edit);
        note = mockNote(title: 'Saved while editing', body: note.body);
        changes.add(<String>[MockIds.syncDesign]);
        await Future<void>.delayed(noteDebounce * 3);
        verifyNever(() => library.note(any()));
        expect(bloc.state.note?.title, isNot('Saved while editing'));
        bloc.add(const NoteEvent.modeChanged(NoteMode.preview));
        await bloc.stream.firstWhere(
          (state) => state.note?.title == 'Saved while editing',
        );
        verify(() => library.note(MockIds.syncDesign)).called(1);
      });

      test('writes that came while reading are not read again', () async {
        final bloc = await started();
        clearInteractions(library);
        changes.add(<String>[MockIds.syncDesign]);
        await Future<void>.delayed(noteDebounce * 3);
        verify(() => library.note(MockIds.syncDesign)).called(1);
        bloc
          ..add(const NoteEvent.modeChanged(NoteMode.edit))
          ..add(const NoteEvent.modeChanged(NoteMode.preview));
        await Future<void>.delayed(noteDebounce);
        verifyNoMoreInteractions(library);
      });
    });

    test('stops listening to the library when it is closed', () async {
      final bloc = build()
        ..add(const NoteEvent.started(MockIds.syncDesign, 'kanso-light'));
      await bloc.stream.firstWhere(
        (state) => state.status == NoteScreenStatus.ready,
      );
      expect(changes.hasListener, isTrue);
      await bloc.close();
      expect(changes.hasListener, isFalse);
    });
  });
}
