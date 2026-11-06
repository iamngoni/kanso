// Kanso — system_calendar_settings_page.dart
// Created 2026-11-06.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:go_router/go_router.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../overview/system_calendar.dart';

/// Settings › Calendar. This controls calendars already configured in iOS or
/// Android; Kanso does not sign in to external calendar services.
class SystemCalendarSettingsPage extends StatefulWidget {
  const SystemCalendarSettingsPage({super.key});

  @override
  State<SystemCalendarSettingsPage> createState() =>
      _SystemCalendarSettingsPageState();
}

class _SystemCalendarSettingsPageState
    extends State<SystemCalendarSettingsPage> {
  final SystemCalendarGateway _gateway = SystemCalendarGateway();
  List<SystemCalendar> _calendars = const <SystemCalendar>[];
  bool _loading = true;
  bool _hasAccess = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final calendars = await _gateway.calendars();
      if (!mounted) return;
      setState(() {
        _calendars = calendars;
        _hasAccess = true;
        _loading = false;
      });
    } on PlatformException {
      if (mounted) {
        setState(() {
          _hasAccess = false;
          _loading = false;
        });
      }
    }
  }

  Future<void> _connect() async {
    final strings = context.l10n;
    final allow = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(strings.calendarPermissionTitle),
        content: Text(strings.calendarPermissionCopy),
        actions: <Widget>[
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: Text(strings.cancel),
          ),
          TextButton(
            onPressed: () => Navigator.pop(context, true),
            child: Text(strings.calendarAllowAccess),
          ),
        ],
      ),
    );
    if (allow != true || !mounted) return;
    setState(() => _loading = true);
    try {
      final granted = await _gateway.requestAccess();
      if (!granted) {
        if (mounted) setState(() => _loading = false);
        return;
      }
      await _load();
    } on PlatformException {
      if (mounted) setState(() => _loading = false);
    }
  }

  Future<void> _toggle(SystemCalendar calendar, bool enabled) async {
    await _gateway.setCalendarEnabled(id: calendar.id, enabled: enabled);
    if (!mounted) return;
    setState(() {
      _calendars = _calendars
          .map(
            (it) => it.id == calendar.id
                ? SystemCalendar(
                    id: it.id,
                    name: it.name,
                    color: it.color,
                    enabled: enabled,
                  )
                : it,
          )
          .toList(growable: false);
    });
  }

  @override
  Widget build(BuildContext context) {
    final strings = context.l10n;
    final colors = context.colors;
    return KansoPage(
      backgroundColor: colors.app,
      body: Column(
        children: <Widget>[
          KansoTopBar(
            title: strings.settingsCalendar,
            backgroundColor: colors.app,
            leading: KansoIconButton(
              icon: KansoIcons.chevronLeft,
              semanticLabel: strings.settingsClose,
              onPressed: () => context.pop(),
            ),
          ),
          Expanded(
            child: ListView(
              padding: const EdgeInsets.all(KansoSpacing.md),
              children: <Widget>[
                if (_loading)
                  const Center(child: CircularProgressIndicator())
                else if (!_hasAccess)
                  KansoGroupedSection(
                    footer: strings.calendarPrivacyCopy,
                    children: <Widget>[
                      KansoSettingsRow(
                        icon: KansoIcons.calendarDays,
                        label: strings.calendarConnect,
                        chevron: true,
                        onTap: _connect,
                      ),
                    ],
                  )
                else if (_calendars.isEmpty)
                  KansoGroupedSection(
                    footer: strings.calendarNoCalendars,
                    children: const <Widget>[],
                  )
                else
                  KansoGroupedSection(
                    title: strings.calendarSources,
                    footer: strings.calendarPrivacyCopy,
                    children: <Widget>[
                      for (final calendar in _calendars)
                        KansoSettingsRow(
                          icon: KansoIcons.calendarDays,
                          label: calendar.name,
                          trailing: Switch.adaptive(
                            value: calendar.enabled,
                            onChanged: (enabled) => _toggle(calendar, enabled),
                          ),
                        ),
                    ],
                  ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
