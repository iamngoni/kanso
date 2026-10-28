// Kanso — rust_note_links_repository.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/library.dart';
import '../../../../core/bridge/api/linking_types.dart';
import '../../../../core/models/data/application_error.dart';
import '../../../../core/utils/kanso_error_mapping.dart';
import '../abs/note_links_repository.dart';

/// [NoteLinksRepository] over the Rust core's linking.
@LazySingleton(as: NoteLinksRepository)
class RustNoteLinksRepository implements NoteLinksRepository {
  RustNoteLinksRepository(this._library);

  final KansoLibrary _library;

  @override
  Either<ApplicationError, List<OutlineHeading>> outline(String text) =>
      attemptSync(() => _library.linking().outline(text: text));

  @override
  Future<Either<ApplicationError, List<LinkRef>>> backlinks(String id) =>
      attempt(() => _library.linking().backlinks(id: id));

  @override
  Future<Either<ApplicationError, List<LinkRef>>> unlinkedMentions(String id) =>
      attempt(() => _library.linking().unlinkedMentions(id: id));
}
