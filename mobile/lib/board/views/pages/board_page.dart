// Kanso — board_page.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/list_options.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../state/board/board_bloc.dart';
import '../widgets/board_pane.dart';

/// The board of a place's notes: the notes by status, or their tasks as To
/// do and Done. Read when it is opened and given up when it is left.
class BoardPage extends StatelessWidget {
  const BoardPage({
    required this.place,
    this.kind = BoardKind.notes,
    this.inPane = false,
    super.key,
  });

  final ListPlace place;
  final BoardKind kind;

  /// Whether it is the note pane of a wide window, beside the list: cards
  /// are dragged between columns there.
  final bool inPane;

  @override
  Widget build(BuildContext context) {
    return BlocProvider<BoardBloc>(
      key: ValueKey<ListPlace>(place),
      create: (_) => getIt<BoardBloc>()
        ..add(
          BoardEvent.started(
            query: const ListOptions().queryFor(place),
            kind: kind,
          ),
        ),
      child: Scaffold(
        resizeToAvoidBottomInset: false,
        backgroundColor: context.colors.editor,
        body: BoardPane(place: place, draggable: inPane),
      ),
    );
  }
}
