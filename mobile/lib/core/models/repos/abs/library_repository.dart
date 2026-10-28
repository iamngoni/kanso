// Kanso — library_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../bridge/api/types.dart';
import '../../data/application_error.dart';

/// The notes of the open library: what the Rust core knows about them.
///
/// A locked note's text, snippet, tags, status and properties never come back
/// while it is closed; its title and its lock do (they show in lists). A tag
/// place, a status narrowing and a tag search never list one. Streams do not
/// fail: they end when the library closes.
abstract interface class LibraryRepository {
  /// The library's default word goal for notes without their own choice.
  Future<Either<ApplicationError, int?>> defaultWordGoal();

  /// Sets the library's default word goal (or clears it with null).
  Future<Either<ApplicationError, int?>> setDefaultWordGoal(int? goal);

  /// A note's own choice, kept separate from the library default.
  Future<Either<ApplicationError, GoalSetting>> noteWordGoal(String id);

  /// Sets a note's goal, using its open editor session when one exists.
  Future<Either<ApplicationError, Unit>> setNoteWordGoal(
    String id,
    GoalSetting setting,
  );

  /// The notes [query] picks, in its order, with the layout the desktop
  /// gives that list (pinned first; Recent by edit time; headers by date).
  Future<Either<ApplicationError, NoteList>> list(ListQuery query);

  /// How many notes each place of the library lists.
  Future<Either<ApplicationError, LibraryOverview>> overview();

  /// Makes [change] to every note in [ids] in one write; how many notes it
  /// wrote, what the notice says and the ticket for [undo]. Unlocked notes
  /// into a locked notebook, and notes out of one, are a [LockedError]: those
  /// go one at a time.
  Future<Either<ApplicationError, Changed>> change(
    List<String> ids,
    NoteChange change,
  );

  /// Takes back the change kept under [ticket], as far as it still holds (a
  /// field changed again since keeps its newer value); how many notes it
  /// wrote. A ticket is used once.
  Future<Either<ApplicationError, int>> undo(int ticket);

  /// One note; its text is null while it is locked and closed.
  Future<Either<ApplicationError, NoteView>> note(String id);

  /// Saves [body] as the text of the note [id], which was at [baseRev].
  Future<Either<ApplicationError, Saved>> save({
    required String id,
    required String baseRev,
    required String body,
  });

  /// Makes a note.
  Future<Either<ApplicationError, Saved>> create(NewNote note);

  /// Moves the note to the trash.
  Future<Either<ApplicationError, Unit>> trash(String id);

  /// The notebooks in tree order, with their depth and role.
  Future<Either<ApplicationError, List<NotebookRow>>> notebooks();

  /// Makes a notebook, inside [parentId] if given.
  Future<Either<ApplicationError, NotebookRow>> createNotebook(
    String name, {
    String? parentId,
  });

  Future<Either<ApplicationError, List<TagRow>>> tags();

  /// Notes matching [text] in [scope], with where the words matched.
  Future<Either<ApplicationError, List<Hit>>> search(
    String text, {
    SearchScope scope = const SearchScope.all(),
    int limit = 50,
  });

  /// A note's front-matter properties; none for a closed locked note.
  Future<Either<ApplicationError, List<Property>>> properties(String id);

  /// The table a query view's [source] describes.
  Future<Either<ApplicationError, Table>> queryView(
    String source, {
    String? selfId,
  });

  /// The Tasks place: the tasks of the notes in [query], in groups (Overdue,
  /// Today, Upcoming, No date and, if asked, Completed), with the desktop's
  /// labels. A locked note's tasks are left out.
  Future<Either<ApplicationError, TaskBoard>> tasks(TaskListQuery query);

  /// How many open tasks are overdue: the badge of the Tasks place.
  Future<Either<ApplicationError, int>> overdueTasks();

  /// Ticks (or clears) the task on [line] of the note [noteId], whose text
  /// in the note is [text] (a task's `sourceText`), in one write. The result
  /// says whether the note changed (with the ticket for [undo]), already was
  /// so, or no longer has that task.
  Future<Either<ApplicationError, Ticked>> setTask({
    required String noteId,
    required int line,
    required String text,
    required bool checked,
  });

  /// Ticks (or clears) the [index]-th task box of the note [noteId], as the
  /// preview drawn at [baseRev] showed it, in one write. A
  /// [ConflictError] when the note has changed since.
  Future<Either<ApplicationError, Ticked>> toggleTaskAt({
    required String noteId,
    required int index,
    required String baseRev,
  });

  /// The note a wiki-link in the note [fromNote] to the title [title] asks
  /// for: made beside [fromNote], or the one with that title if there is one
  /// by now.
  Future<Either<ApplicationError, Saved>> createLinkedNote({
    required String title,
    required String fromNote,
  });

  /// An attachment's name, type and bytes, when a note that can be read now
  /// links it.
  Future<Either<ApplicationError, AttachmentFile>> attachment(String id);

  /// The IDs of what changed, in batches, as it is written: by this phone, a
  /// sync or the background.
  Stream<List<String>> changes();

  /// The statuses a note can have, and what the desktop calls them.
  Future<Either<ApplicationError, List<StatusChoice>>> statuses();

  /// Deletes every note in the trash for good; how many.
  Future<Either<ApplicationError, int>> emptyTrash();

  /// Copies note [id] as the desktop does: its text, notebook, tags and
  /// status, titled "(copy)".
  Future<Either<ApplicationError, Saved>> duplicate(String id);

  /// The wiki-link to note [id] to paste elsewhere, and what to say.
  Future<Either<ApplicationError, CopiedLink>> noteLink(
    String id, {
    required bool kanso,
  });

  /// The notebook rows that show with [folds] applied, around [showing].
  Future<Either<ApplicationError, List<VisibleNotebook>>> visibleNotebooks(
    FoldState folds, {
    String? showing,
  });

  /// The folds after a tap on notebook [id]'s arrow.
  Future<Either<ApplicationError, FoldState>> toggleFold(
    FoldState folds,
    String id, {
    String? showing,
  });

  /// The tag chips and the nested tag tree, folded as [openPaths] say.
  Future<Either<ApplicationError, TagPlaces>> tagPlaces({
    required bool all,
    required List<String> openPaths,
    String? selected,
    String? showing,
  });

  /// Renames notebook [id]; names needn't be unique.
  Future<Either<ApplicationError, Unit>> renameNotebook(String id, String name);

  /// Deletes notebook [id] and what is inside it.
  Future<Either<ApplicationError, Unit>> deleteNotebook(String id);

  /// Sets notebook [id]'s icon, or takes its own away with null.
  Future<Either<ApplicationError, Unit>> setNotebookIcon(
    String id,
    String? icon,
  );

  /// Sets notebook [id]'s colour (`#RRGGBB`), or takes its own away.
  Future<Either<ApplicationError, Unit>> setNotebookColor(
    String id,
    String? color,
  );

  /// Names a tag [name], making it.
  Future<Either<ApplicationError, TagRow>> createTag(String name);

  /// Renames tag [id]; names needn't be unique.
  Future<Either<ApplicationError, Unit>> renameTag(String id, String name);

  /// Deletes tag [id], and takes it off the notes that had it.
  Future<Either<ApplicationError, Unit>> deleteTag(String id);

  /// Sets tag [id]'s colour (`red`, `default`, ...).
  Future<Either<ApplicationError, Unit>> setTagColor(String id, String color);

  /// The notebook icons matching [query].
  Future<Either<ApplicationError, List<IconChoice>>> notebookIcons(
    String query,
  );

  /// The notebook colours the desktop offers.
  Future<Either<ApplicationError, List<ColorChoice>>> notebookColors();

  /// The tag colours the desktop offers.
  Future<Either<ApplicationError, List<ColorChoice>>> tagColors();

  /// `#rrggbb` for [text] when it spells a colour (`#rgb`, `#rrggbb`, the
  /// `#` optional and any case), or null when it does not.
  String? parseColor(String text);
}
