// Kanso — icon_themes_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../state/icon_themes/icon_themes_bloc.dart';
import '../widgets/icon_themes_pane.dart';

/// Settings › Icons: the icon themes the app can be drawn in.
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
