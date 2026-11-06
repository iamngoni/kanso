// Kanso — settings_category_pane.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_large_title.dart';
import '../../../core/views/widgets/kanso_list_scrollbar.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../shell/views/widgets/shell_back_button.dart';

/// One category of Settings (General, Appearance…) on a page of its own: a
/// back button, a large [title] that folds into the bar as the list scrolls,
/// and the category's [sections], spaced as on Settings itself.
class SettingsCategoryPane extends StatelessWidget {
  const SettingsCategoryPane({
    required this.title,
    required this.sections,
    super.key,
  });

  /// How far a row's hairline is inset: past its icon.
  static const double iconInset = 48;

  final String title;
  final List<Widget> sections;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final bottom = MediaQuery.paddingOf(context).bottom;
    return ColoredBox(
      color: colors.app,
      child: KansoLargeTitleScroll(
        builder: (context, collapsed) => Column(
          children: <Widget>[
            KansoTopBar(
              backgroundColor: colors.app,
              titleShown: collapsed,
              leading: const ShellBackButton(),
              title: title,
            ),
            Expanded(
              child: KansoListScrollbar(
                child: ListView(
                  padding: EdgeInsets.fromLTRB(
                    KansoSpacing.md,
                    0,
                    KansoSpacing.md,
                    bottom + KansoSpacing.lg,
                  ),
                  children: <Widget>[
                    KansoLargeTitle(
                      title: title,
                      padding: const EdgeInsets.only(bottom: KansoSpacing.md),
                    ),
                    for (final (index, section)
                        in sections.indexed) ...<Widget>[
                      if (index > 0) const SizedBox(height: KansoSpacing.lg),
                      section,
                    ],
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
