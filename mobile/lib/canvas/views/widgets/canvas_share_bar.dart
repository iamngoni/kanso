// Kanso — canvas_share_bar.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/canvas_types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import 'canvas_pointers.dart';

/// A quiet line under a shared canvas's bar: that it is shared and, when the
/// person may only look, that it is view only, with the people who are here
/// now as a row of dots in the colours their pointers have. A private canvas
/// has none.
class CanvasShareBar extends StatelessWidget {
  const CanvasShareBar({
    required this.view,
    required this.pointers,
    super.key,
  });

  final CanvasView view;
  final List<CanvasPointerRow> pointers;

  @override
  Widget build(BuildContext context) {
    if (!view.shared) {
      return const SizedBox.shrink();
    }
    final colors = context.colors;
    final l10n = context.l10n;
    final dark = context.isDark;
    final label = view.readOnly
        ? '${l10n.canvasSharedLabel} · ${l10n.canvasViewOnlyLabel}'
        : l10n.canvasSharedLabel;
    return Padding(
      padding: const EdgeInsets.fromLTRB(
        KansoSpacing.md,
        KansoSpacing.xxs,
        KansoSpacing.md,
        KansoSpacing.xxs,
      ),
      child: Row(
        children: <Widget>[
          KansoIcon(
            KansoIcons.users,
            size: KansoSizes.iconSmall,
            color: colors.textMuted,
          ),
          const SizedBox(width: KansoSpacing.xxs),
          Expanded(
            child: Text(
              label,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: context.typography.meta,
            ),
          ),
          if (pointers.isNotEmpty) ...<Widget>[
            _Dots(pointers: pointers, dark: dark, ring: colors.editor),
            const SizedBox(width: KansoSpacing.xxs),
            Text(
              l10n.canvasOthersHere(pointers.length),
              style: context.typography.meta,
            ),
          ],
        ],
      ),
    );
  }
}

/// The people here, overlapping, at most four.
class _Dots extends StatelessWidget {
  const _Dots({
    required this.pointers,
    required this.dark,
    required this.ring,
  });

  final List<CanvasPointerRow> pointers;
  final bool dark;
  final Color ring;

  static const double _size = 16;
  static const double _step = 11;

  @override
  Widget build(BuildContext context) {
    final shown = pointers.take(4).toList(growable: false);
    return SizedBox(
      width: _size + _step * (shown.length - 1),
      height: _size,
      child: Stack(
        children: <Widget>[
          for (var i = 0; i < shown.length; i++)
            Positioned(
              left: i * _step,
              child: Container(
                width: _size,
                height: _size,
                decoration: BoxDecoration(
                  color: pointerColor(
                    dark ? shown[i].colourDark : shown[i].colourLight,
                  ),
                  shape: BoxShape.circle,
                  border: Border.all(color: ring, width: 2),
                ),
              ),
            ),
        ],
      ),
    );
  }
}
