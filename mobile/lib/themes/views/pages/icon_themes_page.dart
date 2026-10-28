// Kanso — icon_themes_page.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../state/icon_themes/icon_themes_bloc.dart';
import '../widgets/icon_themes_pane.dart';

/// Settings › Icon themes: the icon themes the phone has, and importing
/// one.
class IconThemesPage extends StatelessWidget {
  const IconThemesPage({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider<IconThemesBloc>(
      create: (context) =>
          getIt<IconThemesBloc>()..add(const IconThemesEvent.started()),
      child: Scaffold(
        resizeToAvoidBottomInset: false,
        backgroundColor: context.colors.app,
        body: const IconThemesPane(),
      ),
    );
  }
}
