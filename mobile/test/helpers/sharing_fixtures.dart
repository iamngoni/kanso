// Kanso — sharing_fixtures.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:kanso/core/bridge/api/types.dart';

/// A person in a shared note, for tests.
MemberRow memberRow(
  String username, {
  ShareRole role = ShareRole.editor,
  bool me = false,
  String? fingerprint,
  bool keyChanged = false,
  bool invited = false,
  bool left = false,
  bool verified = false,
}) {
  return MemberRow(
    accountId: 'a_$username',
    username: username,
    role: role,
    me: me,
    fingerprint: fingerprint ?? '12345 67890 24680 13579',
    keyChanged: keyChanged,
    invited: invited,
    left: left,
    verified: verified,
  );
}

/// A person looked up, for tests.
PersonRow personRow(
  String username, {
  String fingerprint = '11111 22222 33333 44444',
  bool verified = false,
}) {
  return PersonRow(
    accountId: 'a_$username',
    username: username,
    fingerprint: fingerprint,
    verified: verified,
  );
}

/// The sharing of the note [noteId], owned by [owner] (this account unless
/// [myRole] says another).
ShareInfo shareInfo(
  String noteId, {
  String owner = 'ngoni',
  ShareRole myRole = ShareRole.owner,
  List<MemberRow>? members,
  List<ShareLinkRow> links = const <ShareLinkRow>[],
  List<MemberRow> requests = const <MemberRow>[],
}) {
  return ShareInfo(
    shareId: 's_$noteId',
    noteId: noteId,
    myRole: myRole,
    owner: owner,
    members:
        members ??
        <MemberRow>[
          memberRow(
            owner,
            role: ShareRole.owner,
            me: myRole == ShareRole.owner,
            verified: true,
          ),
        ],
    links: links,
    requests: requests,
  );
}

/// An invitation, for tests.
InvitationRow invitationRow(
  String shareId, {
  String from = 'ana',
  ShareRole role = ShareRole.editor,
  int atMs = 1790000000000,
}) {
  return InvitationRow(shareId: shareId, from: from, role: role, atMs: atMs);
}

/// A comment [id] by [author] saying [body], on the words [quote] at
/// [start]..[end], or orphaned there when [orphaned].
CommentRow commentRow(
  String id, {
  String author = 'bo',
  String body = 'Why this?',
  String quote = 'Alpha',
  int start = 0,
  int end = 5,
  bool orphaned = false,
  bool resolved = false,
  bool mine = false,
  bool mayComment = true,
  int createdMs = 0,
  int? editedMs,
  bool pending = false,
  List<MentionSpan> mentions = const <MentionSpan>[],
}) => CommentRow(
  id: id,
  author: 'a_$author',
  authorName: author,
  body: body,
  createdMs: createdMs,
  editedMs: editedMs,
  resolved: resolved,
  mine: mine,
  pending: pending,
  canEdit: mayComment && mine,
  canDelete: mayComment && mine,
  canResolve: mayComment,
  quote: quote,
  placement: orphaned
      ? CommentPlacement.orphaned(at: start)
      : CommentPlacement.anchored(start: start, end: end),
  mentions: mentions,
);
