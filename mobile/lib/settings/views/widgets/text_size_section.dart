// Kanso — text_size_section.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/kanso_text_size.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/font_labels.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_segment.dart';
import '../../../core/views/widgets/kanso_segmented.dart';

/// The text size control: Kanso's segmented control over a paragraph set
/// in the preview font at the chosen size, so the effect is seen at once.
///
/// It serves the editor's text size and the preview's: [title] and [footer]
/// say which, and default to the editor's.
class TextSizeSection extends StatelessWidget {
  const TextSizeSection({
    required this.selected,
    required this.onChanged,
    this.title,
    this.footer,
    super.key,
  });

  final String? title;
  final String? footer;

  final KansoTextSize selected;
  final ValueChanged<KansoTextSize> onChanged;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return KansoGroupedSection(
      title: title ?? l10n.fontsTextSizeSection,
      footer: footer ?? l10n.fontsTextSizeFooter,
      children: <Widget>[
        Padding(
          padding: const EdgeInsets.all(KansoSpacing.sm),
          child: KansoSegmented<KansoTextSize>(
            expanded: true,
            selected: selected,
            semanticLabel: title ?? l10n.fontsTextSizeSection,
            onChanged: onChanged,
            segments: <KansoSegment<KansoTextSize>>[
              for (final size in KansoTextSize.values)
                KansoSegment<KansoTextSize>(
                  value: size,
                  label: l10n.textSizeName(size),
                ),
            ],
          ),
        ),
        Padding(
          padding: const EdgeInsets.fromLTRB(
            KansoSpacing.md,
            KansoSpacing.xs,
            KansoSpacing.md,
            KansoSpacing.md,
          ),
          child: Text(
            l10n.fontsTextSizeSample,
            style: context.typography.previewBody,
          ),
        ),
      ],
    );
  }
}
