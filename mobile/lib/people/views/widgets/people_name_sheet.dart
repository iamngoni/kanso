// Kanso — people_name_sheet.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/people_types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';
import '../../../core/views/widgets/kanso_text_input.dart';
import '../../../library/views/widgets/library_text_prompt_sheet.dart';

/// A name and the other names it goes by.
typedef PeopleNameAnswer = ({String name, List<String> aliases});

/// The "New person" / "New project" sheet: a name and, optionally, other
/// names ("Sam, Samuel"), which become the page's aliases.
class PeopleNameSheet extends StatefulWidget {
  const PeopleNameSheet({required this.kind, super.key});

  final PeopleKind kind;

  static Future<PeopleNameAnswer?> show(
    BuildContext context,
    PeopleKind kind,
  ) => showKansoBottomSheet<PeopleNameAnswer>(
    context,
    builder: (_) => PeopleNameSheet(kind: kind),
  );

  /// Asks for one more name for a page.
  static Future<String?> alias(BuildContext context) =>
      LibraryTextPromptSheet.show(
        context,
        title: context.l10n.peopleAddAliasTitle,
        confirmLabel: context.l10n.peopleAddAlias,
        hint: context.l10n.peopleNameHint,
      );

  /// Lets the person pick one of [pages] to merge with.
  static Future<PeoplePage?> pick(
    BuildContext context,
    List<PeoplePage> pages,
  ) => showKansoBottomSheet<PeoplePage>(
    context,
    builder: (sheet) => Padding(
      padding: const EdgeInsets.only(bottom: KansoSpacing.md),
      child: SingleChildScrollView(
        child: KansoSheetGroup(
          title: context.l10n.peopleMergeTitle,
          children: <Widget>[
            for (final p in pages)
              KansoSettingsRow(
                label: p.title,
                value: p.detail,
                onTap: () => Navigator.of(sheet).pop(p),
              ),
          ],
        ),
      ),
    ),
  );

  @override
  State<PeopleNameSheet> createState() => _PeopleNameSheetState();
}

class _PeopleNameSheetState extends State<PeopleNameSheet> {
  final TextEditingController _name = TextEditingController();
  final TextEditingController _also = TextEditingController();

  @override
  void dispose() {
    _name.dispose();
    _also.dispose();
    super.dispose();
  }

  void _submit() {
    final name = _name.text.trim();
    if (name.isEmpty) {
      return;
    }
    final aliases = _also.text
        .split(',')
        .map((a) => a.trim())
        .where((a) => a.isNotEmpty)
        .toList();
    Navigator.of(context).pop((name: name, aliases: aliases));
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final title = widget.kind == PeopleKind.person
        ? l10n.peopleNewPersonTitle
        : l10n.peopleNewProjectTitle;
    return Padding(
      padding: const EdgeInsets.fromLTRB(
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
                child: Text(title, style: context.typography.barTitle),
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
          KansoTextInput(
            controller: _name,
            hint: l10n.peopleNameHint,
            semanticLabel: l10n.peopleNameHint,
            autofocus: true,
            textCapitalization: TextCapitalization.words,
            textInputAction: TextInputAction.next,
          ),
          const SizedBox(height: KansoSpacing.xs),
          KansoTextInput(
            controller: _also,
            hint: l10n.peopleAlsoKnownAsHint,
            semanticLabel: l10n.peopleAlsoKnownAsHint,
            textCapitalization: TextCapitalization.words,
            onSubmitted: (_) => _submit(),
          ),
          const SizedBox(height: KansoSpacing.sm),
          ListenableBuilder(
            listenable: _name,
            builder: (context, _) => KansoButton(
              label: l10n.peopleCreate,
              onPressed: _name.text.trim().isEmpty ? null : _submit,
            ),
          ),
        ],
      ),
    );
  }
}
