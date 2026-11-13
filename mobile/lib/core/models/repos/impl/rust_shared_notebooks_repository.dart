// Kanso — rust_shared_notebooks_repository.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/shared.dart';
import '../../../bridge/api/types.dart';
import '../../../utils/cancelable_stream.dart';
import '../../../utils/kanso_error_mapping.dart';
import '../../data/application_error.dart';
import '../abs/shared_notebooks_repository.dart';

/// [SharedNotebooksRepository] over the Rust core's shared notebooks.
@LazySingleton(as: SharedNotebooksRepository)
class RustSharedNotebooksRepository implements SharedNotebooksRepository {
  RustSharedNotebooksRepository(this._shared);

  final KansoShared _shared;

  @override
  Future<Either<ApplicationError, BookShareCheck>> canShare(
    String notebookId,
  ) => attempt(() => _shared.canShareBook(notebookId: notebookId));

  @override
  Future<Either<ApplicationError, BookShareInfo>> share(
    String notebookId,
    List<(String, BookRole)> invites,
  ) => attempt(
    () => _shared.shareBook(notebookId: notebookId, invites: invites),
  );

  @override
  Stream<BookShareProgress> progress() =>
      cancelableStream(_shared.bookProgress());

  @override
  Future<Either<ApplicationError, BookShareInfo?>> info(String notebookId) =>
      attempt(() => _shared.bookInfo(notebookId: notebookId));

  @override
  Future<Either<ApplicationError, BookShareInfo>> invite(
    String notebookId,
    String username,
    BookRole role,
  ) => attempt(
    () => _shared.inviteToBook(
      notebookId: notebookId,
      username: username,
      role: role,
    ),
  );

  @override
  Future<Either<ApplicationError, BookShareInfo>> setRole(
    String notebookId,
    String accountId,
    BookRole role,
  ) => attempt(
    () => _shared.setBookRole(
      notebookId: notebookId,
      accountId: accountId,
      role: role,
    ),
  );

  @override
  Future<Either<ApplicationError, BookShareInfo>> remove(
    String notebookId,
    String accountId,
  ) => attempt(
    () => _shared.removeFromBook(notebookId: notebookId, accountId: accountId),
  );

  @override
  Future<Either<ApplicationError, BookEndedRow>> leave(String notebookId) =>
      attempt(() => _shared.leaveBook(notebookId: notebookId));

  @override
  Future<Either<ApplicationError, BookEndedRow>> stop(String notebookId) =>
      attempt(() => _shared.stopBook(notebookId: notebookId));

  @override
  BookWords words(String name, String who) =>
      _shared.bookWords(name: name, who: who);

  @override
  Future<Either<ApplicationError, List<BookInvitationRow>>> invitations() =>
      attempt(_shared.bookInvitations);

  @override
  Future<Either<ApplicationError, String>> accept(String shareId) =>
      attempt(() => _shared.acceptBook(shareId: shareId));

  @override
  Future<Either<ApplicationError, Unit>> decline(String shareId) =>
      attempt(() async {
        await _shared.declineBook(shareId: shareId);
        return unit;
      });

  @override
  Future<Either<ApplicationError, List<SharedBookRow>>> sharedBooks() =>
      attempt(_shared.sharedBooks);

  @override
  Future<Either<ApplicationError, List<NoteEditRow>>> edits(
    String notebookId,
  ) => attempt(() => _shared.bookEdits(notebookId: notebookId));

  @override
  Future<Either<ApplicationError, BookSyncStatus?>> status(String notebookId) =>
      attempt(() => _shared.bookStatus(notebookId: notebookId));
}
