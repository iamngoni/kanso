// Kanso — template_picker_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_check_row.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/sheet_header.dart';
import '../../../injection.dart';
import '../../state/template_picker/template_picker_bloc.dart';

/// The templates to make a note from, or, with [current] given, to choose
/// the daily template from (then "None" is the first choice and the one in
/// use is checked).
///
/// Choosing one closes the sheet with its id, or with [noTemplate] for
/// "None"; closing it any other way completes with null.
class TemplatePickerSheet extends StatelessWidget {
  const TemplatePickerSheet({
    required this.title,
    this.daily = false,
    this.current,
    super.key,
  });

  /// What the sheet completes with for the choice "None".
  static const String noTemplate = '';

  final String title;

  /// Whether this chooses the daily template.
  final bool daily;

  /// The template in use, while choosing the daily template.
  final String? current;

  /// Shows the templates over the whole app, to make a note from.
  static Future<String?> show(BuildContext context) {
    return showKansoBottomSheet<String>(
      context,
      builder: (_) => BlocProvider<TemplatePickerBloc>(
        create: (_) =>
            getIt<TemplatePickerBloc>()
              ..add(const TemplatePickerEvent.started()),
        child: TemplatePickerSheet(title: context.l10n.templatePickerTitle),
      ),
    );
  }

  /// Shows the templates over the whole app, to choose the daily template
  /// from; [current] is the one in use.
  static Future<String?> showDaily(BuildContext context, {String? current}) {
    return showKansoBottomSheet<String>(
      context,
      builder: (_) => BlocProvider<TemplatePickerBloc>(
        create: (_) =>
            getIt<TemplatePickerBloc>()
              ..add(const TemplatePickerEvent.started()),
        child: TemplatePickerSheet(
          title: context.l10n.settingsDailyTemplate,
          daily: true,
          current: current,
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocBuilder<TemplatePickerBloc, TemplatePickerState>(
      builder: (context, state) {
        final children = <Widget>[
          SheetHeader(title: title, actionLabel: l10n.searchCancel),
        ];
        if (daily) {
          children.add(
            KansoCheckRow(
              label: l10n.dailyTemplateNone,
              checked: current == null,
              onTap: () => Navigator.of(context).pop(noTemplate),
            ),
          );
        }
        switch (state.status) {
          case LoadStatus.loading:
            break;
          case LoadStatus.failed:
            children.addAll(<Widget>[
              Padding(
                padding: const EdgeInsets.all(KansoSpacing.md),
                child: Text(
                  l10n.templatesFailed,
                  textAlign: TextAlign.center,
                  style: context.typography.secondary,
                ),
              ),
              Center(
                child: KansoButton(
                  label: l10n.retry,
                  expanded: false,
                  onPressed: () => context.read<TemplatePickerBloc>().add(
                    const TemplatePickerEvent.started(),
                  ),
                ),
              ),
            ]);
          case LoadStatus.ready when state.templates.isEmpty && !daily:
            children.add(
              KansoEmptyState(
                title: l10n.templatesEmptyTitle,
                message: l10n.templatesEmptyMessage,
              ),
            );
          case LoadStatus.ready:
            for (final TemplateRow row in state.templates) {
              final title = row.title.trim().isEmpty
                  ? l10n.noteUntitled
                  : row.title;
              children.add(
                KansoCheckRow(
                  label: title,
                  icon: KansoIcons.layoutTemplate,
                  checked: daily && current == row.id,
                  onTap: () => Navigator.of(context).pop(row.id),
                ),
              );
            }
        }
        return ListView(
          shrinkWrap: true,
          padding: const EdgeInsets.only(bottom: KansoSpacing.sm),
          children: children,
        );
      },
    );
  }
}
