// Kanso — share_labels.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../core/bridge/api/types.dart';
import '../../gen/l10n/app_localizations.dart';
import '../models/data/share_done.dart';
import '../models/data/share_problem.dart';
import 'member_status.dart';

/// The words of sharing, in the desktop's: roles, what a step did and why
/// one did not happen, and the notes under a person's name.
extension ShareLabels on AppLocalizations {
  /// "Can edit": what [role] may do.
  String shareRoleName(ShareRole role) {
    return switch (role) {
      ShareRole.owner => shareRoleOwner,
      ShareRole.editor => shareRoleEditor,
      ShareRole.commenter => shareRoleCommenter,
      ShareRole.viewer => shareRoleViewer,
    };
  }

  /// What [role] can do with the note, to finish "You …".
  String shareAccess(ShareRole role) {
    return switch (role) {
      ShareRole.viewer => shareAccessView,
      ShareRole.commenter => shareAccessComment,
      ShareRole.editor || ShareRole.owner => shareAccessEdit,
    };
  }

  /// Why a step of sharing did not happen.
  String shareProblemText(ShareProblem problem) {
    return switch (problem) {
      ShareProblemWords(:final message) => message,
      ShareProblemNameMissing() => shareProblemNameMissing,
      ShareProblemLinkMissing() => shareProblemLinkMissing,
      ShareProblemOffline() => shareProblemOffline,
      ShareProblemSignedOut() => shareProblemSignedOut,
      ShareProblemUnexpected() => shareProblemUnexpected,
    };
  }

  /// What the last step of the share sheet did.
  String shareDoneText(ShareDone done) {
    return switch (done) {
      ShareDoneInvited(:final username) => shareInvited(username),
      ShareDoneStopped() => shareStopped,
      ShareDoneLeft() => shareLeft,
    };
  }

  /// The words under a person's name.
  String shareMemberNoteText(MemberNote note) {
    return switch (note) {
      MemberNote.declined => shareMemberDeclined,
      MemberNote.left => shareMemberLeft,
      MemberNote.invited => shareMemberInvited,
      MemberNote.you => shareMemberYou,
      MemberNote.unchecked => shareMemberUnchecked,
      MemberNote.verified => shareMemberVerified,
    };
  }

  /// "Can edit, for 6 more days", or "Can edit, expired".
  String shareLinkText(ShareRole role, int daysLeft) {
    final name = shareRoleName(role);
    return daysLeft <= 0
        ? shareLinkExpired(name)
        : shareLinkDaysLeft(name, daysLeft);
  }

  /// A shared note's status line, in the desktop footer's words: "Shared ·
  /// Can view · Read only · Live", and the size limit when it is near; a
  /// note whose sharing ended is a private copy.
  String sharedStatusLine(LiveStatus status) {
    if (status.ended) {
      return sharedStatusPrivateCopy;
    }
    final canEdit =
        status.role == ShareRole.owner || status.role == ShareRole.editor;
    return <String>[
      sharedStatusShared,
      shareRoleName(status.role),
      if (!canEdit) sharedStatusReadOnly,
      if (status.connected) sharedStatusLive else sharedStatusOffline,
      if (status.nearLimit) sharedStatusNearLimit,
    ].join(' · ');
  }

  /// What the banner over a shared note says of its [problems] (newest
  /// first): the newest one's words, and how many more there are. Null
  /// with none.
  String? sharedProblemsBanner(List<String> problems) {
    if (problems.isEmpty) {
      return null;
    }
    final newest = problems.first.trim();
    final more = problems.length - 1;
    return more == 0 ? newest : sharedProblemsMore(newest, more);
  }
}
