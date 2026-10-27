// Kanso — capture_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../bridge/api/types.dart';
import '../../data/application_error.dart';

/// What gets into the library from outside the editor: the items other apps
/// shared (left in the library's inbox folder), what is picked in the app
/// and today's daily note.
abstract interface class CaptureRepository {
  /// Saves every finished shared item waiting in the inbox folder as a note
  /// in the Inbox, removes what was saved, and tells the lists to read
  /// again (another process may have saved items the library never
  /// reported). A locked Inbox discards what was shared; that and every
  /// item left out is in the result's problems.
  Future<Either<ApplicationError, DropsSaved>> saveDrops();

  /// Saves what was picked or shared inside the app as a note in the
  /// Inbox. A locked Inbox is a [LockedError]; files left out are in the
  /// outcome's problems, a sentence each.
  Future<Either<ApplicationError, CaptureOutcome>> capture(Share share);

  /// The ID of today's daily note, made if it isn't there. [now] is the
  /// phone's local date and time.
  Future<Either<ApplicationError, String>> todayNote(DateTime now);
}
