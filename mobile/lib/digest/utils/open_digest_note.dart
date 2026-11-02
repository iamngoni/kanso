// Kanso — open_digest_note.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:go_router/go_router.dart';

import '../../core/bridge/api/types.dart';
import '../../core/models/data/browse_location.dart';
import '../../core/router/kanso_routes.dart';

/// Opens note [noteId] in All Notes, with the cursor on [line] (0-based)
/// when it is given: how a loop or a task of the brief takes the person to
/// where it is written.
void openDigestNote(BuildContext context, String noteId, {int? line}) {
  final target = const BrowseLocation.place(ListPlace.all()).withNote(noteId);
  final uri = Uri.parse(target.location);
  context.go(
    uri
        .replace(
          queryParameters: <String, String>{
            ...uri.queryParameters,
            if (line != null) KansoRoutes.taskLineParameter: line.toString(),
          },
        )
        .toString(),
  );
}
