// Kanso — property_choice_sheet.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_check_row.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_search_input.dart';

/// One row of a [PropertyChoiceSheet]: what is written ([value]) and what
/// is read ([label]).
class PropertyChoiceRow {
  const PropertyChoiceRow(this.value, this.label);

  final String value;
  final String label;
}

/// What the sheet pops with to say the value is emptied.
const String propertyChoiceCleared = '';

/// A list to pick one row from, in a bottom sheet: the options of a select,
/// the people of a shared note or the time zones. Over [searchAbove] rows it
/// has a search field. It pops with the [PropertyChoiceRow.value] picked,
/// [propertyChoiceCleared] for Clear (offered when it has a [clearLabel] and
/// a row is in use), or nothing when it is dismissed.
class PropertyChoiceSheet extends StatefulWidget {
  const PropertyChoiceSheet({
    required this.title,
    required this.rows,
    required this.selected,
    this.clearLabel,
    this.searchHint,
    this.emptyLabel,
    this.hint,
    super.key,
  });

  /// More rows than this have a search field.
  static const int searchAbove = 8;

  final String title;
  final List<PropertyChoiceRow> rows;

  /// The value of the row in use, if any.
  final String? selected;

  /// The label of the Clear button; with none there is no such button.
  final String? clearLabel;

  final String? searchHint;
  final String? emptyLabel;

  /// A line under the title: what is off about the value in use.
  final String? hint;

  /// Shows the sheet over the whole app.
  static Future<String?> show(
    BuildContext context, {
    required String title,
    required List<PropertyChoiceRow> rows,
    required String? selected,
    String? clearLabel,
    String? searchHint,
    String? emptyLabel,
    String? hint,
  }) => showKansoBottomSheet<String>(
    context,
    restingFraction: rows.length > searchAbove ? 0.8 : null,
    builder: (_) => PropertyChoiceSheet(
      title: title,
      rows: rows,
      selected: selected,
      clearLabel: clearLabel,
      searchHint: searchHint,
      emptyLabel: emptyLabel,
      hint: hint,
    ),
  );

  @override
  State<PropertyChoiceSheet> createState() => _PropertyChoiceSheetState();
}

class _PropertyChoiceSheetState extends State<PropertyChoiceSheet> {
  String query = '';

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final needle = query.trim().toLowerCase();
    final shown = needle.isEmpty
        ? widget.rows
        : <PropertyChoiceRow>[
            for (final row in widget.rows)
              if (row.label.toLowerCase().contains(needle) ||
                  row.value.toLowerCase().contains(needle))
                row,
          ];
    final searchable = widget.rows.length > PropertyChoiceSheet.searchAbove;
    final clearLabel = widget.clearLabel;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Padding(
          padding: const EdgeInsetsDirectional.fromSTEB(
            KansoSpacing.md,
            KansoSpacing.sm,
            KansoSpacing.md,
            KansoSpacing.xs,
          ),
          child: Row(
            children: <Widget>[
              Expanded(
                child: Text(
                  widget.title,
                  style: context.typography.barTitle,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              if (clearLabel != null && widget.selected != null)
                KansoPressable(
                  onTap: () => Navigator.of(context).pop(propertyChoiceCleared),
                  child: Padding(
                    padding: const EdgeInsets.all(KansoSpacing.xs),
                    child: Text(
                      clearLabel,
                      style: context.typography.body.copyWith(
                        color: colors.accent,
                      ),
                    ),
                  ),
                ),
            ],
          ),
        ),
        if (widget.hint case final hint?)
          Padding(
            padding: const EdgeInsetsDirectional.fromSTEB(
              KansoSpacing.md,
              0,
              KansoSpacing.md,
              KansoSpacing.xs,
            ),
            child: Text(
              hint,
              style: context.typography.meta.copyWith(color: colors.warning),
            ),
          ),
        if (searchable)
          Padding(
            padding: const EdgeInsets.fromLTRB(
              KansoSpacing.md,
              0,
              KansoSpacing.md,
              KansoSpacing.xs,
            ),
            child: KansoSearchInput(
              hint: widget.searchHint ?? '',
              onChanged: (value) => setState(() => query = value),
            ),
          ),
        Flexible(
          child: ListView(
            shrinkWrap: true,
            padding: const EdgeInsets.fromLTRB(
              KansoSpacing.md,
              KansoSpacing.xs,
              KansoSpacing.md,
              KansoSpacing.md,
            ),
            children: <Widget>[
              if (shown.isEmpty)
                Padding(
                  padding: const EdgeInsets.all(KansoSpacing.md),
                  child: Text(
                    widget.emptyLabel ?? '',
                    textAlign: TextAlign.center,
                    style: context.typography.secondary,
                  ),
                )
              else
                KansoGroupedSection(
                  children: <Widget>[
                    for (final row in shown)
                      KansoCheckRow(
                        label: row.label,
                        checked: row.value == widget.selected,
                        onTap: () => Navigator.of(context).pop(row.value),
                      ),
                  ],
                ),
            ],
          ),
        ),
      ],
    );
  }
}
