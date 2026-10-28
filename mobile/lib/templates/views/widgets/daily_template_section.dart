// Kanso — daily_template_section.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../state/daily_template/daily_template_bloc.dart';
import 'template_picker_sheet.dart';

/// Settings > Daily notes: the template new daily notes start from, "None"
/// or a note of the Templates notebook, with the desktop's words under it.
/// It needs the `DailyTemplateBloc` above it.
class DailyTemplateSection extends StatelessWidget {
  const DailyTemplateSection({required this.dividerIndent, super.key});

  final double dividerIndent;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocBuilder<DailyTemplateBloc, DailyTemplateState>(
      builder: (context, state) {
        final chosen = state.chosen;
        final title = chosen == null
            ? l10n.dailyTemplateNone
            : chosen.title.trim().isEmpty
            ? l10n.noteUntitled
            : chosen.title;
        return KansoGroupedSection(
          title: l10n.settingsDailyNotes,
          footer: state.error != null && state.status == LoadStatus.ready
              ? l10n.dailyTemplateFailed
              : state.templates.isEmpty
              ? l10n.dailyTemplateDetailEmpty
              : l10n.dailyTemplateDetail('{{date}}', '{{time}}', '{{cursor}}'),
          dividerIndent: dividerIndent,
          children: <Widget>[
            KansoSettingsRow(
              icon: KansoIcons.calendar,
              label: l10n.settingsDailyTemplate,
              value: state.status == LoadStatus.loading ? null : title,
              chevron: true,
              onTap: state.status == LoadStatus.ready
                  ? () async {
                      final bloc = context.read<DailyTemplateBloc>();
                      final id = await TemplatePickerSheet.showDaily(
                        context,
                        current: state.current,
                      );
                      if (id != null) {
                        bloc.add(
                          DailyTemplateEvent.chosen(
                            id == TemplatePickerSheet.noTemplate ? null : id,
                          ),
                        );
                      }
                    }
                  : null,
            ),
          ],
        );
      },
    );
  }
}
