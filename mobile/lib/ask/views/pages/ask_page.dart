// Kanso — ask_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/models/data/ask_scope.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../injection.dart';
import '../../state/ask/ask_bloc.dart';
import '../widgets/ask_pane.dart';

/// Ask your notes, as a page of its own: a page on a phone, and a sheet over
/// the panes on a tablet ([inSheet]). What is asked is not kept after it
/// closes.
class AskPage extends StatelessWidget {
  const AskPage({this.inSheet = false, this.scope, super.key});

  final bool inSheet;

  /// What the first question searches, when Ask was opened from a notebook.
  final AskScope? scope;

  @override
  Widget build(BuildContext context) {
    return BlocProvider<AskBloc>(
      create: (context) => getIt<AskBloc>()..add(AskEvent.started(scope: scope)),
      child: KansoPage(
        // The question field rises with the keyboard.
        backgroundColor: context.colors.app,
        body: AskPane(inSheet: inSheet),
      ),
    );
  }
}
