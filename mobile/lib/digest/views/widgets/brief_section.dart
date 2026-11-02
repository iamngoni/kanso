// Kanso — brief_section.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/repos/abs/digest_repository.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_menu.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_switch.dart';
import '../../../injection.dart';
import '../../../reminders/utils/reminders_coordinator.dart';

/// Settings > Brief and review: the open loops detector, the morning brief
/// and the weekly review, when they notify, and whether a done loop is
/// ticked in its note. Where no repository is registered (a widget test of
/// the pane) the section is left out.
class BriefSection extends StatefulWidget {
  const BriefSection({required this.dividerIndent, super.key});

  final double dividerIndent;

  @override
  State<BriefSection> createState() => _BriefSectionState();
}

/// `minutes` of the day moved by [delta], wrapping around midnight.
(int, int) stepTime(int hour, int minute, int delta) {
  final total = (hour * 60 + minute + delta) % (24 * 60);
  final wrapped = total < 0 ? total + 24 * 60 : total;
  return (wrapped ~/ 60, wrapped % 60);
}

String _clock(int hour, int minute) =>
    '${hour.toString().padLeft(2, '0')}:${minute.toString().padLeft(2, '0')}';

class _BriefSectionState extends State<BriefSection> {
  BriefPrefs? prefs;

  @override
  void initState() {
    super.initState();
    unawaited(load());
  }

  Future<void> load() async {
    if (!getIt.isRegistered<DigestRepository>()) {
      return;
    }
    final result = await getIt<DigestRepository>().prefs();
    if (!mounted) {
      return;
    }
    result.fold((_) {}, (it) => setState(() => prefs = it));
  }

  Future<void> change(BriefPrefs next) async {
    setState(() => prefs = next);
    final result = await getIt<DigestRepository>().setPrefs(next);
    if (!mounted) {
      return;
    }
    result.fold((_) {}, (kept) => setState(() => prefs = kept));
    // The notifications are planned again for the new times.
    if (getIt.isRegistered<RemindersCoordinator>()) {
      unawaited(getIt<RemindersCoordinator>().refresh());
    }
  }

  BriefPrefs copy(
    BriefPrefs p, {
    bool? loops,
    bool? brief,
    int? briefHour,
    int? briefMinute,
    bool? briefWeekdaysOnly,
    bool? review,
    int? reviewDay,
    int? reviewHour,
    int? reviewMinute,
    bool? markDoneInNote,
  }) => BriefPrefs(
    loops: loops ?? p.loops,
    brief: brief ?? p.brief,
    briefHour: briefHour ?? p.briefHour,
    briefMinute: briefMinute ?? p.briefMinute,
    briefWeekdaysOnly: briefWeekdaysOnly ?? p.briefWeekdaysOnly,
    review: review ?? p.review,
    reviewDay: reviewDay ?? p.reviewDay,
    reviewHour: reviewHour ?? p.reviewHour,
    reviewMinute: reviewMinute ?? p.reviewMinute,
    notifyBrief: p.notifyBrief,
    notifyReview: p.notifyReview,
    markDoneInNote: markDoneInNote ?? p.markDoneInNote,
  );

  Widget stepper(
    BuildContext context, {
    required int hour,
    required int minute,
    required void Function(int hour, int minute) onChanged,
  }) {
    final l10n = context.l10n;
    final colors = context.colors;
    Widget button(KansoIcons icon, String label, int delta) => KansoIconButton(
      icon: icon,
      semanticLabel: label,
      onPressed: () {
        final (h, m) = stepTime(hour, minute, delta);
        onChanged(h, m);
      },
    );
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: <Widget>[
        button(KansoIcons.chevronLeft, l10n.settingsBriefEarlier, -15),
        SizedBox(
          width: 52,
          child: Text(
            _clock(hour, minute),
            textAlign: TextAlign.center,
            style: context.typography.body.copyWith(color: colors.text),
          ),
        ),
        button(KansoIcons.chevronRight, l10n.settingsBriefLater, 15),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final p = prefs;
    if (p == null) {
      return const SizedBox.shrink();
    }
    final days = <String>[
      l10n.weekdayMonday,
      l10n.weekdayTuesday,
      l10n.weekdayWednesday,
      l10n.weekdayThursday,
      l10n.weekdayFriday,
      l10n.weekdaySaturday,
      l10n.weekdaySunday,
    ];
    return Padding(
      padding: const EdgeInsets.only(bottom: KansoSpacing.lg),
      child: KansoGroupedSection(
        title: l10n.settingsBrief,
        footer: l10n.settingsBriefDetail,
        dividerIndent: widget.dividerIndent,
        children: <Widget>[
          KansoSettingsRow(
            icon: KansoIcons.circleHelp,
            label: l10n.settingsBriefLoops,
            onTap: () => unawaited(change(copy(p, loops: !p.loops))),
            trailing: KansoSwitch(
              value: p.loops,
              onChanged: (on) => unawaited(change(copy(p, loops: on))),
            ),
          ),
          KansoSettingsRow(
            icon: KansoIcons.clock,
            label: l10n.settingsBriefMorning,
            onTap: () => unawaited(change(copy(p, brief: !p.brief))),
            trailing: KansoSwitch(
              value: p.brief,
              onChanged: (on) => unawaited(change(copy(p, brief: on))),
            ),
          ),
          if (p.brief) ...<Widget>[
            KansoSettingsRow(
              label: l10n.settingsBriefTime,
              trailing: stepper(
                context,
                hour: p.briefHour,
                minute: p.briefMinute,
                onChanged: (h, m) => unawaited(
                  change(copy(p, briefHour: h, briefMinute: m)),
                ),
              ),
            ),
            KansoSettingsRow(
              label: l10n.settingsBriefMorningWeekdays,
              onTap: () => unawaited(
                change(copy(p, briefWeekdaysOnly: !p.briefWeekdaysOnly)),
              ),
              trailing: KansoSwitch(
                value: p.briefWeekdaysOnly,
                onChanged: (on) =>
                    unawaited(change(copy(p, briefWeekdaysOnly: on))),
              ),
            ),
          ],
          KansoSettingsRow(
            icon: KansoIcons.calendarDays,
            label: l10n.settingsBriefReview,
            onTap: () => unawaited(change(copy(p, review: !p.review))),
            trailing: KansoSwitch(
              value: p.review,
              onChanged: (on) => unawaited(change(copy(p, review: on))),
            ),
          ),
          if (p.review) ...<Widget>[
            Builder(
              builder: (context) => KansoSettingsRow(
                label: l10n.settingsBriefDay,
                value: days[p.reviewDay.clamp(0, 6)],
                chevron: true,
                onTap: () async {
                  final box = context.findRenderObject() as RenderBox?;
                  final at = box == null
                      ? Offset.zero
                      : box.localToGlobal(
                          Offset(box.size.width - 180, box.size.height),
                        );
                  final chosen = await showKansoMenu<int>(
                    context,
                    at,
                    <KansoMenuEntry<int>>[
                      for (var i = 0; i < days.length; i++)
                        KansoMenuEntry<int>(
                          i,
                          days[i],
                          checked: i == p.reviewDay,
                        ),
                    ],
                  );
                  if (chosen != null) {
                    unawaited(change(copy(p, reviewDay: chosen)));
                  }
                },
              ),
            ),
            KansoSettingsRow(
              label: l10n.settingsBriefTime,
              trailing: stepper(
                context,
                hour: p.reviewHour,
                minute: p.reviewMinute,
                onChanged: (h, m) => unawaited(
                  change(copy(p, reviewHour: h, reviewMinute: m)),
                ),
              ),
            ),
          ],
          KansoSettingsRow(
            icon: KansoIcons.check,
            label: l10n.settingsBriefMarkDone,
            onTap: () =>
                unawaited(change(copy(p, markDoneInNote: !p.markDoneInNote))),
            trailing: KansoSwitch(
              value: p.markDoneInNote,
              onChanged: (on) => unawaited(change(copy(p, markDoneInNote: on))),
            ),
          ),
        ],
      ),
    );
  }
}
