// Kanso — open_made_note.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/widgets.dart';
import 'package:go_router/go_router.dart';

import '../../core/bridge/api/types.dart';
import '../../core/models/data/browse_location.dart';
import '../../core/router/kanso_routes.dart';

/// Opens [made] in the list the person is in (All Notes when the address is
/// not a list), as a tap on its row does. The cursor and the title focus
/// are for the editor, which opens the note at them once it is there.
void openMadeNote(BuildContext context, MadeNote made) {
  final router = GoRouter.of(context);
  final here = BrowseLocation.parse(
    router.routerDelegate.currentConfiguration.uri,
  );
  final target = (here ?? const BrowseLocation.place(ListPlace.all())).withNote(
    made.id,
  );
  final parameters = <String, String>{
    if (made.cursor16 != null)
      KansoRoutes.madeCursorParameter: made.cursor16.toString(),
    if (made.titleFirst) KansoRoutes.madeTitleFirstParameter: '1',
  };
  final uri = Uri.parse(target.location);
  final location = parameters.isEmpty
      ? target.location
      : uri
            .replace(
              queryParameters: <String, String>{
                ...uri.queryParameters,
                ...parameters,
              },
            )
            .toString();
  router.go(location);
}

/// Opens the note [id] in the list the person is in.
void openNoteById(BuildContext context, String id, {bool push = false}) {
  final router = GoRouter.of(context);
  final here = BrowseLocation.parse(
    router.routerDelegate.currentConfiguration.uri,
  );
  final target = (here ?? const BrowseLocation.place(ListPlace.all()))
      .withNote(id)
      .location;
  if (push) {
    unawaited(router.push<void>(target));
  } else {
    router.go(target);
  }
}
