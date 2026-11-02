// Kanso — triage_settings_page.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';

import '../../../core/bridge/api/triage_types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_switch.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../injection.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../models/repos/abs/triage_repository.dart';

/// Settings › Inbox, at `/settings/inbox`: what Triage shows and suggests.
class TriageSettingsPage extends StatefulWidget {
  const TriageSettingsPage({super.key});

  @override
  State<TriageSettingsPage> createState() => _TriageSettingsPageState();
}

class _TriageSettingsPageState extends State<TriageSettingsPage> {
  final TriageRepository _triage = getIt<TriageRepository>();
  TriageSettings? _settings;

  /// The most tags a card may suggest, as the Rust core clamps it.
  static const int _maxTagsLimit = 5;

  @override
  void initState() {
    super.initState();
    unawaited(_load());
  }

  Future<void> _load() async {
    final read = await _triage.settings();
    if (mounted) {
      setState(() => _settings = read.toOption().toNullable());
    }
  }

  Future<void> _change(TriageSettings next) async {
    setState(() => _settings = next);
    final saved = await _triage.setSettings(next);
    if (mounted) {
      setState(() => _settings = saved.toOption().toNullable() ?? next);
    }
  }

  Widget _switch(
    IconData? unused,
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

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final s = _settings;
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
              title: l10n.triageSettingsTitle,
            ),
            if (s != null)
              Expanded(
                child: ListView(
                  padding: const EdgeInsets.all(KansoSpacing.md),
                  children: <Widget>[
                    KansoGroupedSection(
                      footer: l10n.triageSettingsShowBarHelp,
                      dividerIndent: 48,
                      children: <Widget>[
                        _switch(
                          null,
                          KansoIcons.inbox,
                          l10n.triageSettingsShowBar,
                          s.showBar,
                          (v) => _change(_copy(s, showBar: v)),
                        ),
                      ],
                    ),
                    const SizedBox(height: KansoSpacing.lg),
                    KansoGroupedSection(
                      footer: l10n.triageSettingsTitlesHelp,
                      dividerIndent: 48,
                      children: <Widget>[
                        _switch(
                          null,
                          KansoIcons.typeface,
                          l10n.triageSettingsTitles,
                          s.suggestTitles,
                          (v) => _change(_copy(s, suggestTitles: v)),
                        ),
                      ],
                    ),
                    const SizedBox(height: KansoSpacing.lg),
                    KansoGroupedSection(
                      footer: l10n.triageSettingsLinksHelp,
                      dividerIndent: 48,
                      children: <Widget>[
                        _switch(
                          null,
                          KansoIcons.hash,
                          l10n.triageSettingsLinks,
                          s.suggestLinks,
                          (v) => _change(_copy(s, suggestLinks: v)),
                        ),
                      ],
                    ),
                    const SizedBox(height: KansoSpacing.lg),
                    KansoGroupedSection(
                      dividerIndent: 48,
                      children: <Widget>[
                        KansoSettingsRow(
                          icon: KansoIcons.hash,
                          label: l10n.triageSettingsMaxTags,
                          value: '${s.maxTags}',
                          onTap: () => _change(
                            _copy(
                              s,
                              maxTags: s.maxTags >= _maxTagsLimit
                                  ? 1
                                  : s.maxTags + 1,
                            ),
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: KansoSpacing.lg),
                    KansoGroupedSection(
                      footer: l10n.triageSettingsUseAiHelp,
                      dividerIndent: 48,
                      children: <Widget>[
                        _switch(
                          null,
                          KansoIcons.sparkles,
                          l10n.triageSettingsUseAi,
                          s.useAi,
                          (v) => _change(_copy(s, useAi: v)),
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

  static TriageSettings _copy(
    TriageSettings s, {
    bool? showBar,
    bool? suggestTitles,
    bool? suggestLinks,
    int? maxTags,
    bool? useAi,
  }) => TriageSettings(
    showBar: showBar ?? s.showBar,
    suggestTitles: suggestTitles ?? s.suggestTitles,
    suggestLinks: suggestLinks ?? s.suggestLinks,
    maxTags: maxTags ?? s.maxTags,
    useAi: useAi ?? s.useAi,
  );
}
