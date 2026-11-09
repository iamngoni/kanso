// Kanso — kanso_time_sheet.dart
// Created 2026-11-09.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';

import '../../configs/kanso_spacing.dart';
import '../../utils/clock_time.dart';
import '../../utils/context_extensions.dart';
import 'kanso_bottom_sheet_route.dart';
import 'kanso_button.dart';
import 'kanso_pressable.dart';

/// Picks a time of day with the iOS wheel inside Kanso's own bottom sheet,
/// so no Material dialog appears for it.
class KansoTimeSheet extends StatefulWidget {
  const KansoTimeSheet({
    required this.title,
    required this.initial,
    super.key,
  });

  final String title;
  final DateTime initial;

  /// Shows the picker; the chosen time as `HH:mm`, or null when cancelled.
  static Future<String?> show(
    BuildContext context, {
    required String title,
    required DateTime initial,
  }) => showKansoBottomSheet<String>(
    context,
    builder: (_) => KansoTimeSheet(title: title, initial: initial),
  );

  @override
  State<KansoTimeSheet> createState() => _KansoTimeSheetState();
}

class _KansoTimeSheetState extends State<KansoTimeSheet> {
  late DateTime _value = widget.initial;

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
                  widget.title,
                  style: context.typography.barTitle,
                ),
              ),
              KansoPressable(
                onTap: () => Navigator.of(context).pop(),
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
          SizedBox(
            height: 180,
            child: CupertinoDatePicker(
              mode: CupertinoDatePickerMode.time,
              initialDateTime: widget.initial,
              use24hFormat: MediaQuery.alwaysUse24HourFormatOf(context),
              onDateTimeChanged: (v) => _value = v,
            ),
          ),
          const SizedBox(height: KansoSpacing.sm),
          KansoButton(
            label: l10n.noteListDone,
            variant: KansoButtonVariant.primary,
            onPressed: () => Navigator.of(
              context,
            ).pop(formatClockTime(_value.hour, _value.minute)),
          ),
        ],
      ),
    );
  }
}
