// Kanso — people_menu.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_menu.dart';

/// A "more" button that opens [entries] in the app's menu just under it and
/// reports the choice.
class PeopleMenuButton<T> extends StatelessWidget {
  const PeopleMenuButton({
    required this.semanticLabel,
    required this.entries,
    required this.onChosen,
    super.key,
  });

  final String semanticLabel;
  final List<KansoMenuEntry<T>> entries;
  final ValueChanged<T> onChosen;

  @override
  Widget build(BuildContext context) {
    return KansoIconButton(
      icon: KansoIcons.ellipsis,
      semanticLabel: semanticLabel,
      onPressed: () async {
        final box = context.findRenderObject() as RenderBox?;
        if (box == null) {
          return;
        }
        final at = box.localToGlobal(Offset(0, box.size.height));
        final chosen = await showKansoMenu<T>(context, at, entries);
        if (chosen != null) {
          onChosen(chosen);
        }
      },
    );
  }
}
