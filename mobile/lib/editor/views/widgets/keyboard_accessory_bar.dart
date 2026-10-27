// Kanso — keyboard_accessory_bar.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart' show CustomSemanticsAction;

import '../../../core/bridge/api/types.dart' hide EditorEvent;
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import 'accessory_key.dart';

/// The bar of Markdown shortcuts above the keyboard: heading, list, task,
/// bold, link, image, `[[`, indent, outdent and undo.
///
/// Each key asks the editor for a command ([onCommand]); the editor decides
/// what text that makes. Indent and outdent are dimmed where there is
/// nothing to indent (outside a list or a table), and undo where there is
/// nothing to undo. Holding Undo redoes; a screen reader is offered Redo as
/// an action of the key.
class KeyboardAccessoryBar extends StatelessWidget {
  const KeyboardAccessoryBar({
    required this.onCommand,
    required this.onUndo,
    required this.onRedo,
    this.editorContext,
    super.key,
  });

  /// What is under the caret, which says what the keys can do; null before
  /// the editor has told.
  final EditorContext? editorContext;

  final ValueChanged<EditCommand> onCommand;
  final VoidCallback onUndo;
  final VoidCallback onRedo;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    // Taps on the bar are in the field, as far as its focus goes.
    return TextFieldTapRegion(
      child: ColoredBox(
        color: context.colors.sidebar,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: <Widget>[
            const KansoDivider(),
            SizedBox(
              height: KansoSizes.touchTarget,
              child: LayoutBuilder(
                builder: (context, constraints) {
                  final here = editorContext;
                  final indentable =
                      here != null && (here.inList || here.inTable);
                  final canUndo = here?.history.canUndo ?? false;
                  final keys = <Widget>[
                    AccessoryKey(
                      icon: KansoIcons.hash,
                      semanticLabel: l10n.accessoryHeading,
                      onTap: () => onCommand(const EditCommand.cycleHeading()),
                    ),
                    AccessoryKey(
                      icon: KansoIcons.list,
                      semanticLabel: l10n.accessoryList,
                      onTap: () => onCommand(
                        const EditCommand.format(format: FormatKind.bulleted),
                      ),
                    ),
                    AccessoryKey(
                      icon: KansoIcons.listChecks,
                      semanticLabel: l10n.accessoryTask,
                      onTap: () => onCommand(
                        const EditCommand.format(format: FormatKind.task),
                      ),
                    ),
                    AccessoryKey(
                      icon: KansoIcons.bold,
                      semanticLabel: l10n.accessoryBold,
                      onTap: () => onCommand(
                        const EditCommand.format(format: FormatKind.bold),
                      ),
                    ),
                    AccessoryKey(
                      icon: KansoIcons.link,
                      semanticLabel: l10n.accessoryLink,
                      onTap: () => onCommand(
                        const EditCommand.format(format: FormatKind.link),
                      ),
                    ),
                    AccessoryKey(
                      icon: KansoIcons.image,
                      semanticLabel: l10n.accessoryImage,
                      onTap: () => onCommand(
                        const EditCommand.format(format: FormatKind.image),
                      ),
                    ),
                    AccessoryKey(
                      glyph: '[[',
                      semanticLabel: l10n.accessoryWikiLink,
                      onTap: () => onCommand(const EditCommand.wikiLink()),
                    ),
                    AccessoryKey(
                      icon: KansoIcons.indentIncrease,
                      semanticLabel: l10n.accessoryIndent,
                      onTap: indentable
                          ? () => onCommand(const EditCommand.tab(out: false))
                          : null,
                    ),
                    AccessoryKey(
                      icon: KansoIcons.indentDecrease,
                      semanticLabel: l10n.accessoryOutdent,
                      onTap: indentable
                          ? () => onCommand(const EditCommand.tab(out: true))
                          : null,
                    ),
                    Semantics(
                      customSemanticsActions:
                          <CustomSemanticsAction, VoidCallback>{
                            CustomSemanticsAction(label: l10n.accessoryRedo):
                                onRedo,
                          },
                      child: AccessoryKey(
                        icon: KansoIcons.undo,
                        semanticLabel: l10n.accessoryUndo,
                        onTap: canUndo ? onUndo : null,
                        onLongPress: onRedo,
                      ),
                    ),
                  ];
                  // Every key stays on screen: on a phone they share the
                  // width; where there's room they keep their full size.
                  final roomy =
                      constraints.maxWidth >=
                      keys.length * KansoSizes.touchTarget +
                          KansoSpacing.xs * 2;
                  return Padding(
                    padding: const EdgeInsets.symmetric(
                      horizontal: KansoSpacing.xs,
                    ),
                    child: Row(
                      children: <Widget>[
                        for (final key in keys)
                          roomy ? key : Expanded(child: key),
                      ],
                    ),
                  );
                },
              ),
            ),
          ],
        ),
      ),
    );
  }
}
