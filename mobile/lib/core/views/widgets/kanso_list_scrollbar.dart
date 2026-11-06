// Kanso — kanso_list_scrollbar.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../state/appearance/appearance_bloc.dart';

/// Adds the app's saved transient or persistent scrollbar behavior to a list.
class KansoListScrollbar extends StatelessWidget {
  const KansoListScrollbar({required this.child, this.controller, super.key});

  final Widget child;
  final ScrollController? controller;

  @override
  Widget build(BuildContext context) {
    try {
      context.read<AppearanceBloc>();
    } on ProviderNotFoundException {
      return child;
    }
    return BlocBuilder<AppearanceBloc, AppearanceState>(
      buildWhen: (before, after) =>
          before.alwaysShowScrollbars != after.alwaysShowScrollbars,
      builder: (context, appearance) => appearance.alwaysShowScrollbars
          ? Scrollbar(
              controller: controller,
              thumbVisibility: true,
              child: child,
            )
          : child,
    );
  }
}
