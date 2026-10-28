// Kanso — share_note_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/sharing/models/data/share_done.dart';
import 'package:kanso/sharing/models/data/share_problem.dart';
import 'package:kanso/sharing/models/data/share_prompt.dart';
import 'package:kanso/sharing/models/data/share_sheet_status.dart';
import 'package:kanso/sharing/state/share_note/share_note_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/mock_repositories.dart';
import '../../../helpers/settle.dart';
import '../../../helpers/sharing_fixtures.dart';

Either<ApplicationError, T> _ok<T>(T value) =>
    Right<ApplicationError, T>(value);

Either<ApplicationError, T> _bad<T>(ApplicationError error) =>
    Left<ApplicationError, T>(error);

void main() {
  late MockSharedNotesRepository shared;
  late MockSystemShareRepository system;

  setUpAll(() {
    registerFallbackValue(ShareRole.viewer);
    registerFallbackValue(<(String, ShareRole)>[]);
  });

  setUp(() {
    shared = MockSharedNotesRepository();
    system = MockSystemShareRepository();
  });

  ShareNoteBloc build() => ShareNoteBloc(shared, system);

  void notShared() {
    when(
      () => shared.info('n1'),
    ).thenAnswer((_) async => _ok<ShareInfo?>(null));
    when(() => shared.canShare('n1')).thenAnswer((_) async => _ok(unit));
  }

  void sharedWith(ShareInfo info) {
    when(
      () => shared.info('n1'),
    ).thenAnswer((_) async => _ok<ShareInfo?>(info));
  }

  final ana = personRow('ana', verified: true);
  final fresh = personRow('bo');

  group('ShareNoteBloc reading the sharing', () {
    blocTest<ShareNoteBloc, ShareNoteState>(
      'a note that is not shared yet can be shared: the invite field, nothing '
      'else',
      setUp: notShared,
      build: build,
      act: (bloc) => bloc.add(const ShareNoteEvent.started('n1')),
      expect: () => <ShareNoteState>[
        // Opened for the note, and then ready: the same state twice in a row
        // is one.
        const ShareNoteState(noteId: 'n1'),
        const ShareNoteState(noteId: 'n1', status: ShareSheetStatus.ready),
      ],
      verify: (bloc) {
        expect(bloc.state.info, isNull);
        expect(bloc.state.manages, isTrue);
      },
    );

    final owned = shareInfo('n1');

    blocTest<ShareNoteBloc, ShareNoteState>(
      'a shared note shows its sharing',
      setUp: () => sharedWith(owned),
      build: build,
      act: (bloc) => bloc.add(const ShareNoteEvent.started('n1')),
      verify: (bloc) {
        expect(bloc.state.status, ShareSheetStatus.ready);
        expect(bloc.state.info, same(owned));
        verifyNever(() => shared.canShare(any()));
      },
    );

    blocTest<ShareNoteBloc, ShareNoteState>(
      'a locked note is locked and nothing else is asked of it',
      setUp: () {
        when(() => shared.info('n1')).thenAnswer(
          (_) async => _bad<ShareInfo?>(
            const ApplicationError.locked(message: 'Locked.'),
          ),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const ShareNoteEvent.started('n1')),
      verify: (bloc) {
        expect(bloc.state.status, ShareSheetStatus.locked);
        expect(bloc.state.info, isNull);
        verifyNever(() => shared.canShare(any()));
        verifyNever(() => shared.lookupPerson(any()));
      },
    );

    blocTest<ShareNoteBloc, ShareNoteState>(
      'a locked note is locked when canShare says so too',
      setUp: () {
        when(
          () => shared.info('n1'),
        ).thenAnswer((_) async => _ok<ShareInfo?>(null));
        when(() => shared.canShare('n1')).thenAnswer(
          (_) async => _bad(const ApplicationError.locked(message: 'Locked.')),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const ShareNoteEvent.started('n1')),
      verify: (bloc) => expect(bloc.state.status, ShareSheetStatus.locked),
    );

    blocTest<ShareNoteBloc, ShareNoteState>(
      'without an account it says so',
      setUp: () {
        when(() => shared.info('n1')).thenAnswer(
          (_) async => _bad<ShareInfo?>(
            const ApplicationError.signedOut(message: 'Not signed in.'),
          ),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const ShareNoteEvent.started('n1')),
      verify: (bloc) => expect(bloc.state.status, ShareSheetStatus.signedOut),
    );

    blocTest<ShareNoteBloc, ShareNoteState>(
      'a note the library refuses to share gives its reason',
      setUp: () {
        when(
          () => shared.info('n1'),
        ).thenAnswer((_) async => _ok<ShareInfo?>(null));
        when(() => shared.canShare('n1')).thenAnswer(
          (_) async => _bad(
            const ApplicationError.invalid(
              message: 'The note is in the trash.',
            ),
          ),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const ShareNoteEvent.started('n1')),
      verify: (bloc) {
        expect(bloc.state.status, ShareSheetStatus.notShareable);
        expect(bloc.state.reason, 'The note is in the trash.');
      },
    );

    blocTest<ShareNoteBloc, ShareNoteState>(
      'a sharing that could not be read fails, and is read again when asked',
      setUp: () {
        var calls = 0;
        when(() => shared.info('n1')).thenAnswer((_) async {
          calls++;
          return calls == 1
              ? _bad<ShareInfo?>(
                  const ApplicationError.network(message: 'Off.'),
                )
              : _ok<ShareInfo?>(owned);
        });
      },
      build: build,
      act: (bloc) async {
        bloc.add(const ShareNoteEvent.started('n1'));
        await settle();
        expect(bloc.state.status, ShareSheetStatus.failed);
        bloc.add(const ShareNoteEvent.retried());
      },
      verify: (bloc) {
        expect(bloc.state.status, ShareSheetStatus.ready);
        expect(bloc.state.info, same(owned));
      },
    );
  });

  group('ShareNoteBloc inviting', () {
    blocTest<ShareNoteBloc, ShareNoteState>(
      'with no username typed it says so and asks nobody',
      setUp: notShared,
      build: build,
      act: (bloc) async {
        bloc.add(const ShareNoteEvent.started('n1'));
        await settle();
        bloc.add(const ShareNoteEvent.invited('   '));
      },
      verify: (bloc) {
        expect(bloc.state.problem, const ShareProblem.nameMissing());
        verifyNever(() => shared.lookupPerson(any()));
      },
    );

    blocTest<ShareNoteBloc, ShareNoteState>(
      'the first person invited, with keys already compared, makes the note '
      'shared with the role chosen',
      setUp: () {
        notShared();
        when(
          () => shared.lookupPerson('ana'),
        ).thenAnswer((_) async => _ok(PersonLookup.found(ana)));
        when(() => shared.share('n1', any())).thenAnswer(
          (_) async => _ok(
            shareInfo(
              'n1',
              members: <MemberRow>[
                memberRow('ngoni', role: ShareRole.owner, me: true),
                memberRow('ana', role: ShareRole.viewer, invited: true),
              ],
            ),
          ),
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const ShareNoteEvent.started('n1'));
        await settle();
        bloc
          ..add(const ShareNoteEvent.inviteRoleChosen(ShareRole.viewer))
          ..add(const ShareNoteEvent.invited(' ana '));
      },
      verify: (bloc) {
        verify(
          () => shared.share('n1', <(String, ShareRole)>[
            ('ana', ShareRole.viewer),
          ]),
        ).called(1);
        expect(bloc.state.done, const ShareDone.invited('ana'));
        expect(bloc.state.info!.members.map((m) => m.username), <String>[
          'ngoni',
          'ana',
        ]);
        expect(bloc.state.busy, isFalse);
        expect(bloc.state.problem, isNull);
      },
    );

    blocTest<ShareNoteBloc, ShareNoteState>(
      'on a shared note the person is invited, not the note shared again',
      setUp: () {
        sharedWith(shareInfo('n1'));
        when(
          () => shared.lookupPerson('ana'),
        ).thenAnswer((_) async => _ok(PersonLookup.found(ana)));
        when(() => shared.invite('n1', 'ana', ShareRole.editor)).thenAnswer(
          (_) async => _ok(
            shareInfo(
              'n1',
              members: <MemberRow>[
                memberRow('ngoni', role: ShareRole.owner, me: true),
                memberRow('ana', invited: true),
              ],
            ),
          ),
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const ShareNoteEvent.started('n1'));
        await settle();
        bloc.add(const ShareNoteEvent.invited('ana'));
      },
      verify: (bloc) {
        verify(() => shared.invite('n1', 'ana', ShareRole.editor)).called(1);
        verifyNever(() => shared.share(any(), any()));
        expect(bloc.state.done, const ShareDone.invited('ana'));
      },
    );

    blocTest<ShareNoteBloc, ShareNoteState>(
      'someone new asks for their fingerprint to be compared first, and is '
      'invited only once it is',
      setUp: () {
        notShared();
        when(
          () => shared.lookupPerson('bo'),
        ).thenAnswer((_) async => _ok(PersonLookup.found(fresh)));
        when(
          () => shared.verifyPerson('a_bo'),
        ).thenAnswer((_) async => _ok(unit));
        when(
          () => shared.share('n1', any()),
        ).thenAnswer((_) async => _ok(shareInfo('n1')));
      },
      build: build,
      act: (bloc) async {
        bloc.add(const ShareNoteEvent.started('n1'));
        await settle();
        bloc.add(const ShareNoteEvent.invited('bo'));
        await settle();
        expect(
          bloc.state.prompt,
          SharePrompt.verify(person: fresh, role: ShareRole.editor),
        );
        verifyNever(() => shared.share(any(), any()));
        bloc.add(const ShareNoteEvent.fingerprintConfirmed());
      },
      verify: (bloc) {
        verifyInOrder(<void Function()>[
          () => shared.verifyPerson('a_bo'),
          () => shared.share('n1', <(String, ShareRole)>[
            ('bo', ShareRole.editor),
          ]),
        ]);
        expect(bloc.state.prompt, isNull);
        expect(bloc.state.done, const ShareDone.invited('bo'));
      },
    );

    blocTest<ShareNoteBloc, ShareNoteState>(
      'leaving the fingerprint question invites nobody',
      setUp: () {
        notShared();
        when(
          () => shared.lookupPerson('bo'),
        ).thenAnswer((_) async => _ok(PersonLookup.found(fresh)));
      },
      build: build,
      act: (bloc) async {
        bloc.add(const ShareNoteEvent.started('n1'));
        await settle();
        bloc.add(const ShareNoteEvent.invited('bo'));
        await settle();
        bloc.add(const ShareNoteEvent.promptDismissed());
      },
      verify: (bloc) {
        expect(bloc.state.prompt, isNull);
        verifyNever(() => shared.share(any(), any()));
        verifyNever(() => shared.verifyPerson(any()));
      },
    );

    blocTest<ShareNoteBloc, ShareNoteState>(
      'someone new can be invited without comparing the fingerprint, which '
      'then stays unverified',
      setUp: () {
        notShared();
        when(
          () => shared.lookupPerson('bo'),
        ).thenAnswer((_) async => _ok(PersonLookup.found(fresh)));
        when(
          () => shared.share('n1', any()),
        ).thenAnswer((_) async => _ok(shareInfo('n1')));
      },
      build: build,
      act: (bloc) async {
        bloc.add(const ShareNoteEvent.started('n1'));
        await settle();
        bloc.add(const ShareNoteEvent.invited('bo'));
        await settle();
        bloc.add(const ShareNoteEvent.invitedUnverified());
      },
      verify: (bloc) {
        verifyNever(() => shared.verifyPerson(any()));
        verify(
          () => shared.share('n1', <(String, ShareRole)>[
            ('bo', ShareRole.editor),
          ]),
        ).called(1);
        expect(bloc.state.prompt, isNull);
        expect(bloc.state.done, const ShareDone.invited('bo'));
      },
    );

    blocTest<ShareNoteBloc, ShareNoteState>(
      'inviting without comparing does nothing when no fingerprint is asked '
      'about',
      setUp: notShared,
      build: build,
      act: (bloc) async {
        bloc.add(const ShareNoteEvent.started('n1'));
        await settle();
        bloc.add(const ShareNoteEvent.invitedUnverified());
      },
      verify: (_) => verifyNever(() => shared.share(any(), any())),
    );

    blocTest<ShareNoteBloc, ShareNoteState>(
      'someone whose keys changed shows the old and the new fingerprint, and '
      'the new keys are trusted only when the person says so',
      setUp: () {
        notShared();
        final before = personRow('bo', fingerprint: '11111 11111');
        final now = personRow('bo', fingerprint: '99999 99999');
        when(() => shared.lookupPerson('bo')).thenAnswer(
          (_) async => _ok(PersonLookup.changed(pinned: before, offered: now)),
        );
        when(
          () => shared.acceptNewKey('bo', '99999 99999'),
        ).thenAnswer((_) async => _ok(now));
        when(
          () => shared.share('n1', any()),
        ).thenAnswer((_) async => _ok(shareInfo('n1')));
      },
      build: build,
      act: (bloc) async {
        bloc.add(const ShareNoteEvent.started('n1'));
        await settle();
        bloc.add(const ShareNoteEvent.invited('bo'));
        await settle();
        final prompt = bloc.state.prompt! as SharePromptKeyChanged;
        expect(prompt.pinned!.fingerprint, '11111 11111');
        expect(prompt.offered.fingerprint, '99999 99999');
        expect(prompt.role, ShareRole.editor);
        verifyNever(() => shared.acceptNewKey(any(), any()));
        bloc.add(const ShareNoteEvent.newKeyAccepted());
        await settle();
        // Trusted is not verified: the new fingerprint is compared next,
        // and nobody is invited before that step is answered.
        expect(
          bloc.state.prompt,
          SharePrompt.verify(
            person: personRow('bo', fingerprint: '99999 99999'),
            role: ShareRole.editor,
          ),
        );
        verifyNever(() => shared.share(any(), any()));
        bloc.add(const ShareNoteEvent.invitedUnverified());
      },
      verify: (bloc) {
        verifyInOrder(<void Function()>[
          () => shared.acceptNewKey('bo', '99999 99999'),
          () => shared.share('n1', <(String, ShareRole)>[
            ('bo', ShareRole.editor),
          ]),
        ]);
        expect(bloc.state.done, const ShareDone.invited('bo'));
      },
    );

    blocTest<ShareNoteBloc, ShareNoteState>(
      'a lookup that fails says why in the library’s words and invites nobody',
      setUp: () {
        notShared();
        when(() => shared.lookupPerson('zed')).thenAnswer(
          (_) async => _bad(
            const ApplicationError.invalid(
              message:
                  'There’s no Kanso account called zed that can share yet.',
            ),
          ),
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const ShareNoteEvent.started('n1'));
        await settle();
        bloc.add(const ShareNoteEvent.invited('zed'));
      },
      verify: (bloc) {
        expect(
          bloc.state.problem,
          const ShareProblem.words(
            'There’s no Kanso account called zed that can share yet.',
          ),
        );
        expect(bloc.state.busy, isFalse);
        verifyNever(() => shared.share(any(), any()));
      },
    );

    blocTest<ShareNoteBloc, ShareNoteState>(
      'an unreachable server is said in the app’s own words',
      setUp: () {
        notShared();
        when(() => shared.lookupPerson('ana')).thenAnswer(
          (_) async => _bad(
            const ApplicationError.network(message: 'dns failure at x.invalid'),
          ),
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const ShareNoteEvent.started('n1'));
        await settle();
        bloc.add(const ShareNoteEvent.invited('ana'));
      },
      verify: (bloc) =>
          expect(bloc.state.problem, const ShareProblem.offline()),
    );
  });

  group('ShareNoteBloc managing a shared note', () {
    final owned = shareInfo(
      'n1',
      members: <MemberRow>[
        memberRow('ngoni', role: ShareRole.owner, me: true),
        memberRow('ana'),
      ],
      requests: <MemberRow>[memberRow('cy', role: ShareRole.viewer)],
      links: const <ShareLinkRow>[
        ShareLinkRow(id: 'l_1', role: ShareRole.viewer, expiresMs: 1),
      ],
    );

    Future<ShareNoteBloc> started() async {
      sharedWith(owned);
      final bloc = build()..add(const ShareNoteEvent.started('n1'));
      await settle();
      return bloc;
    }

    test('changes a role', () async {
      final bloc = await started();
      when(() => shared.setRole('n1', 'a_ana', ShareRole.viewer)).thenAnswer(
        (_) async => _ok(
          shareInfo(
            'n1',
            members: <MemberRow>[
              memberRow('ngoni', role: ShareRole.owner, me: true),
              memberRow('ana', role: ShareRole.viewer),
            ],
          ),
        ),
      );
      bloc.add(
        const ShareNoteEvent.memberRoleChanged('a_ana', ShareRole.viewer),
      );
      await settle();
      expect(bloc.state.info!.members.last.role, ShareRole.viewer);
      expect(bloc.state.busy, isFalse);
      await bloc.close();
    });

    test('removes a person', () async {
      final bloc = await started();
      when(() => shared.remove('n1', 'a_ana')).thenAnswer(
        (_) async => _ok(
          shareInfo(
            'n1',
            members: <MemberRow>[
              memberRow('ngoni', role: ShareRole.owner, me: true),
            ],
          ),
        ),
      );
      bloc.add(const ShareNoteEvent.memberRemoved('a_ana'));
      await settle();
      expect(bloc.state.info!.members.map((m) => m.username), <String>[
        'ngoni',
      ]);
      await bloc.close();
    });

    test('approves and declines the people who asked through a link', () async {
      final bloc = await started();
      when(
        () => shared.approveRequest('n1', 'a_cy'),
      ).thenAnswer((_) async => _ok(shareInfo('n1')));
      when(
        () => shared.declineRequest('n1', 'a_cy'),
      ).thenAnswer((_) async => _ok(shareInfo('n1')));
      bloc.add(const ShareNoteEvent.requestApproved('a_cy'));
      await settle();
      expect(bloc.state.info!.requests, isEmpty);
      bloc.add(const ShareNoteEvent.requestDeclined('a_cy'));
      await settle();
      verify(() => shared.approveRequest('n1', 'a_cy')).called(1);
      verify(() => shared.declineRequest('n1', 'a_cy')).called(1);
      await bloc.close();
    });

    test('cancels a link', () async {
      final bloc = await started();
      when(
        () => shared.cancelLink('n1', 'l_1'),
      ).thenAnswer((_) async => _ok(shareInfo('n1')));
      bloc.add(const ShareNoteEvent.linkCancelled('l_1'));
      await settle();
      expect(bloc.state.info!.links, isEmpty);
      await bloc.close();
    });

    test('a step that fails keeps the sharing and says why', () async {
      final bloc = await started();
      when(() => shared.remove('n1', 'a_ana')).thenAnswer(
        (_) async => _bad(
          const ApplicationError.invalid(
            message: 'Your role in this note doesn’t allow that.',
          ),
        ),
      );
      bloc.add(const ShareNoteEvent.memberRemoved('a_ana'));
      await settle();
      expect(bloc.state.info, owned);
      expect(
        bloc.state.problem,
        const ShareProblem.words('Your role in this note doesn’t allow that.'),
      );
      expect(bloc.state.busy, isFalse);
      await bloc.close();
    });

    test('a sharing step on a note that is not shared does nothing', () async {
      notShared();
      final bloc = build()..add(const ShareNoteEvent.started('n1'));
      await settle();
      bloc
        ..add(const ShareNoteEvent.memberRemoved('a_ana'))
        ..add(const ShareNoteEvent.linkMade())
        ..add(const ShareNoteEvent.endAsked());
      await settle();
      verifyNever(() => shared.remove(any(), any()));
      verifyNever(() => shared.createLink(any(), any(), any()));
      expect(bloc.state.confirmEnd, isFalse);
      await bloc.close();
    });

    test('reviewing a member whose keys are the pinned ones trusts '
        'nothing and asks nothing', () async {
      final bloc = await started();
      when(
        () => shared.lookupPerson('ana'),
      ).thenAnswer((_) async => _ok(PersonLookup.found(personRow('ana'))));
      bloc.add(const ShareNoteEvent.keyReviewed('ana'));
      await settle();
      expect(bloc.state.prompt, isNull);
      expect(bloc.state.busy, isFalse);
      verifyNever(() => shared.acceptNewKey(any(), any()));
      verifyNever(() => shared.verifyPerson(any()));
      await bloc.close();
    });

    test('reviews the new keys of a member', () async {
      final bloc = await started();
      final now = personRow('ana', fingerprint: '55555 55555');
      when(() => shared.lookupPerson('ana')).thenAnswer(
        (_) async => _ok(
          PersonLookup.changed(
            pinned: personRow('ana', fingerprint: '12345 67890'),
            offered: now,
          ),
        ),
      );
      when(
        () => shared.acceptNewKey('ana', '55555 55555'),
      ).thenAnswer((_) async => _ok(now));
      bloc.add(const ShareNoteEvent.keyReviewed('ana'));
      await settle();
      final prompt = bloc.state.prompt! as SharePromptKeyChanged;
      expect(prompt.role, isNull);
      bloc.add(const ShareNoteEvent.newKeyAccepted());
      await settle();
      expect(bloc.state.prompt, isNull);
      verify(() => shared.acceptNewKey('ana', '55555 55555')).called(1);
      verifyNever(() => shared.invite(any(), any(), any()));
      await bloc.close();
    });
  });

  group('ShareNoteBloc links', () {
    final owned = shareInfo('n1');

    Future<ShareNoteBloc> started() async {
      sharedWith(owned);
      final bloc = build()..add(const ShareNoteEvent.started('n1'));
      await settle();
      return bloc;
    }

    test(
      'makes a link with the role and days chosen, and shows it once',
      () async {
        final bloc = await started();
        final withLink = shareInfo(
          'n1',
          links: const <ShareLinkRow>[
            ShareLinkRow(id: 'l_9', role: ShareRole.viewer, expiresMs: 5),
          ],
        );
        when(
          () => shared.createLink('n1', ShareRole.viewer, 30),
        ).thenAnswer((_) async => _ok('https://example.invalid/join/x#k'));
        sharedWith(withLink);
        bloc
          ..add(const ShareNoteEvent.linkRoleChosen(ShareRole.viewer))
          ..add(const ShareNoteEvent.linkDaysChosen(30))
          ..add(const ShareNoteEvent.linkMade());
        await settle();
        expect(bloc.state.madeLink, 'https://example.invalid/join/x#k');
        expect(bloc.state.info, withLink);
        expect(bloc.state.linkCopied, isFalse);
        await bloc.close();
      },
    );

    test('copies the link, and sends it to the system’s share sheet', () async {
      final bloc = await started();
      when(
        () => shared.createLink(any(), any(), any()),
      ).thenAnswer((_) async => _ok('https://example.invalid/join/x#k'));
      when(
        () => system.copy('https://example.invalid/join/x#k'),
      ).thenAnswer((_) async => _ok(unit));
      const origin = Rect.fromLTWH(10, 20, 30, 40);
      when(
        () => system.send('https://example.invalid/join/x#k', origin: origin),
      ).thenAnswer((_) async => _ok(unit));
      bloc.add(const ShareNoteEvent.linkMade());
      await settle();
      bloc.add(const ShareNoteEvent.linkCopied());
      await settle();
      expect(bloc.state.linkCopied, isTrue);
      bloc.add(const ShareNoteEvent.linkSent(origin: origin));
      await settle();
      verify(() => system.copy('https://example.invalid/join/x#k')).called(1);
      verify(
        () => system.send('https://example.invalid/join/x#k', origin: origin),
      ).called(1);
      await bloc.close();
    });

    test('copying with no link made does nothing', () async {
      final bloc = await started();
      bloc.add(const ShareNoteEvent.linkCopied());
      await settle();
      verifyNever(() => system.copy(any()));
      await bloc.close();
    });

    test('a link that could not be made says why', () async {
      final bloc = await started();
      when(() => shared.createLink(any(), any(), any())).thenAnswer(
        (_) async => _bad(const ApplicationError.network(message: 'Off.')),
      );
      bloc.add(const ShareNoteEvent.linkMade());
      await settle();
      expect(bloc.state.madeLink, isNull);
      expect(bloc.state.problem, const ShareProblem.offline());
      await bloc.close();
    });
  });

  group('ShareNoteBloc stopping and leaving', () {
    test('the owner is asked first, then stops sharing', () async {
      sharedWith(shareInfo('n1'));
      when(() => shared.stop('n1')).thenAnswer((_) async => _ok(unit));
      final bloc = build()..add(const ShareNoteEvent.started('n1'));
      await settle();
      bloc.add(const ShareNoteEvent.endConfirmed());
      await settle();
      verifyNever(() => shared.stop(any()));
      bloc.add(const ShareNoteEvent.endAsked());
      await settle();
      expect(bloc.state.confirmEnd, isTrue);
      bloc.add(const ShareNoteEvent.endConfirmed());
      await settle();
      verify(() => shared.stop('n1')).called(1);
      expect(bloc.state.info, isNull);
      expect(bloc.state.done, const ShareDone.stopped());
      expect(bloc.state.confirmEnd, isFalse);
      await bloc.close();
    });

    test('someone else leaves instead', () async {
      sharedWith(
        shareInfo(
          'n1',
          owner: 'ana',
          myRole: ShareRole.editor,
          members: <MemberRow>[
            memberRow('ana', role: ShareRole.owner),
            memberRow('ngoni', me: true),
          ],
        ),
      );
      when(() => shared.leave('n1')).thenAnswer((_) async => _ok(unit));
      final bloc = build()..add(const ShareNoteEvent.started('n1'));
      await settle();
      expect(bloc.state.manages, isFalse);
      bloc
        ..add(const ShareNoteEvent.endAsked())
        ..add(const ShareNoteEvent.endConfirmed());
      await settle();
      verify(() => shared.leave('n1')).called(1);
      verifyNever(() => shared.stop(any()));
      expect(bloc.state.done, const ShareDone.left());
      await bloc.close();
    });

    test('the question can be answered no', () async {
      sharedWith(shareInfo('n1'));
      final bloc = build()..add(const ShareNoteEvent.started('n1'));
      await settle();
      bloc
        ..add(const ShareNoteEvent.endAsked())
        ..add(const ShareNoteEvent.endDismissed());
      await settle();
      expect(bloc.state.confirmEnd, isFalse);
      verifyNever(() => shared.stop(any()));
      await bloc.close();
    });

    test('a refusal keeps the sharing and says why', () async {
      final owned = shareInfo('n1');
      sharedWith(owned);
      when(() => shared.stop('n1')).thenAnswer(
        (_) async => _bad(
          const ApplicationError.invalid(
            message: 'Only the note’s owner can stop sharing it.',
          ),
        ),
      );
      final bloc = build()..add(const ShareNoteEvent.started('n1'));
      await settle();
      bloc
        ..add(const ShareNoteEvent.endAsked())
        ..add(const ShareNoteEvent.endConfirmed());
      await settle();
      expect(bloc.state.info, same(owned));
      expect(bloc.state.confirmEnd, isFalse);
      expect(
        bloc.state.problem,
        const ShareProblem.words('Only the note’s owner can stop sharing it.'),
      );
      await bloc.close();
    });
  });
}
