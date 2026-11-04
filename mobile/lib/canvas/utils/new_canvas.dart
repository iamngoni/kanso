// Kanso — new_canvas.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:go_router/go_router.dart';

import '../../core/router/kanso_routes.dart';
import '../../injection.dart';
import '../models/repos/abs/canvas_repository.dart';

/// Makes an empty canvas in [notebookId] (the preferred notebook, then the
/// first, when null) and opens it.
Future<void> newCanvas(BuildContext context, {String? notebookId}) async {
  final router = GoRouter.of(context);
  final made = await getIt<CanvasRepository>().create(notebookId: notebookId);
  final id = made.toOption().toNullable()?.id;
  if (id != null) {
    await router.push('${KansoRoutes.noteById}/$id');
  }
}
