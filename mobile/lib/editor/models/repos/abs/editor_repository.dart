// Kanso — editor_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/bridge/api/types.dart';
import '../../../../core/models/data/application_error.dart';

/// Notes open for editing, in the Rust core.
// One member, but a repository like the others: more are for the screens
// that follow (the note's place, new notes).
// ignore: one_member_abstracts
abstract interface class EditorRepository {
  /// Opens the note [noteId] for editing. A [LockedError] while the note is
  /// locked and closed, a [NotFoundError] when there is no such note.
  Future<Either<ApplicationError, EditorDocument>> open(String noteId);
}

/// One note open for editing. The text is in Rust; the field that draws it
/// hands over every change, and asks for the rest.
///
/// What a keystroke costs matters here, so the calls the field makes on every
/// change ([apply], [select], [command], [undo], [redo], [setTitle]) are
/// synchronous: they run on the UI isolate in microseconds, and a command's
/// change to the text and the field's own happen in the same turn, with no
/// keystroke between them. Everything else is a [Future].
///
/// Offsets are UTF-16 units of the visible text, which does not include the
/// note's front matter. Once the note is locked and its lock closes, or the
/// document is closed, every call is a [LockedError] (or an error that says
/// it is closed); nothing of a locked note's text is kept here.
abstract interface class EditorDocument {
  /// The note as it is now.
  Future<Either<ApplicationError, EditorSnapshot>> snapshot();

  /// The field's text changed. An [InvalidError] when [change] was made
  /// against an older version of the text than Rust has (ask for the
  /// [snapshot]) or names units outside it.
  Either<ApplicationError, EditOutcome> apply(TextChange change);

  /// The field's selection changed; what is under the caret.
  Either<ApplicationError, EditorContext> select(
    Range16 selection, {
    required bool composing,
  });

  /// A change the editor decides on (a newline in a list, a format, a
  /// completion chosen); null when there is nothing for it to do, and the
  /// field types what it was going to.
  Either<ApplicationError, EditOutcome?> command(EditCommand command);

  /// Undoes the last step; null when there is none.
  Either<ApplicationError, EditOutcome?> undo();

  /// Redoes the last undone step; null when there is none.
  Either<ApplicationError, EditOutcome?> redo();

  /// Stores [file], an image or a PDF, and puts it in the text over the
  /// selection as a paragraph of its own. An [InvalidError] that says why
  /// for a locked note or a file that is neither.
  Future<Either<ApplicationError, EditOutcome>> attach(ShareFile file);

  /// The title changed.
  Either<ApplicationError, Unit> setTitle(String title);

  /// The colours of [start] to [end] of the text in the built-in theme
  /// [themeId], as of the version in the batch.
  Future<Either<ApplicationError, RunBatch>> styledRuns({
    required int start,
    required int end,
    required String themeId,
  });

  /// What the caret completes (`[[` with titles, `#` with tags); null when
  /// it completes nothing. Titles only, never a note's text.
  Future<Either<ApplicationError, Completion?>> completion();

  /// Where the link at [offset] of the text leads, if there is one.
  Future<Either<ApplicationError, LinkTarget?>> linkAt(int offset);

  /// Sets a property: the value of the one in row [index], or, without,
  /// the property called [key] (added when the note has none). The
  /// properties come back; the text does not change.
  Future<Either<ApplicationError, List<Property>>> setProperty({
    required String key,
    required String value,
    int? index,
  });

  /// Removes the property in row [index].
  Future<Either<ApplicationError, List<Property>>> removeProperty(int index);

  /// Saves now. With a change made elsewhere waiting for an answer this
  /// keeps the typed text, as leaving the note does.
  Future<Either<ApplicationError, SaveState>> flush();

  /// Keeps the typed text over a change made elsewhere; theirs stays in the
  /// note's history.
  Future<Either<ApplicationError, SaveState>> keepMine();

  /// Takes a change made elsewhere; the typed text goes to the note's
  /// history. A [NotFoundError] for a note deleted elsewhere: it is let go.
  Future<Either<ApplicationError, EditorSnapshot>> useTheirs();

  /// Saves, and ends the session.
  Future<Either<ApplicationError, SaveState>> close();

  /// What happens to the note that was not asked for: saved, changed
  /// elsewhere, reloaded, closed. Single-subscription.
  Stream<EditorEvent> events();

  /// Comments on the selected words of a shared note, the desktop's Comment
  /// on Selection; the typing goes into the room first. Answers with the new
  /// comment's ID, or why not (no words selected, not shared, may only
  /// read).
  Future<Either<ApplicationError, String>> commentOnSelection(String body);

  /// The words the note's open comments are on, in UTF-16 units of the
  /// note's text, to tint. None for a note that isn't shared.
  Future<Either<ApplicationError, List<Range16>>> commentMarks();

  /// Selects the words comment [commentId] is on; the selection, or null
  /// when there is no such comment.
  Future<Either<ApplicationError, Range16?>> selectComment(String commentId);

  /// Other people's carets and selections in the note, in UTF-16 units of
  /// the text, with their colours; none for a note that isn't open live.
  Future<Either<ApplicationError, List<CursorMark>>> remoteCursors();
}
