// Kanso — notebook_icon_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart' show Either;
import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_icon_set_scope.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_search_input.dart';
import '../../../core/views/widgets/kanso_themed_icon.dart';
import '../../../injection.dart';

/// What the icon picker gave back: a Lucide [name], or null for the
/// default, which keeps no icon of its own.
class IconPick {
  const IconPick(this.name);

  final String? name;
}

/// The icons a notebook can show, as the desktop's picker has them: the
/// default first, then every choice, found by name or by words.
class NotebookIconSheet extends StatefulWidget {
  const NotebookIconSheet({this.current, super.key});

  /// The icon the notebook keeps now, if any.
  final String? current;

  /// Shows the picker over the whole app; null when it was closed without
  /// a choice.
  static Future<IconPick?> show(BuildContext context, {String? current}) {
    return showKansoBottomSheet<IconPick>(
      context,
      builder: (_) => NotebookIconSheet(current: current),
    );
  }

  @override
  State<NotebookIconSheet> createState() => _NotebookIconSheetState();
}

class _NotebookIconSheetState extends State<NotebookIconSheet> {
  final LibraryRepository library = getIt<LibraryRepository>();
  List<IconChoice> icons = const <IconChoice>[];

  @override
  void initState() {
    super.initState();
    _load('');
  }

  Future<void> _load(String text) async {
    final Either<ApplicationError, List<IconChoice>> result =
        await library.notebookIcons(text);
    if (!mounted) {
      return;
    }
    setState(() {
      icons = result.getOrElse(() => const <IconChoice>[]);
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    return Padding(
      padding: const EdgeInsetsDirectional.fromSTEB(
        KansoSpacing.md,
        KansoSpacing.xs,
        KansoSpacing.md,
        KansoSpacing.md,
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Row(
            children: <Widget>[
              Expanded(
                child: Text(
                  l10n.libraryChangeIconTitle,
                  style: context.typography.barTitle,
                ),
              ),
              KansoPressable(
                onTap: () => Navigator.of(context).maybePop(),
                child: Padding(
                  padding: const EdgeInsets.all(KansoSpacing.sm),
                  child: Text(
                    l10n.searchCancel,
                    style: context.typography.body.copyWith(
                      color: colors.accent,
                    ),
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: KansoSpacing.xs),
          KansoSearchInput(
            hint: l10n.libraryIconSearchHint,
            onChanged: _load,
            clearLabel: l10n.searchClear,
          ),
          const SizedBox(height: KansoSpacing.sm),
          SizedBox(
            height: 360,
            child: GridView.builder(
              gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
                maxCrossAxisExtent: 76,
                mainAxisExtent: 64,
                crossAxisSpacing: KansoSpacing.xs,
                mainAxisSpacing: KansoSpacing.xs,
              ),
              itemCount: icons.length + 1,
              itemBuilder: (context, position) {
                // The first cell stands for the default: no icon kept.
                if (position == 0) {
                  return _IconCell(
                    name: 'notebook',
                    label: l10n.libraryIconDefault,
                    selected: widget.current == null,
                    onTap: () =>
                        Navigator.of(context).pop(const IconPick(null)),
                  );
                }
                final choice = icons[position - 1];
                return _IconCell(
                  name: choice.name,
                  label: choice.label,
                  selected: widget.current == choice.name,
                  onTap: () => Navigator.of(context).pop(IconPick(choice.name)),
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}

/// One icon of the picker: the glyph, its name, and a ring while chosen.
class _IconCell extends StatelessWidget {
  const _IconCell({
    required this.name,
    required this.label,
    required this.selected,
    required this.onTap,
  });

  final String name;
  final String label;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return KansoPressable(
      onTap: onTap,
      semanticLabel: label,
      child: Container(
        decoration: BoxDecoration(
          color: selected ? colors.sidebarActive : null,
          border: Border.all(
            color: selected ? colors.accent : colors.divider,
          ),
          borderRadius: KansoRadii.smallBorder,
        ),
        padding: const EdgeInsets.symmetric(
          horizontal: KansoSpacing.xxs,
          vertical: KansoSpacing.xxs,
        ),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: <Widget>[
            KansoThemedIcon(
              lucideName: name,
              svg: KansoIconSetScope.of(context).svgFor(name),
              color: selected ? colors.text : colors.textSecondary,
            ),
            const SizedBox(height: 2),
            Text(
              label,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              textAlign: TextAlign.center,
              style: context.typography.meta.copyWith(color: colors.textMuted),
            ),
          ],
        ),
      ),
    );
  }
}
