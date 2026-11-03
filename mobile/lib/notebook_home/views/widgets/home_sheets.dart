// Kanso — home_sheets.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/notebook_home_types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';
import '../../../core/views/widgets/kanso_switch.dart';
import '../../models/data/home_layout_choice.dart';

/// Choose Home note…: the notes of the notebook to pick one from. Returns
/// the note's ID.
Future<String?> showHomeCandidatesSheet(
  BuildContext context,
  List<HomeCandidate> candidates,
) {
  return showKansoBottomSheet<String>(
    context,
    builder: (sheet) {
      final l10n = sheet.l10n;
      return ConstrainedBox(
        constraints: BoxConstraints(
          maxHeight: MediaQuery.sizeOf(sheet).height * 0.7,
        ),
        child: Padding(
          padding: const EdgeInsets.only(bottom: KansoSpacing.md),
          child: SingleChildScrollView(
            child: candidates.isEmpty
                ? Padding(
                    padding: const EdgeInsets.all(KansoSpacing.md),
                    child: Text(
                      l10n.notebookHomeChooseNone,
                      style: sheet.typography.secondary,
                    ),
                  )
                : KansoSheetGroup(
                    title: l10n.notebookHomeChooseTitle,
                    children: <Widget>[
                      for (final c in candidates)
                        KansoSettingsRow(
                          label: c.title,
                          trailing: c.current
                              ? const Icon(Icons.check, size: 18)
                              : null,
                          onTap: () => Navigator.of(sheet).pop(c.id),
                        ),
                    ],
                  ),
          ),
        ),
      );
    },
  );
}

/// Customise home…: the sections with a handle to drag each into place and a
/// switch to hide it. Returns the layout chosen.
Future<HomeLayoutChoice?> showHomeCustomiseSheet(
  BuildContext context,
  List<HomeSectionSetting> sections,
) {
  return showKansoBottomSheet<HomeLayoutChoice>(
    context,
    builder: (sheet) => HomeCustomiseSheet(sections: sections),
  );
}

/// The body of the customise sheet.
class HomeCustomiseSheet extends StatefulWidget {
  const HomeCustomiseSheet({required this.sections, super.key});

  final List<HomeSectionSetting> sections;

  @override
  State<HomeCustomiseSheet> createState() => _HomeCustomiseSheetState();
}

class _HomeCustomiseSheetState extends State<HomeCustomiseSheet> {
  late List<HomeSectionSetting> _rows = List<HomeSectionSetting>.of(
    widget.sections,
  );

  void _reorder(int from, int to) {
    setState(() {
      final next = List<HomeSectionSetting>.of(_rows);
      final moved = next.removeAt(from);
      next.insert(to, moved);
      _rows = next;
    });
  }

  void _setHidden(int at, {required bool hidden}) {
    setState(() {
      final row = _rows[at];
      _rows = List<HomeSectionSetting>.of(_rows)
        ..[at] = HomeSectionSetting(
          key: row.key,
          title: row.title,
          hidden: hidden,
        );
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    return ConstrainedBox(
      constraints: BoxConstraints(
        maxHeight: MediaQuery.sizeOf(context).height * 0.8,
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: <Widget>[
          Padding(
            padding: const EdgeInsetsDirectional.only(
              start: KansoSpacing.md,
              top: KansoSpacing.xs,
            ),
            child: Row(
              children: <Widget>[
                Expanded(
                  child: Text(
                    l10n.notebookHomeCustomiseTitle,
                    style: context.typography.barTitle,
                  ),
                ),
                KansoPressable(
                  onTap: () => Navigator.of(context).pop(
                    HomeLayoutChoice(
                      order: <String>[for (final r in _rows) r.key],
                      hidden: <String>[
                        for (final r in _rows)
                          if (r.hidden) r.key,
                      ],
                    ),
                  ),
                  child: Padding(
                    padding: const EdgeInsets.all(KansoSpacing.sm),
                    child: Text(
                      l10n.notebookHomeDone,
                      style: context.typography.body.copyWith(
                        color: colors.accent,
                      ),
                    ),
                  ),
                ),
              ],
            ),
          ),
          Flexible(
            child: ReorderableListView.builder(
              shrinkWrap: true,
              buildDefaultDragHandles: false,
              itemCount: _rows.length,
              onReorderItem: _reorder,
              itemBuilder: (context, i) {
                final row = _rows[i];
                return Padding(
                  key: ValueKey<String>(row.key),
                  padding: const EdgeInsets.symmetric(
                    horizontal: KansoSpacing.md,
                    vertical: KansoSpacing.xxs,
                  ),
                  child: Row(
                    children: <Widget>[
                      ReorderableDragStartListener(
                        index: i,
                        child: Padding(
                          padding: const EdgeInsets.only(
                            right: KansoSpacing.sm,
                            top: KansoSpacing.xs,
                            bottom: KansoSpacing.xs,
                          ),
                          child: KansoIcon(
                            KansoIcons.menu,
                            size: 18,
                            color: colors.textMuted,
                          ),
                        ),
                      ),
                      Expanded(
                        child: Text(row.title, style: context.typography.body),
                      ),
                      KansoSwitch(
                        value: !row.hidden,
                        onChanged: (shown) => _setHidden(i, hidden: !shown),
                      ),
                    ],
                  ),
                );
              },
            ),
          ),
          Padding(
            padding: const EdgeInsets.all(KansoSpacing.sm),
            child: KansoButton(
              label: l10n.notebookHomeResetLayout,
              variant: KansoButtonVariant.quiet,
              compact: true,
              expanded: false,
              onPressed: () => Navigator.of(
                context,
              ).pop(const HomeLayoutChoice(reset: true)),
            ),
          ),
        ],
      ),
    );
  }
}
