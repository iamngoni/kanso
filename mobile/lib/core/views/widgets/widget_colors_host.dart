// Kanso — widget_colors_host.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../state/appearance/appearance_bloc.dart';
import '../../state/widget_colors/widget_colors_bloc.dart';

/// [child] with the widgets' colours kept in step with the app's themes:
/// whenever the light or dark theme changes, the [WidgetColorsBloc] is told.
/// It needs an [AppearanceBloc] and a [WidgetColorsBloc] above it.
class WidgetColorsHost extends StatelessWidget {
  const WidgetColorsHost({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    return BlocListener<AppearanceBloc, AppearanceState>(
      listenWhen: (before, after) =>
          before.light != after.light || before.dark != after.dark,
      listener: (context, state) => context.read<WidgetColorsBloc>().add(
        WidgetColorsEvent.changed(
          light: state.light.colors,
          dark: state.dark.colors,
        ),
      ),
      child: child,
    );
  }
}
