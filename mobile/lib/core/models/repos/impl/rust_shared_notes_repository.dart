// Kanso — rust_shared_notes_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/shared.dart';
import '../../../bridge/api/types.dart';
import '../../../utils/cancelable_stream.dart';
import '../../../utils/kanso_error_mapping.dart';
import '../../../utils/live_rooms.dart';
import '../../data/application_error.dart';
import '../abs/shared_notes_repository.dart';

/// [SharedNotesRepository] over the Rust core's shared notes.
@LazySingleton(as: SharedNotesRepository)
class RustSharedNotesRepository implements SharedNotesRepository {
  RustSharedNotesRepository(this._shared)
    : _rooms = LiveRooms(
        (noteId) => _shared.openLive(noteId: noteId),
        (noteId) => _shared.closeLive(noteId: noteId),
      );

  final KansoShared _shared;

  /// Who has each note's live room open. The library's room has no count of
  /// its own, so it is left only when the last holder lets go.
  final LiveRooms _rooms;

  @override
  Future<Either<ApplicationError, Unit>> canShare(String noteId) =>
      attempt(() async {
        await _shared.canShare(noteId: noteId);
        return unit;
      });

  @override
  Future<Either<ApplicationError, ShareInfo>> share(
    String noteId,
    List<(String, ShareRole)> invites,
  ) => attempt(() => _shared.share(noteId: noteId, invites: invites));

  @override
  Future<Either<ApplicationError, ShareInfo?>> info(String noteId) =>
      attempt(() => _shared.info(noteId: noteId));

  @override
  Future<Either<ApplicationError, ShareInfo>> invite(
    String noteId,
    String username,
    ShareRole role,
  ) => attempt(
    () => _shared.invite(noteId: noteId, username: username, role: role),
  );

  @override
  Future<Either<ApplicationError, ShareInfo>> setRole(
    String noteId,
    String accountId,
    ShareRole role,
  ) => attempt(
    () => _shared.setRole(noteId: noteId, accountId: accountId, role: role),
  );

  @override
  Future<Either<ApplicationError, ShareInfo>> remove(
    String noteId,
    String accountId,
  ) => attempt(() => _shared.remove(noteId: noteId, accountId: accountId));

  @override
  Future<Either<ApplicationError, ShareInfo>> approveRequest(
    String noteId,
    String accountId,
  ) => attempt(
    () => _shared.approveRequest(noteId: noteId, accountId: accountId),
  );

  @override
  Future<Either<ApplicationError, ShareInfo>> declineRequest(
    String noteId,
    String accountId,
  ) => attempt(
    () => _shared.declineRequest(noteId: noteId, accountId: accountId),
  );

  @override
  Future<Either<ApplicationError, String>> createLink(
    String noteId,
    ShareRole role,
    int days,
  ) =>
      attempt(() => _shared.createLink(noteId: noteId, role: role, days: days));

  @override
  Future<Either<ApplicationError, ShareInfo>> cancelLink(
    String noteId,
    String linkId,
  ) => attempt(() => _shared.cancelLink(noteId: noteId, linkId: linkId));

  @override
  Future<Either<ApplicationError, JoinRow>> join(String link) =>
      attempt(() => _shared.join(link: link));

  @override
  Future<Either<ApplicationError, Unit>> stop(String noteId) =>
      attempt(() async {
        await _shared.stop(noteId: noteId);
        return unit;
      });

  @override
  Future<Either<ApplicationError, Unit>> leave(String noteId) =>
      attempt(() async {
        await _shared.leave(noteId: noteId);
        return unit;
      });

  @override
  Future<Either<ApplicationError, List<InvitationRow>>> invitations() =>
      attempt(_shared.invitations);

  @override
  Future<Either<ApplicationError, String>> accept(String shareId) =>
      attempt(() => _shared.accept(shareId: shareId));

  @override
  Future<Either<ApplicationError, Unit>> decline(String shareId) =>
      attempt(() async {
        await _shared.decline(shareId: shareId);
        return unit;
      });

  @override
  Future<Either<ApplicationError, PersonLookup>> lookupPerson(
    String username,
  ) => attempt(() => _shared.lookupPerson(username: username));

  @override
  Future<Either<ApplicationError, Unit>> verifyPerson(String accountId) =>
      attempt(() async {
        await _shared.verifyPerson(accountId: accountId);
        return unit;
      });

  @override
  Future<Either<ApplicationError, PersonRow>> acceptNewKey(
    String username,
    String fingerprint,
  ) => attempt(
    () => _shared.acceptNewKey(username: username, fingerprint: fingerprint),
  );

  @override
  Future<Either<ApplicationError, SharedEvent>> syncShared() =>
      attempt(_shared.syncShared);

  @override
  Future<Either<ApplicationError, Unit>> openLive(
    String noteId,
    Object holder,
  ) => attempt(() async {
    await _rooms.take(noteId, holder);
    return unit;
  });

  @override
  Future<Either<ApplicationError, Unit>> closeLive(
    String noteId,
    Object holder,
  ) => attempt(() async {
    await _rooms.letGo(noteId, holder);
    return unit;
  });

  @override
  Stream<SharedEvent> events() => cancelableStream(_shared.events());

  @override
  Future<Either<ApplicationError, List<CommentRow>>> comments(String noteId) =>
      attempt(() => _shared.comments(noteId: noteId));

  @override
  Future<Either<ApplicationError, String>> commentsBand(String noteId) =>
      attempt(() => _shared.commentsBand(noteId: noteId));

  @override
  Future<Either<ApplicationError, Unit>> editComment(
    String noteId,
    String commentId,
    String body,
  ) => attempt(() async {
    await _shared.editComment(noteId: noteId, commentId: commentId, body: body);
    return unit;
  });

  @override
  Future<Either<ApplicationError, Unit>> deleteComment(
    String noteId,
    String commentId,
  ) => attempt(() async {
    await _shared.deleteComment(noteId: noteId, commentId: commentId);
    return unit;
  });

  @override
  Future<Either<ApplicationError, Unit>> resolveComment(
    String noteId,
    String commentId, {
    required bool resolved,
  }) => attempt(() async {
    await _shared.resolveComment(
      noteId: noteId,
      commentId: commentId,
      resolved: resolved,
    );
    return unit;
  });

  @override
  Future<Either<ApplicationError, LiveStatus?>> liveStatus(String noteId) =>
      attempt(() => _shared.liveStatus(noteId: noteId));

  @override
  Future<Either<ApplicationError, Unit>> dismissProblems(String noteId) =>
      attempt(() async {
        await _shared.dismissProblems(noteId: noteId);
        return unit;
      });
}
