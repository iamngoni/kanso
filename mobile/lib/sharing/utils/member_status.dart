// Kanso — member_status.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../core/bridge/api/types.dart';

/// What is said under a person's name in a shared note.
enum MemberNote { declined, left, invited, you, unchecked, verified }

/// How a person's row reads.
enum MemberTone {
  normal,

  /// They left or declined: dimmed until the owner's next change drops them.
  gone,

  /// Their keys changed or couldn't be checked: a warning tint.
  unchecked,
}

/// The notes under [member]'s name and how the row reads: the desktop's
/// `member_status`.
///
/// Someone who left or declined says only that (declined if they never
/// accepted). Otherwise an invitation not accepted yet says so; then "You"
/// for the person's own row, else "Can't be checked" (and the warning tone)
/// when their keys changed, else "Fingerprint verified" when it was compared.
({List<MemberNote> notes, MemberTone tone}) memberStatus(MemberRow member) {
  if (member.left) {
    // Someone who never accepted declined; anyone else left.
    final gone = member.invited ? MemberNote.declined : MemberNote.left;
    return (notes: <MemberNote>[gone], tone: MemberTone.gone);
  }
  final notes = <MemberNote>[if (member.invited) MemberNote.invited];
  if (member.me) {
    notes.add(MemberNote.you);
  } else if (member.keyChanged) {
    return (
      notes: <MemberNote>[...notes, MemberNote.unchecked],
      tone: MemberTone.unchecked,
    );
  } else if (member.verified) {
    notes.add(MemberNote.verified);
  }
  return (notes: notes, tone: MemberTone.normal);
}
