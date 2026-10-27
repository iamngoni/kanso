// Kanso — default_notebook_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/utils/context_extensions.dart';
import '../widgets/default_notebook_pane.dart';

/// Settings › General › Default notebook: the notebook new notes go in.
class DefaultNotebookPage extends StatelessWidget {
  const DefaultNotebookPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      resizeToAvoidBottomInset: false,
      backgroundColor: context.colors.app,
      body: const DefaultNotebookPane(),
    );
  }
}
