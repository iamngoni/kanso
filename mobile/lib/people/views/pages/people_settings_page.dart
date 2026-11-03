// Kanso — people_settings_page.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/library.dart';
import '../../../core/bridge/api/people_types.dart';
import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../core/views/widgets/kanso_section_header.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';
import '../../../core/views/widgets/kanso_switch.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../injection.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../models/repos/abs/people_repository.dart';

/// Settings › People & Projects, at `/settings/people`.
class PeopleSettingsPage extends StatefulWidget {
  const PeopleSettingsPage({super.key});

  @override
  State<PeopleSettingsPage> createState() => _PeopleSettingsPageState();
}

class _PeopleSettingsPageState extends State<PeopleSettingsPage> {
  final PeopleRepository _people = getIt<PeopleRepository>();
  PeopleSettings? _settings;
  List<String> _dismissed = const <String>[];
  List<NotebookRow> _books = const <NotebookRow>[];

  /// The refresh choices, in the order they cycle.
  static const List<String> _refreshModes = <String>['auto', 'open', 'press'];

  @override
  void initState() {
    super.initState();
    unawaited(_load());
  }

  Future<void> _load() async {
    final settings = await _people.settings();
    final dismissed = await _people.dismissed();
    var books = const <NotebookRow>[];
    try {
      books = (await getIt<KansoLibrary>().notebooks())
          .where((b) => !b.locked && b.role == null)
          .toList();
    } on Object {
      // No library to ask (a preview): the default notebook is all there is.
    }
    if (mounted) {
      setState(() {
        _books = books;
        _settings = settings.toOption().toNullable();
        _dismissed = dismissed.toOption().toNullable() ?? const <String>[];
      });
    }
  }

  Future<void> _change(PeopleSettings next) async {
    setState(() => _settings = next);
    final saved = await _people.setSettings(next);
    if (mounted) {
      setState(() => _settings = saved.toOption().toNullable() ?? next);
    }
  }

  Future<void> _suggestAgain(String name) async {
    await _people.suggestAgain(name);
    await _load();
  }

  /// The notebook pages are kept in: a sheet of the notebooks, with the
  /// default first.
  Future<void> _chooseBook(PeopleSettings s) async {
    final l10n = context.l10n;
    final picked = await showKansoBottomSheet<String>(
      context,
      builder: (sheet) => Padding(
        padding: const EdgeInsets.only(bottom: KansoSpacing.md),
        child: SingleChildScrollView(
          child: KansoSheetGroup(
            title: l10n.peopleSettingsKeepIn,
            children: <Widget>[
              KansoSettingsRow(
                label: l10n.peopleSettingsKeepInDefault,
                trailing: s.keepIn.isEmpty
                    ? const Icon(Icons.check, size: 18)
                    : null,
                onTap: () => Navigator.of(sheet).pop(''),
              ),
              for (final book in _books)
                KansoSettingsRow(
                  label: book.name,
                  trailing: s.keepIn == book.id
                      ? const Icon(Icons.check, size: 18)
                      : null,
                  onTap: () => Navigator.of(sheet).pop(book.id),
                ),
            ],
          ),
        ),
      ),
    );
    if (picked != null) {
      await _change(_copy(s, keepIn: picked));
    }
  }

  Widget _switch(
    KansoIcons icon,
    String label,
    bool value,
    ValueChanged<bool> onChanged,
  ) => KansoSettingsRow(
    icon: icon,
    label: label,
    trailing: KansoSwitch(value: value, onChanged: onChanged),
    onTap: () => onChanged(!value),
  );

  static PeopleSettings _copy(
    PeopleSettings s, {
    bool? suggest,
    bool? writeWithAi,
    String? refresh,
    String? keepIn,
    bool? detectWithAi,
  }) => PeopleSettings(
    suggest: suggest ?? s.suggest,
    keepIn: keepIn ?? s.keepIn,
    writeWithAi: writeWithAi ?? s.writeWithAi,
    refresh: refresh ?? s.refresh,
    detectWithAi: detectWithAi ?? s.detectWithAi,
  );

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final s = _settings;
    String refreshName(String mode) => switch (mode) {
      'open' => l10n.peopleRefreshOpen,
      'press' => l10n.peopleRefreshPress,
      _ => l10n.peopleRefreshAuto,
    };
    return KansoPage(
      backgroundColor: colors.app,
      body: ColoredBox(
        color: colors.app,
        child: Column(
          children: <Widget>[
            KansoTopBar(
              backgroundColor: colors.app,
              bordered: false,
              leading: const ShellBackButton(),
              title: l10n.peopleSettingsTitle,
            ),
            if (s != null)
              Expanded(
                child: ListView(
                  padding: const EdgeInsets.all(KansoSpacing.md),
                  children: <Widget>[
                    KansoGroupedSection(
                      dividerIndent: 48,
                      children: <Widget>[
                        KansoSettingsRow(
                          icon: KansoIcons.users,
                          label: l10n.peopleTitle,
                          chevron: true,
                          onTap: () => context.push(KansoRoutes.people),
                        ),
                      ],
                    ),
                    const SizedBox(height: KansoSpacing.lg),
                    KansoGroupedSection(
                      footer: l10n.peopleSettingsSuggestHelp,
                      dividerIndent: 48,
                      children: <Widget>[
                        _switch(
                          KansoIcons.users,
                          l10n.peopleSettingsSuggest,
                          s.suggest,
                          (v) => unawaited(_change(_copy(s, suggest: v))),
                        ),
                      ],
                    ),
                    const SizedBox(height: KansoSpacing.lg),
                    KansoGroupedSection(
                      footer: l10n.peopleSettingsKeepInHelp,
                      dividerIndent: 48,
                      children: <Widget>[
                        KansoSettingsRow(
                          icon: KansoIcons.folder,
                          label: l10n.peopleSettingsKeepIn,
                          value:
                              _books
                                  .where((b) => b.id == s.keepIn)
                                  .map((b) => b.name)
                                  .firstOrNull ??
                              l10n.peopleSettingsKeepInDefault,
                          chevron: true,
                          onTap: () => unawaited(_chooseBook(s)),
                        ),
                      ],
                    ),
                    const SizedBox(height: KansoSpacing.lg),
                    KansoGroupedSection(
                      footer: l10n.peopleSettingsWriteAiHelp,
                      dividerIndent: 48,
                      children: <Widget>[
                        _switch(
                          KansoIcons.sparkles,
                          l10n.peopleSettingsWriteAi,
                          s.writeWithAi,
                          (v) => unawaited(_change(_copy(s, writeWithAi: v))),
                        ),
                        KansoSettingsRow(
                          icon: KansoIcons.refreshCw,
                          label: l10n.peopleSettingsRefresh,
                          value: refreshName(s.refresh),
                          onTap: () {
                            final at = _refreshModes.indexOf(s.refresh);
                            final next =
                                _refreshModes[(at + 1) % _refreshModes.length];
                            unawaited(_change(_copy(s, refresh: next)));
                          },
                        ),
                      ],
                    ),
                    const SizedBox(height: KansoSpacing.lg),
                    KansoGroupedSection(
                      footer: l10n.peopleSettingsDetectAiHelp,
                      dividerIndent: 48,
                      children: <Widget>[
                        _switch(
                          KansoIcons.sparkles,
                          l10n.peopleSettingsDetectAi,
                          s.detectWithAi,
                          (v) => unawaited(_change(_copy(s, detectWithAi: v))),
                        ),
                      ],
                    ),
                    const SizedBox(height: KansoSpacing.lg),
                    KansoSectionHeader(
                      label: l10n.peopleSettingsDismissed,
                      horizontalPadding: 0,
                    ),
                    if (_dismissed.isEmpty)
                      Text(
                        l10n.peopleDismissedNone,
                        style: context.typography.secondary.copyWith(
                          color: colors.textMuted,
                        ),
                      )
                    else
                      KansoGroupedSection(
                        children: <Widget>[
                          for (final name in _dismissed)
                            KansoSettingsRow(
                              label: name,
                              trailing: KansoButton(
                                label: l10n.peopleSuggestAgain,
                                variant: KansoButtonVariant.quiet,
                                compact: true,
                                expanded: false,
                                onPressed: () => unawaited(_suggestAgain(name)),
                              ),
                            ),
                        ],
                      ),
                  ],
                ),
              ),
          ],
        ),
      ),
    );
  }
}
