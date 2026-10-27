// Kanso — theme_picker_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../state/theme_gallery/theme_gallery_bloc.dart';
import '../../state/theme_picker/theme_picker_bloc.dart';
import '../widgets/theme_picker_pane.dart';

/// Settings › Theme: the themes the app can be drawn in, and the gallery to
/// install more from.
class ThemePickerPage extends StatelessWidget {
  const ThemePickerPage({super.key});

  @override
  Widget build(BuildContext context) {
    return MultiBlocProvider(
      providers: <BlocProvider<dynamic>>[
        BlocProvider<ThemePickerBloc>(
          create: (context) =>
              getIt<ThemePickerBloc>()..add(const ThemePickerEvent.started()),
        ),
        BlocProvider<ThemeGalleryBloc>(
          create: (context) =>
              getIt<ThemeGalleryBloc>()..add(const ThemeGalleryEvent.started()),
        ),
      ],
      child: Scaffold(
        resizeToAvoidBottomInset: false,
        backgroundColor: context.colors.app,
        body: const ThemePickerPane(),
      ),
    );
  }
}
