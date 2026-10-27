// Kanso: import_page.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../state/import_flow/import_flow_bloc.dart';
import '../widgets/import_pane.dart';

/// Import notes: from another app's export, or a folder of Markdown.
///
/// Given a [path] (a file or folder the app already has, such as an export
/// that was shared) it reads that instead of asking for one; [kind] names
/// the source when it is known (see `KansoRoutes.importFile`).
class ImportPage extends StatelessWidget {
  const ImportPage({this.path, this.kind, super.key});

  final String? path;
  final String? kind;

  @override
  Widget build(BuildContext context) {
    return BlocProvider<ImportFlowBloc>(
      create: (context) =>
          getIt<ImportFlowBloc>()
            ..add(ImportFlowEvent.started(path: path, kind: kind)),
      child: Scaffold(
        resizeToAvoidBottomInset: false,
        backgroundColor: context.colors.app,
        body: const ImportPane(),
      ),
    );
  }
}
