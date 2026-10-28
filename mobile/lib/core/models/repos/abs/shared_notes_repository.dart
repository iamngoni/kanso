// Kanso — shared_notes_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../bridge/api/types.dart';
import '../../data/application_error.dart';

/// Shared notes: who a note is shared with, the invitations and links, and
/// the live room of an open shared note. The keys and the sealed text stay
/// in the Rust core; a locked note is refused ([LockedError]) before
/// anything else.
abstract interface class SharedNotesRepository {
  /// Whether the note [noteId] can be shared; the failure says why not: it
  /// is locked ([LockedError]), in the trash, or shared already.
  Future<Either<ApplicationError, Unit>> canShare(String noteId);

  /// Shares the note with [invites] (a username and a role each, never
  /// `owner`) and answers with its sharing.
  Future<Either<ApplicationError, ShareInfo>> share(
    String noteId,
    List<(String, ShareRole)> invites,
  );

  /// The note's sharing, or null when it is not shared.
  Future<Either<ApplicationError, ShareInfo?>> info(String noteId);

  Future<Either<ApplicationError, ShareInfo>> invite(
    String noteId,
    String username,
    ShareRole role,
  );

  Future<Either<ApplicationError, ShareInfo>> setRole(
    String noteId,
    String accountId,
    ShareRole role,
  );

  /// Removes a member, or cancels an invitation.
  Future<Either<ApplicationError, ShareInfo>> remove(
    String noteId,
    String accountId,
  );

  Future<Either<ApplicationError, ShareInfo>> approveRequest(
    String noteId,
    String accountId,
  );

  Future<Either<ApplicationError, ShareInfo>> declineRequest(
    String noteId,
    String accountId,
  );

  /// An invite link of [days] days (1 to 30) with [role]: its address,
  /// which is known only to this call.
  Future<Either<ApplicationError, String>> createLink(
    String noteId,
    ShareRole role,
    int days,
  );

  Future<Either<ApplicationError, ShareInfo>> cancelLink(
    String noteId,
    String linkId,
  );

  /// Asks to join the note an invite link is for; its owner lets the person
  /// in.
  Future<Either<ApplicationError, JoinRow>> join(String link);

  /// Stops sharing a note this account owns.
  Future<Either<ApplicationError, Unit>> stop(String noteId);

  /// Leaves a note someone else shared.
  Future<Either<ApplicationError, Unit>> leave(String noteId);

  /// The notes shared with this account that wait to be accepted.
  Future<Either<ApplicationError, List<InvitationRow>>> invitations();

  /// Accepts an invitation; the note's id.
  Future<Either<ApplicationError, String>> accept(String shareId);

  Future<Either<ApplicationError, Unit>> decline(String shareId);

  /// Looks someone up by username, before sharing: their pinned keys, or
  /// the keys the server gives instead of them.
  Future<Either<ApplicationError, PersonLookup>> lookupPerson(String username);

  /// Marks someone's fingerprint as compared and matching.
  Future<Either<ApplicationError, Unit>> verifyPerson(String accountId);

  /// Pins the new keys of [username] after the person compared
  /// [fingerprint].
  Future<Either<ApplicationError, PersonRow>> acceptNewKey(
    String username,
    String fingerprint,
  );

  /// Syncs shared notes now.
  Future<Either<ApplicationError, SharedEvent>> syncShared();

  /// Opens the live room of the shared note [noteId] for [holder], whatever
  /// has the note open. Opening it again for the same holder (after the app
  /// came back to the front) is fine.
  Future<Either<ApplicationError, Unit>> openLive(String noteId, Object holder);

  /// Lets go of the room of [noteId] for [holder]. The room is left when the
  /// last holder lets go: the library has one room per note, which two open
  /// copies of the note share.
  Future<Either<ApplicationError, Unit>> closeLive(
    String noteId,
    Object holder,
  );

  /// What each sync of shared notes finds that is news.
  Stream<SharedEvent> events();

  /// The comments on the shared note [noteId], those on words still in the
  /// text in text order, then the ones whose words were deleted; a note
  /// that isn't shared is refused.
  Future<Either<ApplicationError, List<CommentRow>>> comments(String noteId);

  /// "2 open · 1 resolved", empty with none.
  Future<Either<ApplicationError, String>> commentsBand(String noteId);

  /// Changes the words of one of this account's comments.
  Future<Either<ApplicationError, Unit>> editComment(
    String noteId,
    String commentId,
    String body,
  );

  /// Deletes one of this account's comments, for everyone in the note.
  Future<Either<ApplicationError, Unit>> deleteComment(
    String noteId,
    String commentId,
  );

  /// Resolves a comment, or opens it again.
  Future<Either<ApplicationError, Unit>> resolveComment(
    String noteId,
    String commentId, {
    required bool resolved,
  });

  /// What the page of shared note [noteId] says of its sharing: its role,
  /// whether its room is live, whether it ended, how near its size limit it
  /// is and what couldn't be done for it. Null for a note that isn't
  /// shared.
  Future<Either<ApplicationError, LiveStatus?>> liveStatus(String noteId);

  /// Forgets what couldn't be done for shared note [noteId].
  Future<Either<ApplicationError, Unit>> dismissProblems(String noteId);
}
