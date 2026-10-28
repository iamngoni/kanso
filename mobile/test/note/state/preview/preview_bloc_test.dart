// Kanso — preview_bloc_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';
import 'dart:typed_data';

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' as glados;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/state/undo_notice/undo_notice_bloc.dart';
import 'package:kanso/note/models/data/preview_action.dart';
import 'package:kanso/note/state/preview/preview_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/library_fixtures.dart';
import '../../../helpers/mock_repositories.dart';
import '../../../helpers/seeded_appearance_bloc.dart'
    show MockRendererRepository;
import '../../../helpers/theme_tokens.dart';

final ThemeTokens _tokens = tokensFor(fakePalette('nord'));

const _failure = ApplicationError.storage(message: 'Disk error.');

void main() {
  late MockLibraryRepository library;
  late MockRendererRepository renderer;
  late MockWebLinkRepository links;
  late MockUndoNoticeBloc undoNotice;
  late MockHapticsRepository haptics;
  late StreamController<List<String>> changes;
  late int drawn;

  setUpAll(() {
    registerFallbackValue(_tokens.palette);
    registerFallbackValue(
      const PreviewOptions(size: 16, utcOffsetSecs: 0, code: []),
    );
    registerFallbackValue(const UndoNoticeEvent.dismissed());
  });

  setUp(() {
    drawn = 0;
    changes = StreamController<List<String>>.broadcast();
    addTearDown(changes.close);
    library = MockLibraryRepository();
    renderer = MockRendererRepository();
    links = MockWebLinkRepository();
    undoNotice = MockUndoNoticeBloc();
    haptics = MockHapticsRepository();
    when(() => library.changes()).thenAnswer((_) => changes.stream);
    when(() => library.note(any())).thenAnswer(
      (_) async => Right<ApplicationError, NoteView>(mockOpenNote),
    );
    when(() => renderer.builtinThemeTokens(any())).thenAnswer(
      (_) async => Right<ApplicationError, ThemeTokens>(_tokens),
    );
    when(() => renderer.previewPage(any(), any(), any())).thenAnswer(
      (_) async => Right<ApplicationError, String>('<p>page ${++drawn}</p>'),
    );
    when(() => haptics.selection()).thenAnswer((_) async {});
  });

  PreviewBloc build() =>
      PreviewBloc(library, renderer, links, undoNotice, haptics);

  void answer(String url, PreviewLink target) => when(
    () => renderer.linkTarget(any(), url),
  ).thenAnswer((_) async => Right<ApplicationError, PreviewLink>(target));

  Future<PreviewBloc> started() async {
    final bloc = build()..add(PreviewEvent.started(mockOpenNote.id, 'nord'));
    addTearDown(bloc.close);
    await bloc.stream.firstWhere((state) => state.html != null);
    return bloc;
  }

  group('PreviewBloc', () {
    test(
      "draws the note's page in the theme, with its code colours",
      () async {
        final bloc = await started();
        expect(bloc.state.html, '<p>page 1</p>');
        expect(bloc.state.rev, mockOpenNote.rev);
        final options =
            verify(
                  () => renderer.previewPage(
                    mockOpenNote.id,
                    _tokens.palette,
                    captureAny(),
                  ),
                ).captured.single
                as PreviewOptions;
        expect(options.size, previewTextSize);
        expect(options.code, _tokens.syntax);
      },
    );

    test('draws it again when the library changes', () async {
      final bloc = await started();
      changes.add(<String>['note:other']);
      await bloc.stream.firstWhere((state) => state.html == '<p>page 2</p>');
    });

    test('a page that cannot be drawn says why', () async {
      when(() => renderer.previewPage(any(), any(), any())).thenAnswer(
        (_) async => const Left<ApplicationError, String>(_failure),
      );
      final bloc = build()..add(PreviewEvent.started(mockOpenNote.id, 'nord'));
      addTearDown(bloc.close);
      final state = await bloc.stream.firstWhere((s) => s.error != null);
      expect(state.error, _failure);
      expect(state.html, isNull);
    });

    test('a note link opens the note', () async {
      final bloc = await started();
      answer('inkdrop://note/x', const PreviewLink.note(id: 'note:x'));
      bloc.add(const PreviewEvent.linkTapped('inkdrop://note/x'));
      final state = await bloc.stream.firstWhere((s) => s.action != null);
      expect(state.action, const PreviewAction.openNote('note:x'));
      bloc.add(const PreviewEvent.actionHandled());
      await bloc.stream.firstWhere((s) => s.action == null);
    });

    test('a link to a note not made yet makes it beside this one', () async {
      final bloc = await started();
      answer('kanso-new:Seed', const PreviewLink.newNote(title: 'Seed'));
      when(
        () =>
            library.createLinkedNote(title: 'Seed', fromNote: mockOpenNote.id),
      ).thenAnswer(
        (_) async => const Right<ApplicationError, Saved>(
          Saved(id: 'note:seed', rev: '1'),
        ),
      );
      bloc.add(const PreviewEvent.linkTapped('kanso-new:Seed'));
      final state = await bloc.stream.firstWhere((s) => s.action != null);
      expect(state.action, const PreviewAction.openNote('note:seed'));
    });

    test('a tag, a heading and an attachment go to the screen', () async {
      final bloc = await started();
      final file = AttachmentFile(
        id: 'file:a',
        name: 'a.pdf',
        contentType: 'application/pdf',
        bytes: Uint8List.fromList(<int>[1, 2, 3]),
      );
      answer('kanso-tag:work', const PreviewLink.tag(name: 'work'));
      answer('#plan', const PreviewLink.heading(id: 'plan'));
      answer(
        'inkdrop://file:a',
        const PreviewLink.attachment(
          id: 'file:a',
          name: 'a.pdf',
          contentType: 'application/pdf',
          size: 3,
        ),
      );
      when(
        () => library.attachment('file:a'),
      ).thenAnswer(
        (_) async => Right<ApplicationError, AttachmentFile>(file),
      );
      for (final (url, action) in <(String, PreviewAction)>[
        ('kanso-tag:work', const PreviewAction.openTag('work')),
        ('#plan', const PreviewAction.scrollTo('plan')),
        ('inkdrop://file:a', PreviewAction.showAttachment(file)),
      ]) {
        bloc.add(PreviewEvent.linkTapped(url));
        final state = await bloc.stream.firstWhere((s) => s.action != null);
        expect(state.action, action);
        bloc.add(const PreviewEvent.actionHandled());
        await bloc.stream.firstWhere((s) => s.action == null);
      }
    });

    test('a web address goes to the system', () async {
      final bloc = await started();
      answer(
        'https://example.com',
        const PreviewLink.web(url: 'https://example.com'),
      );
      when(() => links.open(any())).thenAnswer(
        (_) async => const Right<ApplicationError, Unit>(unit),
      );
      bloc.add(const PreviewEvent.linkTapped('https://example.com'));
      await pumpEventQueue();
      verify(() => links.open('https://example.com')).called(1);
      expect(bloc.state.notice, isNull);
    });

    test('a code block is copied', () async {
      final bloc = await started();
      answer('kanso-copy:0', const PreviewLink.copyCode(index: 0));
      when(() => renderer.codeBlock(mockOpenNote.id, 0)).thenAnswer(
        (_) async => const Right<ApplicationError, String>('let x = 1;'),
      );
      bloc.add(const PreviewEvent.linkTapped('kanso-copy:0'));
      final state = await bloc.stream.firstWhere((s) => s.action != null);
      expect(state.action, const PreviewAction.copyCode(0, 'let x = 1;'));
    });

    test('a ticked box is one write against the page, with Undo', () async {
      final bloc = await started();
      answer('kanso-task:2', const PreviewLink.task(index: 2));
      when(
        () => library.toggleTaskAt(
          noteId: any(named: 'noteId'),
          index: any(named: 'index'),
          baseRev: any(named: 'baseRev'),
        ),
      ).thenAnswer(
        (_) async => const Right<ApplicationError, Ticked>(
          Ticked.changed(rev: 'r2', undo: 9, message: 'Ticked off “Sweep”'),
        ),
      );
      bloc.add(const PreviewEvent.linkTapped('kanso-task:2'));
      await bloc.stream.firstWhere((s) => s.html == '<p>page 2</p>');
      verify(
        () => library.toggleTaskAt(
          noteId: mockOpenNote.id,
          index: 2,
          baseRev: mockOpenNote.rev,
        ),
      ).called(1);
      verify(
        () => undoNotice.add(
          const UndoNoticeEvent.shown(9, 'Ticked off “Sweep”'),
        ),
      ).called(1);
    });

    test('a box on a page older than the note draws the page again', () async {
      final bloc = await started();
      answer('kanso-task:0', const PreviewLink.task(index: 0));
      when(
        () => library.toggleTaskAt(
          noteId: any(named: 'noteId'),
          index: any(named: 'index'),
          baseRev: any(named: 'baseRev'),
        ),
      ).thenAnswer(
        (_) async => const Left<ApplicationError, Ticked>(
          ApplicationError.conflict(message: 'Changed.', currentRev: '3'),
        ),
      );
      bloc.add(const PreviewEvent.linkTapped('kanso-task:0'));
      await bloc.stream.firstWhere((s) => s.html == '<p>page 2</p>');
      expect(bloc.state.notice, isNull);
      verifyNever(() => undoNotice.add(any()));
    });

    test('a tap that leads nowhere does nothing', () async {
      final bloc = await started();
      for (final target in <PreviewLink>[
        const PreviewLink.nothing(),
        const PreviewLink.due(),
      ]) {
        answer('x', target);
        bloc.add(const PreviewEvent.linkTapped('x'));
        await pumpEventQueue();
        expect(bloc.state.action, isNull);
        expect(bloc.state.notice, isNull);
      }
    });

    test('a link to a locked note made elsewhere says so', () async {
      final bloc = await started();
      answer('kanso-new:Seed', const PreviewLink.newNote(title: 'Seed'));
      when(
        () => library.createLinkedNote(
          title: any(named: 'title'),
          fromNote: any(named: 'fromNote'),
        ),
      ).thenAnswer(
        (_) async => const Left<ApplicationError, Saved>(
          ApplicationError.locked(message: 'Locked.'),
        ),
      );
      bloc.add(const PreviewEvent.linkTapped('kanso-new:Seed'));
      final state = await bloc.stream.firstWhere((s) => s.notice != null);
      expect(state.notice, isA<LockedError>());
    });
  });

  // Property: whatever the core says a tap is, the BLoC does at most one
  // thing for it, and never both an action and a notice.
  glados.Glados<int>(glados.any.intInRange(0, 10)).test(
    'a tap ends in one action, one notice or nothing',
    (pick) async {
      final bloc = await started();
      final targets = <PreviewLink>[
        const PreviewLink.note(id: 'note:x'),
        const PreviewLink.newNote(title: 'T'),
        const PreviewLink.tag(name: 't'),
        const PreviewLink.heading(id: 'h'),
        const PreviewLink.web(url: 'https://e.com'),
        const PreviewLink.attachment(
          id: 'file:a',
          name: 'a',
          contentType: 'x',
          size: 1,
        ),
        const PreviewLink.task(index: 0),
        const PreviewLink.copyCode(index: 0),
        const PreviewLink.due(),
        const PreviewLink.nothing(),
      ];
      answer('u', targets[pick % targets.length]);
      when(
        () => library.createLinkedNote(
          title: any(named: 'title'),
          fromNote: any(named: 'fromNote'),
        ),
      ).thenAnswer(
        (_) async => const Left<ApplicationError, Saved>(_failure),
      );
      when(() => library.attachment(any())).thenAnswer(
        (_) async => const Left<ApplicationError, AttachmentFile>(_failure),
      );
      when(() => links.open(any())).thenAnswer(
        (_) async => const Right<ApplicationError, Unit>(unit),
      );
      when(() => renderer.codeBlock(any(), any())).thenAnswer(
        (_) async => const Right<ApplicationError, String>('c'),
      );
      when(
        () => library.toggleTaskAt(
          noteId: any(named: 'noteId'),
          index: any(named: 'index'),
          baseRev: any(named: 'baseRev'),
        ),
      ).thenAnswer(
        (_) async => const Right<ApplicationError, Ticked>(Ticked.unchanged()),
      );
      bloc.add(const PreviewEvent.linkTapped('u'));
      await pumpEventQueue();
      expect(
        bloc.state.action != null && bloc.state.notice != null,
        isFalse,
      );
      await bloc.close();
    },
  );
}
