// Kanso — pdf_reader_bloc_test.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/pdf_types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/pdf_reader/models/data/pdf_opening.dart';
import 'package:kanso/pdf_reader/models/data/pdf_zoom.dart';
import 'package:kanso/pdf_reader/state/pdf_reader/pdf_reader_bloc.dart';

import '../fakes.dart';

const PdfReaderEvent _start = PdfReaderEvent.started(
  noteId: 'note-one',
  file: 'file:pdf-one',
  name: 'Field notes.pdf',
);

PdfReaderBloc _bloc(
  FakePdfReaderRepository repository, [
  FakePdfPreferencesRepository? preferences,
]) => PdfReaderBloc(repository, preferences ?? FakePdfPreferencesRepository());

void main() {
  group('opening', () {
    blocTest<PdfReaderBloc, PdfReaderState>(
      "opens the attachment and reads the note's highlights",
      build: () => _bloc(FakePdfReaderRepository()),
      act: (bloc) => bloc.add(_start),
      wait: const Duration(milliseconds: 40),
      verify: (bloc) {
        expect(bloc.state.status, PdfReaderStatus.opened);
        expect(bloc.state.info?.pages, 3);
        expect(bloc.state.marks?.rev, 'rev-1');
        expect(bloc.state.page, 1);
      },
    );

    blocTest<PdfReaderBloc, PdfReaderState>(
      'a dark reader smart-inverts the pages',
      build: () => _bloc(FakePdfReaderRepository()),
      act: (bloc) => bloc.add(
        const PdfReaderEvent.started(
          noteId: 'note-one',
          file: 'file:pdf-one',
          name: 'Field notes.pdf',
          dark: true,
        ),
      ),
      wait: const Duration(milliseconds: 40),
      verify: (bloc) => expect(bloc.state.invert, isTrue),
    );

    blocTest<PdfReaderBloc, PdfReaderState>(
      'a PDF kept as printed opens without inversion in the dark',
      build: () => _bloc(
        FakePdfReaderRepository(),
        FakePdfPreferencesRepository(
          const Preferences(
            pdfOriginalPages: <String, bool>{'file:pdf-one': true},
          ),
        ),
      ),
      act: (bloc) => bloc.add(
        const PdfReaderEvent.started(
          noteId: 'note-one',
          file: 'file:pdf-one',
          name: 'Field notes.pdf',
          dark: true,
        ),
      ),
      wait: const Duration(milliseconds: 40),
      verify: (bloc) => expect(bloc.state.invert, isFalse),
    );

    blocTest<PdfReaderBloc, PdfReaderState>(
      'opens at the page a link named',
      build: () => _bloc(FakePdfReaderRepository()),
      act: (bloc) => bloc.add(
        const PdfReaderEvent.started(
          noteId: 'note-one',
          file: 'file:pdf-one',
          name: 'Field notes.pdf',
          page: 3,
        ),
      ),
      wait: const Duration(milliseconds: 40),
      verify: (bloc) => expect(bloc.state.page, 3),
    );

    blocTest<PdfReaderBloc, PdfReaderState>(
      'a link to a highlight opens the page it is on',
      build: () => _bloc(FakePdfReaderRepository()),
      act: (bloc) => bloc.add(
        const PdfReaderEvent.started(
          noteId: 'note-one',
          file: 'file:pdf-one',
          name: 'Field notes.pdf',
          highlight: 'mark-2',
        ),
      ),
      wait: const Duration(milliseconds: 40),
      verify: (bloc) {
        expect(bloc.state.page, 3);
        expect(bloc.state.focusHighlight, 'mark-2');
      },
    );

    blocTest<PdfReaderBloc, PdfReaderState>(
      'a locked note says why it cannot open',
      build: () => _bloc(
        FakePdfReaderRepository(
          opening: const PdfOpening.declined(
            reason: PdfDeclined.lockedNote(message: 'Unlock this note.'),
          ),
        ),
      ),
      act: (bloc) => bloc.add(_start),
      wait: const Duration(milliseconds: 40),
      verify: (bloc) {
        expect(bloc.state.status, PdfReaderStatus.declined);
        expect(bloc.state.declined, isA<PdfDeclined_LockedNote>());
      },
    );

    blocTest<PdfReaderBloc, PdfReaderState>(
      'a failure to open is an error, with a way to try again',
      build: () {
        final repository = FakePdfReaderRepository(
          openError: const ApplicationError.storage(message: 'Disk full.'),
        );
        return _bloc(repository);
      },
      act: (bloc) => bloc.add(_start),
      wait: const Duration(milliseconds: 40),
      verify: (bloc) {
        expect(bloc.state.status, PdfReaderStatus.failed);
        expect(bloc.state.error?.message, 'Disk full.');
      },
    );

    test(
      'a password opens an encrypted PDF, and a wrong one is refused',
      () async {
        final repository = FakePdfReaderRepository(
          onOpen: (password) => switch (password) {
            null => const PdfOpening.declined(
              reason: PdfDeclined.needsPassword(),
            ),
            'open sesame' => PdfOpening.opened(
              session: FakePdfSession(),
              info: fakePdfInfo,
            ),
            _ => const PdfOpening.declined(
              reason: PdfDeclined.wrongPassword(),
            ),
          },
        );
        final bloc = _bloc(repository);
        addTearDown(bloc.close);

        bloc.add(_start);
        await bloc.stream.firstWhere(
          (state) => state.status == PdfReaderStatus.declined,
        );
        expect(bloc.state.declined, isA<PdfDeclined_NeedsPassword>());

        bloc.add(const PdfReaderEvent.passwordSubmitted('wrong'));
        await bloc.stream.firstWhere(
          (state) => state.passwordWrong,
        );
        expect(bloc.state.status, PdfReaderStatus.declined);

        bloc.add(const PdfReaderEvent.passwordSubmitted('open sesame'));
        await bloc.stream.firstWhere(
          (state) => state.status == PdfReaderStatus.opened,
        );
        expect(repository.passwords, <String?>[null, 'wrong', 'open sesame']);
      },
    );
  });

  group('reading', () {
    late FakePdfSession session;

    PdfReaderBloc make([FakePdfPreferencesRepository? preferences]) {
      session = FakePdfSession();
      return _bloc(
        FakePdfReaderRepository(
          opening: PdfOpening.opened(session: session, info: fakePdfInfo),
        ),
        preferences,
      );
    }

    Future<PdfReaderBloc> opened() async {
      final bloc = make()..add(_start);
      await bloc.stream.firstWhere(
        (state) =>
            state.status == PdfReaderStatus.opened && state.marks != null,
      );
      return bloc;
    }

    test('holding a word selects it and makes a highlight', () async {
      final bloc = await opened();
      addTearDown(bloc.close);

      bloc.add(const PdfReaderEvent.wordSelected(1, 0.2, 0.3));
      await bloc.stream.firstWhere((state) => state.selection != null);

      bloc.add(
        const PdfReaderEvent.highlightRequested(color: 'blue', quote: true),
      );
      await bloc.stream.firstWhere(
        (state) => state.selection == null && !state.adding,
      );

      expect(session.highlights, hasLength(1));
      final made = session.highlights.single;
      expect(made.baseRev, 'rev-1');
      expect(made.color, 'blue');
      expect(made.quote, isTrue);
      expect(made.start, const PdfCharPos(page: 1, index: 4));
    });

    test('a note that cannot take highlights makes none', () async {
      final readOnly = FakePdfSession();
      final bloc = _bloc(
        FakePdfReaderRepository(
          opening: PdfOpening.opened(session: readOnly, info: fakePdfInfo),
          marks: fakeMarks(editable: false),
        ),
      );
      addTearDown(bloc.close);
      bloc.add(_start);
      await bloc.stream.firstWhere(
        (state) =>
            state.status == PdfReaderStatus.opened && state.marks != null,
      );

      bloc.add(const PdfReaderEvent.wordSelected(1, 0.2, 0.3));
      await bloc.stream.firstWhere((state) => state.selection != null);
      bloc.add(
        const PdfReaderEvent.highlightRequested(color: 'blue', quote: false),
      );
      await Future<void>.delayed(const Duration(milliseconds: 10));

      expect(readOnly.highlights, isEmpty);
      expect(bloc.state.adding, isFalse);
    });

    test('a search finds the hits and steps through them', () async {
      final bloc = await opened();
      addTearDown(bloc.close);

      bloc.add(const PdfReaderEvent.searchChanged('page'));
      await Future<void>.delayed(const Duration(milliseconds: 320));

      expect(bloc.state.found?.total, 2);
      expect(bloc.state.page, 1);

      bloc.add(const PdfReaderEvent.searchNext());
      await bloc.stream.firstWhere((state) => state.hit == 1);
      expect(bloc.state.page, 3);

      bloc.add(const PdfReaderEvent.searchPrevious());
      await bloc.stream.firstWhere((state) => state.hit == 0);
      expect(bloc.state.page, 1);
    });

    test(
      'a theme change keeps a printed PDF printed without writing it',
      () async {
        final preferences = FakePdfPreferencesRepository();
        final bloc = make(preferences)
          ..add(
            const PdfReaderEvent.started(
              noteId: 'note-one',
              file: 'file:pdf-one',
              name: 'Field notes.pdf',
              dark: true,
            ),
          );
        await bloc.stream.firstWhere(
          (state) =>
              state.status == PdfReaderStatus.opened && state.marks != null,
        );
        addTearDown(bloc.close);

        expect(bloc.state.invert, isTrue);

        bloc.add(const PdfReaderEvent.darkChanged(dark: false));
        await Future<void>.delayed(const Duration(milliseconds: 10));
        expect(bloc.state.invert, isFalse);
        expect(preferences.updates, 0);

        bloc.add(const PdfReaderEvent.darkChanged(dark: true));
        await Future<void>.delayed(const Duration(milliseconds: 10));
        expect(bloc.state.invert, isTrue);
        expect(preferences.updates, 0);

        bloc.add(const PdfReaderEvent.invertChanged(invert: false));
        await Future<void>.delayed(const Duration(milliseconds: 10));
        expect(bloc.state.invert, isFalse);
        expect(preferences.updates, 1);

        bloc.add(const PdfReaderEvent.darkChanged(dark: false));
        await Future<void>.delayed(const Duration(milliseconds: 10));
        expect(bloc.state.invert, isFalse);
        bloc.add(const PdfReaderEvent.darkChanged(dark: true));
        await Future<void>.delayed(const Duration(milliseconds: 10));
        expect(bloc.state.invert, isFalse);
        expect(preferences.current.pdfOriginalPages['file:pdf-one'], isTrue);
        expect(preferences.updates, 1);
      },
    );

    test(
      'zoom and dark rendering are kept, and the choice is per PDF',
      () async {
        final preferences = FakePdfPreferencesRepository();
        final bloc = make(preferences)
          ..add(
            const PdfReaderEvent.started(
              noteId: 'note-one',
              file: 'file:pdf-one',
              name: 'Field notes.pdf',
              dark: true,
            ),
          );
        await bloc.stream.firstWhere(
          (state) =>
              state.status == PdfReaderStatus.opened && state.marks != null,
        );
        addTearDown(bloc.close);

        bloc.add(PdfReaderEvent.zoomChanged(PdfZoom.of(4)));
        await bloc.stream.firstWhere((state) => state.zoom.factor == 4);
        bloc.add(const PdfReaderEvent.invertChanged(invert: false));
        await bloc.stream.firstWhere((state) => !state.invert);
        bloc.add(const PdfReaderEvent.pageChanged(2));
        await bloc.stream.firstWhere((state) => state.page == 2);

        expect(bloc.state.zoom.factor, 4);
        expect(bloc.state.invert, isFalse);
        expect(preferences.current.pdfOriginalPages['file:pdf-one'], isTrue);
        expect(preferences.updates, 1);
      },
    );

    test('closing ends the document', () async {
      final bloc = await opened();
      expect(session.disposed, isFalse);
      await bloc.close();
      expect(session.disposed, isTrue);
    });
  });
}
