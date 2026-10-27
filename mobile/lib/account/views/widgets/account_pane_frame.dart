// Kanso — account_pane_frame.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../shell/views/widgets/shell_back_button.dart';

/// What every account screen is drawn in: the page colour, a top bar with a
/// [title] and, unless [leading] is given, a back chevron, and the
/// [children] scrolling below it with the page's gutter.
///
/// [leading] of `SizedBox.shrink()` leaves the bar bare, for a screen that
/// is left only by what it asks.
class AccountPaneFrame extends StatelessWidget {
  const AccountPaneFrame({
    required this.title,
    required this.children,
    this.leading = const ShellBackButton(),
    this.trailing = const <Widget>[],
    super.key,
  });

  final String title;
  final List<Widget> children;
  final Widget leading;
  final List<Widget> trailing;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final bottom = MediaQuery.paddingOf(context).bottom;
    return ColoredBox(
      color: colors.app,
      child: Column(
        children: <Widget>[
          KansoTopBar(
            backgroundColor: colors.app,
            bordered: false,
            leading: leading,
            trailing: trailing,
            title: title,
          ),
          Expanded(
            child: ListView(
              keyboardDismissBehavior: ScrollViewKeyboardDismissBehavior.onDrag,
              padding: EdgeInsets.fromLTRB(
                KansoSpacing.md,
                KansoSpacing.xs,
                KansoSpacing.md,
                bottom + KansoSpacing.lg,
              ),
              children: children,
            ),
          ),
        ],
      ),
    );
  }
}
