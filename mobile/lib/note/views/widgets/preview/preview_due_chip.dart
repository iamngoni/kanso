// Kanso — preview_due_chip.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../../core/bridge/api/types.dart';
import '../../../../core/configs/kanso_icons.dart';
import '../../../../core/configs/kanso_spacing.dart';
import '../../../../core/utils/context_extensions.dart';
import '../../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../../core/views/widgets/kanso_settings_row.dart';
import '../../../../core/views/widgets/kanso_sheet_group.dart';
import '../../../state/preview/preview_bloc.dart' show stopRepeatingLink;
import 'preview_list.dart';
import 'preview_style.dart';

/// A due date as the page draws it: a small rounded chip in its tone,
/// [size] the size of the text it is in.
///
/// An open repeating task's chip (its [url] has a repeat, inside a task item
/// of the page) opens its menu on a tap, with "Stop repeating", which asks
/// the page to drop the task's `every …` words (`kanso-stop-repeat:<index>`).
class PreviewDueChip extends StatelessWidget {
  const PreviewDueChip({
    required this.label,
    required this.title,
    required this.tone,
    required this.size,
    this.url,
    this.onTap,
    super.key,
  });

  final String label;
  final String title;
  final DueTone tone;
  final double size;

  /// The chip's `kanso-due:` address, which says whether its task repeats.
  final String? url;

  /// What a tap on it does; nothing takes the tap without one.
  final VoidCallback? onTap;

  /// Whether a chip with [url] and [tone] is an open repeating task's.
  static bool repeats(String? url, DueTone tone) =>
      tone != DueTone.done && (url?.contains(';repeat=') ?? false);

  @override
  Widget build(BuildContext context) {
    final look = PreviewScope.of(context).style;
    final red = look.palette.dark
        ? const Color(0xFFF2766B)
        : const Color(0xFFB3261E);
    Color tint(Color color) => previewOver(color, look.background, 0.16);
    final (Color text, Color fill) = switch (tone) {
      DueTone.plain => (look.muted, look.surface),
      DueTone.today => (look.accent, tint(look.accent)),
      DueTone.overdue => (red, tint(red)),
      DueTone.done => (
        look.muted,
        previewOver(look.surface, look.background, 0.6),
      ),
    };
    final fontSize = size * 0.8667;
    final Widget box = Container(
      padding: EdgeInsets.symmetric(horizontal: fontSize * 0.55),
      decoration: BoxDecoration(
        color: fill,
        borderRadius: BorderRadius.circular(999),
      ),
      child: Text(
        label,
        style: look.body.copyWith(
          fontSize: fontSize,
          height: 1.5,
          color: text,
          fontWeight: FontWeight.w400,
          fontStyle: FontStyle.normal,
          decoration: TextDecoration.none,
        ),
      ),
    );
    final place = PreviewTaskPlace.of(context);
    final VoidCallback? tap = place != null && repeats(url, tone)
        ? () => _menu(context, place)
        : onTap;
    final chip = tap == null ? box : GestureDetector(onTap: tap, child: box);
    return title.isEmpty ? chip : Tooltip(message: title, child: chip);
  }

  /// The chip's menu; "Stop repeating" asks the page for it.
  Future<void> _menu(BuildContext context, int index) async {
    final onLink = PreviewScope.of(context).onLink;
    final stop = await DueChipSheet.show(context, title: title);
    if (stop ?? false) {
      onLink('$stopRepeatingLink$index');
    }
  }
}

/// A repeating task's due chip menu: its date in words, and Stop repeating.
/// It pops with true when that is chosen.
class DueChipSheet extends StatelessWidget {
  const DueChipSheet({required this.title, super.key});

  /// The chip's longer words, such as "Friday, 14 November 2026 at 15:00".
  final String title;

  /// Shows the sheet over the whole app and completes with true when Stop
  /// repeating is chosen.
  static Future<bool?> show(BuildContext context, {required String title}) {
    return showKansoBottomSheet<bool>(
      context,
      builder: (_) => DueChipSheet(title: title),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        if (title.isNotEmpty)
          Padding(
            padding: const EdgeInsets.fromLTRB(
              KansoSpacing.md,
              KansoSpacing.xs,
              KansoSpacing.md,
              0,
            ),
            child: Text(
              title,
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
              style: context.typography.barTitle,
            ),
          ),
        KansoSheetGroup(
          dividerIndent: 48,
          children: <Widget>[
            KansoSettingsRow(
              icon: KansoIcons.calendar,
              label: context.l10n.previewStopRepeating,
              onTap: () => Navigator.of(context).pop(true),
            ),
          ],
        ),
        const SizedBox(height: KansoSpacing.md),
      ],
    );
  }
}
