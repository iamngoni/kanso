// Kanso — fixture_shared_notebooks_repository.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/shared_notebooks_repository.dart';
import 'package:kanso/core/models/repos/abs/shared_notes_repository.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/sharing/state/notebook_sharing/notebook_sharing_bloc.dart';
import 'package:kanso/sharing/state/share_notebook/share_notebook_bloc.dart';
import 'package:kanso/sharing/state/shared_notebooks/shared_notebooks_bloc.dart';

import 'fixture_shared_notes_repository.dart';

/// A member row of a shared notebook, for tests. The roles a menu offers
/// are given as the Rust core would ([roles]), never worked out here.
BookMemberRow bookMember(
  String username, {
  BookRole role = BookRole.editor,
  bool me = false,
  BookStandingRow standing = BookStandingRow.member,
  List<BookRole> roles = const <BookRole>[],
  bool removable = false,
  bool verified = false,
}) => BookMemberRow(
  accountId: 'a_$username',
  username: username,
  role: role,
  me: me,
  standing: standing,
  fingerprint: 'ABCD EFGH IJKL MNOP',
  verified: verified,
  roles: roles,
  removable: removable,
);

/// A notebook's sharing, for tests.
BookShareInfo bookInfo(
  String notebookId, {
  String name = 'Studio',
  BookRole myRole = BookRole.owner,
  String owner = 'ngoni',
  List<BookMemberRow>? members,
  List<BookRole>? inviteRoles,
}) => BookShareInfo(
  shareId: 's_$notebookId',
  notebookId: notebookId,
  name: name,
  myRole: myRole,
  owner: owner,
  members:
      members ??
      <BookMemberRow>[bookMember('ngoni', role: BookRole.owner, me: true)],
  inviteRoles:
      inviteRoles ??
      (myRole == BookRole.owner
          ? const <BookRole>[
              BookRole.manager,
              BookRole.editor,
              BookRole.commenter,
              BookRole.viewer,
            ]
          : const <BookRole>[]),
  owned: myRole == BookRole.owner,
);

/// Shared notebooks for widget tests that behave like the real ones, in
/// memory: a notebook is shared, people are invited, given roles and
/// removed, invitations are accepted and declined, and a notebook is left
/// or stopped. A test sets up what exists and reads [calls] to see what
/// was done.
class FixtureSharedNotebooksRepository implements SharedNotebooksRepository {
  /// The notebooks shared here, by notebook id.
  final Map<String, BookShareInfo> shares = <String, BookShareInfo>{};

  /// What sharing each notebook not shared yet would do.
  final Map<String, BookShareCheck> checks = <String, BookShareCheck>{};

  /// Why notebooks can't be shared, by id.
  final Map<String, String> unshareable = <String, String>{};

  List<BookInvitationRow> invitationList = <BookInvitationRow>[];
  List<SharedBookRow> books = <SharedBookRow>[];
  final Map<String, List<NoteEditRow>> editsOf = <String, List<NoteEditRow>>{};
  final Map<String, BookSyncStatus> statuses = <String, BookSyncStatus>{};

  /// What the Shared section says once the server refused this Kanso as too
  /// old for shared notebooks; null while it is happy.
  String? updateLineText;

  /// No account is signed in: every call fails with [SignedOutError].
  bool signedOut = false;

  /// When set, the calls that change something fail with it.
  ApplicationError? failure;

  /// Progress each share reports, as (done, total).
  List<(int, int)> shareProgress = const <(int, int)>[];

  /// When set, a share waits for it after reporting its progress, so a
  /// test can see the sheet while the notes go up.
  Completer<void>? shareGate;

  /// What was done, as words: `invite book:1 dan editor`.
  final List<String> calls = <String>[];

  final StreamController<BookShareProgress> progressController =
      StreamController<BookShareProgress>.broadcast();

  Either<ApplicationError, T>? _refused<T>({bool change = false}) {
    if (signedOut) {
      return const Left(ApplicationError.signedOut(message: 'Not signed in.'));
    }
    final failed = failure;
    if (change && failed != null) {
      return Left(failed);
    }
    return null;
  }

  Either<ApplicationError, BookShareInfo> _changed(
    String notebookId,
    BookShareInfo Function(BookShareInfo info) change,
  ) {
    final refused = _refused<BookShareInfo>(change: true);
    if (refused != null) {
      return refused;
    }
    final current = shares[notebookId];
    if (current == null) {
      return const Left(
        ApplicationError.invalid(message: "The notebook isn't shared."),
      );
    }
    return Right(shares[notebookId] = change(current));
  }

  BookShareInfo _members(BookShareInfo info, List<BookMemberRow> members) =>
      BookShareInfo(
        shareId: info.shareId,
        notebookId: info.notebookId,
        name: info.name,
        myRole: info.myRole,
        owner: info.owner,
        members: members,
        inviteRoles: info.inviteRoles,
        owned: info.owned,
      );

  @override
  Future<Either<ApplicationError, BookShareCheck>> canShare(
    String notebookId,
  ) async {
    calls.add('canShare $notebookId');
    final refused = _refused<BookShareCheck>();
    if (refused != null) {
      return refused;
    }
    final reason = unshareable[notebookId];
    if (reason != null) {
      return Left(ApplicationError.invalid(message: reason));
    }
    return Right(
      checks[notebookId] ??
          const BookShareCheck(
            name: 'Studio',
            notes: 3,
            locked: 0,
            lockedName: 'Studio (locked)',
            leftOut: <String>[],
          ),
    );
  }

  @override
  Future<Either<ApplicationError, BookShareInfo>> share(
    String notebookId,
    List<(String, BookRole)> invites,
  ) async {
    calls.add(
      'share $notebookId '
      '${invites.map((i) => '${i.$1} ${i.$2.name}').join(', ')}',
    );
    final refused = _refused<BookShareInfo>(change: true);
    if (refused != null) {
      return refused;
    }
    for (final (done, total) in shareProgress) {
      progressController.add(
        BookShareProgress(notebookId: notebookId, done: done, total: total),
      );
    }
    await shareGate?.future;
    final name = checks[notebookId]?.name ?? 'Studio';
    final info = bookInfo(
      notebookId,
      name: name,
      members: <BookMemberRow>[
        bookMember('ngoni', role: BookRole.owner, me: true),
        for (final (username, role) in invites)
          bookMember(
            username,
            role: role,
            standing: BookStandingRow.invited,
            roles: _ownerRoles,
            removable: true,
          ),
      ],
    );
    shares[notebookId] = info;
    return Right(info);
  }

  static const List<BookRole> _ownerRoles = <BookRole>[
    BookRole.manager,
    BookRole.editor,
    BookRole.commenter,
    BookRole.viewer,
  ];

  @override
  Stream<BookShareProgress> progress() => progressController.stream;

  @override
  Future<Either<ApplicationError, BookShareInfo?>> info(
    String notebookId,
  ) async {
    calls.add('info $notebookId');
    final refused = _refused<BookShareInfo?>();
    if (refused != null) {
      return refused;
    }
    return Right(shares[notebookId]);
  }

  @override
  Future<Either<ApplicationError, BookShareInfo>> invite(
    String notebookId,
    String username,
    BookRole role,
  ) async {
    calls.add('invite $notebookId $username ${role.name}');
    return _changed(
      notebookId,
      (info) => _members(info, <BookMemberRow>[
        ...info.members,
        bookMember(
          username,
          role: role,
          standing: BookStandingRow.invited,
          roles: info.owned ? _ownerRoles : const <BookRole>[],
          removable: info.owned,
        ),
      ]),
    );
  }

  @override
  Future<Either<ApplicationError, BookShareInfo>> setRole(
    String notebookId,
    String accountId,
    BookRole role,
  ) async {
    calls.add('setRole $notebookId $accountId ${role.name}');
    return _changed(
      notebookId,
      (info) => _members(info, <BookMemberRow>[
        for (final m in info.members)
          m.accountId == accountId
              ? BookMemberRow(
                  accountId: m.accountId,
                  username: m.username,
                  role: role,
                  me: m.me,
                  standing: m.standing,
                  fingerprint: m.fingerprint,
                  verified: m.verified,
                  roles: m.roles,
                  removable: m.removable,
                )
              : m,
      ]),
    );
  }

  @override
  Future<Either<ApplicationError, BookShareInfo>> remove(
    String notebookId,
    String accountId,
  ) async {
    calls.add('remove $notebookId $accountId');
    return _changed(
      notebookId,
      (info) => _members(info, <BookMemberRow>[
        for (final m in info.members)
          if (m.accountId != accountId) m,
      ]),
    );
  }

  Either<ApplicationError, BookEndedRow> _end(
    String notebookId,
    String message,
  ) {
    final refused = _refused<BookEndedRow>(change: true);
    if (refused != null) {
      return refused;
    }
    final info = shares.remove(notebookId);
    final name = info?.name ?? 'Studio';
    return Right(
      BookEndedRow(
        notebookId: notebookId,
        copyName: '$name (no longer shared)',
        message: message.replaceAll('{name}', name),
      ),
    );
  }

  @override
  Future<Either<ApplicationError, BookEndedRow>> leave(
    String notebookId,
  ) async {
    calls.add('leave $notebookId');
    return _end(
      notebookId,
      'You left "{name}". Your copy is in "{name} (no longer shared)".',
    );
  }

  @override
  Future<Either<ApplicationError, BookEndedRow>> stop(String notebookId) async {
    calls.add('stop $notebookId');
    return _end(
      notebookId,
      '"{name}" isn\'t shared any more. Everyone keeps a private copy.',
    );
  }

  @override
  BookWords words(String name, String who) => BookWords(
    removeTitle: 'Remove $who from "$name"?',
    removeBody:
        '$who keeps the notes they already have on their devices but sees '
        "nothing new. Kanso changes the notebook's key now.",
    leaveTitle: 'Leave "$name"? You\'ll keep a private copy.',
    stopTitle:
        'Stop sharing "$name"? Everyone keeps a private copy of its notes '
        'as they are now.',
  );

  @override
  String? updateLine() => updateLineText;

  @override
  Future<Either<ApplicationError, List<BookInvitationRow>>>
  invitations() async {
    final refused = _refused<List<BookInvitationRow>>();
    if (refused != null) {
      return refused;
    }
    return Right(List<BookInvitationRow>.of(invitationList));
  }

  @override
  Future<Either<ApplicationError, String>> accept(String shareId) async {
    calls.add('accept $shareId');
    final refused = _refused<String>(change: true);
    if (refused != null) {
      return refused;
    }
    invitationList = <BookInvitationRow>[
      for (final row in invitationList)
        if (row.shareId != shareId) row,
    ];
    return Right('book:$shareId');
  }

  @override
  Future<Either<ApplicationError, Unit>> decline(String shareId) async {
    calls.add('decline $shareId');
    final refused = _refused<Unit>(change: true);
    if (refused != null) {
      return refused;
    }
    invitationList = <BookInvitationRow>[
      for (final row in invitationList)
        if (row.shareId != shareId) row,
    ];
    return const Right(unit);
  }

  @override
  Future<Either<ApplicationError, List<SharedBookRow>>> sharedBooks() async {
    final refused = _refused<List<SharedBookRow>>();
    if (refused != null) {
      return refused;
    }
    return Right(List<SharedBookRow>.of(books));
  }

  @override
  Future<Either<ApplicationError, List<NoteEditRow>>> edits(
    String notebookId,
  ) async => Right(editsOf[notebookId] ?? const <NoteEditRow>[]);

  @override
  Future<Either<ApplicationError, BookSyncStatus?>> status(
    String notebookId,
  ) async => Right(statuses[notebookId]);

  Future<void> close() => progressController.close();
}

/// Registers [books] (a new one when null) and the shared-notebook blocs
/// over it and [shared] in the app's GetIt, as the app does, until the test
/// ends. Answers with the repository.
FixtureSharedNotebooksRepository registerSharedNotebooks({
  FixtureSharedNotebooksRepository? books,
  SharedNotesRepository? shared,
}) {
  final repository = books ?? FixtureSharedNotebooksRepository();
  final notes = shared ?? FixtureSharedNotesRepository();
  void swap<T extends Object>(T Function() make, {bool factory = true}) {
    if (getIt.isRegistered<T>()) {
      getIt.unregister<T>();
    }
    if (factory) {
      getIt.registerFactory<T>(make);
    } else {
      getIt.registerSingleton<T>(make());
    }
  }

  swap<SharedNotebooksRepository>(() => repository, factory: false);
  swap<SharedNotebooksBloc>(() => SharedNotebooksBloc(repository, notes));
  swap<NotebookSharingBloc>(() => NotebookSharingBloc(repository, notes));
  swap<ShareNotebookBloc>(() => ShareNotebookBloc(repository, notes));
  return repository;
}
