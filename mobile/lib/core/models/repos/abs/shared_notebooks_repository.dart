// Kanso — shared_notebooks_repository.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../bridge/api/types.dart';
import '../../data/application_error.dart';

/// Shared notebooks: sharing one, its people and their roles, invitations,
/// leaving and stopping, who last edited each note and whether it is up to
/// date. The keys, the signed member list and every rule of who may do what
/// stay in the Rust core: the rows say which roles each member's menu
/// offers, and the shell only shows them.
abstract interface class SharedNotebooksRepository {
  /// What sharing the notebook would do; the failure says why it can't be
  /// shared (locked, already shared).
  Future<Either<ApplicationError, BookShareCheck>> canShare(String notebookId);

  /// Shares the notebook with [invites] (a username and a role each) and
  /// answers with its sharing. [progress] hears how far it has got.
  Future<Either<ApplicationError, BookShareInfo>> share(
    String notebookId,
    List<(String, BookRole)> invites,
  );

  /// How far a notebook being shared has got.
  Stream<BookShareProgress> progress();

  /// The notebook's sharing, or null when it isn't shared.
  Future<Either<ApplicationError, BookShareInfo?>> info(String notebookId);

  Future<Either<ApplicationError, BookShareInfo>> invite(
    String notebookId,
    String username,
    BookRole role,
  );

  /// Gives a member another role; a lower one changes the notebook's key.
  Future<Either<ApplicationError, BookShareInfo>> setRole(
    String notebookId,
    String accountId,
    BookRole role,
  );

  /// Removes a member, or cancels an invitation.
  Future<Either<ApplicationError, BookShareInfo>> remove(
    String notebookId,
    String accountId,
  );

  /// Leaves a notebook someone else shared, keeping a private copy.
  Future<Either<ApplicationError, BookEndedRow>> leave(String notebookId);

  /// Stops sharing a notebook this account owns.
  Future<Either<ApplicationError, BookEndedRow>> stop(String notebookId);

  /// The words of the notebook's questions, the same as the desktop's.
  BookWords words(String name, String who);

  /// Notebooks shared with this account that this device hasn't joined.
  Future<Either<ApplicationError, List<BookInvitationRow>>> invitations();

  /// Joins the notebook; answers with its library ID.
  Future<Either<ApplicationError, String>> accept(String shareId);

  Future<Either<ApplicationError, Unit>> decline(String shareId);

  /// The notebooks shared here, for the library's Shared section.
  Future<Either<ApplicationError, List<SharedBookRow>>> sharedBooks();

  /// Who last changed each note of the notebook.
  Future<Either<ApplicationError, List<NoteEditRow>>> edits(String notebookId);

  /// Whether the notebook is up to date; null when it isn't shared.
  Future<Either<ApplicationError, BookSyncStatus?>> status(String notebookId);
}
