// Kanso — seed_library.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/library_repository.dart';
import 'package:kanso/core/models/repos/abs/session_repository.dart';
import 'package:kanso/injection.dart';

/// What [seedLibrary] made, for the flows to look for and open.
class SeededLibrary {
  const SeededLibrary({
    required this.work,
    required this.garden,
    required this.sprintPlan,
    required this.standupNotes,
    required this.gardenBeds,
    required this.seedCatalogue,
    required this.weeklyReview,
    required this.privateJournal,
  });

  /// The notebooks.
  final String work;
  final String garden;

  /// The notes, by title.
  final String sprintPlan;
  final String standupNotes;
  final String gardenBeds;
  final String seedCatalogue;
  final String weeklyReview;
  final String privateJournal;

  /// The text of the locked note: it must never be on screen.
  static const String secretBody = 'The vault code is nine nine nine';

  /// The password it was locked under (a test value, for this library
  /// only).
  static const String lockPassword = 'integration-test-password';
}

/// Fills the library the app is running on, the way a person's notes would
/// be: two notebooks, notes with `#todo` in their text (so they are tagged),
/// a pinned note and a locked one, closed.
///
/// It goes through the app's own repositories, so the screens learn of it
/// from the library's change stream, as they would of a sync.
Future<SeededLibrary> seedLibrary() async {
  final library = getIt<LibraryRepository>();
  final session = getIt<SessionRepository>();

  T made<T>(Either<ApplicationError, T> result) => result.fold(
    (error) => throw StateError('Seeding the library failed: $error'),
    (value) => value,
  );

  Future<String> note(String title, String text, {String? notebookId}) async {
    final saved = made(
      await library.create(
        NewNote(
          notebookId: notebookId,
          title: title,
          body: '# $title\n\n$text',
        ),
      ),
    );
    return saved.id;
  }

  final work = made(await library.createNotebook('Work')).notebook.id;
  final garden = made(await library.createNotebook('Garden')).notebook.id;

  final sprintPlan = await note(
    'Sprint plan',
    'Ship the beta build to testers. #todo',
    notebookId: work,
  );
  final standupNotes = await note(
    'Standup notes',
    'Beta feedback goes first on Monday.',
    notebookId: work,
  );
  final gardenBeds = await note(
    'Garden beds',
    'Three raised beds along the fence. #todo',
    notebookId: garden,
  );
  final seedCatalogue = await note(
    'Seed catalogue',
    'Order beans before the garden shop closes.',
    notebookId: garden,
  );
  final weeklyReview = await note(
    'Weekly review',
    'Review the week and plan the next one.',
  );
  made(
    await library.change(<String>[weeklyReview], const NoteChange.togglePin()),
  );

  final privateJournal = await note(
    'Private journal',
    SeededLibrary.secretBody,
  );
  made(
    await session.lockNote(privateJournal, SeededLibrary.lockPassword),
  );
  // Locking the first note leaves the locked notes open; close them.
  made(await session.lock());

  return SeededLibrary(
    work: work,
    garden: garden,
    sprintPlan: sprintPlan,
    standupNotes: standupNotes,
    gardenBeds: gardenBeds,
    seedCatalogue: seedCatalogue,
    weeklyReview: weeklyReview,
    privateJournal: privateJournal,
  );
}

/// Fills the library with [notes] notes in [notebooks] notebooks, for
/// measuring a large library: each note has a title and a few lines of
/// text, as a person's would.
Future<void> seedManyNotes({int notes = 2000, int notebooks = 8}) async {
  final library = getIt<LibraryRepository>();
  final books = <String>[];
  for (var i = 0; i < notebooks; i++) {
    final book = await library.createNotebook('Notebook ${i + 1}');
    books.add(
      book.fold(
        (error) => throw StateError('Seeding the library failed: $error'),
        (row) => row.notebook.id,
      ),
    );
  }
  // A few at a time: the core writes one note at a time anyway.
  const batch = 25;
  for (var from = 0; from < notes; from += batch) {
    final to = from + batch < notes ? from + batch : notes;
    final made = await Future.wait(<Future<Either<ApplicationError, Saved>>>[
      for (var i = from; i < to; i++)
        library.create(
          NewNote(
            notebookId: books[i % books.length],
            title: 'Note ${(i + 1).toString().padLeft(4, '0')}',
            body:
                '# Note ${i + 1}\n\nThe ${i + 1}th note of the library, '
                'with a line or two of text so its row has something to '
                'show.\n\n- one thing\n- another #todo\n',
          ),
        ),
    ]);
    for (final result in made) {
      result.fold(
        (error) => throw StateError('Seeding the library failed: $error'),
        (_) {},
      );
    }
  }
}
