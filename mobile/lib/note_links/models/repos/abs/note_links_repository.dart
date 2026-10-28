// Kanso — note_links_repository.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/bridge/api/linking_types.dart';
import '../../../../core/models/data/application_error.dart';

/// The links around a note: the headings of its text, the notes that name
/// it, and the notes that name it without a link.
abstract interface class NoteLinksRepository {
  /// The headings of [text], in the order they appear.
  Either<ApplicationError, List<OutlineHeading>> outline(String text);

  /// The notes that link to the note [id], with the line each does it on.
  Future<Either<ApplicationError, List<LinkRef>>> backlinks(String id);

  /// The notes that name the note [id] without linking to it.
  Future<Either<ApplicationError, List<LinkRef>>> unlinkedMentions(String id);
}
