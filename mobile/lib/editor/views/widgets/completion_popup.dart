// Kanso — completion_popup.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_shadows.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../search/views/widgets/highlighted_text.dart';

/// The notes (or tags) offered while a `[[` link (or a `#tag`) is typed:
/// each one's title, with what was typed highlighted, and its notebook at
/// the end. The row that is active is the one Return takes.
///
/// Up to [visibleRows] rows show; the rest scroll, and the active row is
/// kept in view. A screen reader hears how many there are and which one is
/// selected.
class CompletionPopup extends StatefulWidget {
  const CompletionPopup({
    required this.completion,
    required this.activeIndex,
    required this.onChoose,
    super.key,
  });

  final Completion completion;

  /// The row that is active.
  final int activeIndex;

  /// A row was tapped.
  final ValueChanged<CompletionItem> onChoose;

  /// How many rows show before the list scrolls.
  static const int visibleRows = 5;

  /// How tall a row is, hairline included.
  static const double rowHeight = KansoSizes.touchTarget + 1;

  /// How tall the popup is for [rows] rows, its border included.
  static double heightFor(int rows) =>
      (rows < visibleRows ? rows : visibleRows) * rowHeight - 1 + 2;

  @override
  State<CompletionPopup> createState() => _CompletionPopupState();
}

class _CompletionPopupState extends State<CompletionPopup> {
  final ScrollController scroll = ScrollController();

  @override
  void didUpdateWidget(CompletionPopup oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.activeIndex != widget.activeIndex) {
      WidgetsBinding.instance.addPostFrameCallback((_) => keepActiveInView());
    }
  }

  /// Scrolls just far enough for the active row to be wholly in view.
  void keepActiveInView() {
    if (!mounted || !scroll.hasClients) {
      return;
    }
    final top = widget.activeIndex * CompletionPopup.rowHeight;
    final bottom = top + CompletionPopup.rowHeight - 1;
    final position = scroll.position;
    if (top < position.pixels) {
      scroll.jumpTo(top);
    } else if (bottom > position.pixels + position.viewportDimension) {
      scroll.jumpTo(bottom - position.viewportDimension);
    }
  }

  @override
  void dispose() {
    scroll.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    final items = widget.completion.items;
    final active = widget.activeIndex.clamp(0, items.length - 1);
    return Semantics(
      container: true,
      liveRegion: true,
      label: items.isEmpty
          ? null
          : context.l10n.completionAnnouncement(
              items.length,
              items[active].label,
            ),
      child: Container(
        clipBehavior: Clip.antiAlias,
        decoration: BoxDecoration(
          color: colors.elevated,
          borderRadius: KansoRadii.mediumBorder,
          border: Border.all(color: colors.divider),
          boxShadow: KansoShadows.floating(Theme.brightnessOf(context)),
        ),
        child: SizedBox(
          height: CompletionPopup.heightFor(items.length) - 2,
          child: ListView.separated(
            controller: scroll,
            padding: EdgeInsets.zero,
            physics: const ClampingScrollPhysics(),
            itemCount: items.length,
            separatorBuilder: (context, at) => const KansoDivider(),
            itemBuilder: (context, at) {
              final item = items[at];
              final detail = item.detail;
              final highlight = item.highlight;
              return Semantics(
                selected: at == active,
                child: KansoPressable(
                  onTap: () => widget.onChoose(item),
                  child: Container(
                    height: KansoSizes.touchTarget,
                    color: at == active ? colors.rowActive : null,
                    padding: const EdgeInsets.symmetric(
                      horizontal: KansoSpacing.sm,
                    ),
                    child: Row(
                      children: <Widget>[
                        Expanded(
                          child: HighlightedText(
                            item.label,
                            ranges: <Range16>[?highlight],
                            style: typography.body,
                            maxLines: 1,
                          ),
                        ),
                        if (detail != null) ...<Widget>[
                          const SizedBox(width: KansoSpacing.xs),
                          ConstrainedBox(
                            constraints: const BoxConstraints(maxWidth: 120),
                            child: Text(
                              detail,
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                              style: typography.meta,
                            ),
                          ),
                        ],
                      ],
                    ),
                  ),
                ),
              );
            },
          ),
        ),
      ),
    );
  }
}
