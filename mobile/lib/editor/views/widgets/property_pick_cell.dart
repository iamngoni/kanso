// Kanso — property_pick_cell.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:intl/intl.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/clock_time.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import 'property_choice_sheet.dart';
import 'property_moment_sheet.dart';

/// How a time zone reads to a person: the city or region without its area
/// and underscores (`America/New_York` is `New York`), as the core's
/// `zone_label` says it.
String propertyZoneLabel(String name) =>
    name.split('/').last.replaceAll('_', ' ');

/// The editor of a property row that has one besides a text field: a select
/// or a person (a list), a date (a calendar), a date and time (a calendar
/// and a clock) and, for a date and time with a zone, a second cell for the
/// zone. Each cell shows the value and opens its sheet when tapped; what is
/// picked is passed to [onPick], and nothing is written here.
class PropertyPickCell extends StatefulWidget {
  const PropertyPickCell({
    required this.property,
    required this.pick,
    required this.onPick,
    required this.loadZones,
    super.key,
  });

  final Property property;
  final PropertyPick pick;
  final ValueChanged<PropertyChoice> onPick;

  /// Every time zone, for the zone sheet.
  final Future<List<ZoneRow>> Function() loadZones;

  @override
  State<PropertyPickCell> createState() => _PropertyPickCellState();
}

class _PropertyPickCellState extends State<PropertyPickCell> {
  /// The zone picked while the value has no time to put in it yet: it waits
  /// here until a time is picked.
  String? waiting;

  String? get zone =>
      widget.pick.zoneSet ? widget.pick.zone : waiting ?? widget.pick.zone;

  Future<void> openList() async {
    final l10n = context.l10n;
    final pick = widget.pick;
    final text = widget.property.text;
    final chosen = await PropertyChoiceSheet.show(
      context,
      title: widget.property.key,
      rows: <PropertyChoiceRow>[
        for (final row in pick.rows) PropertyChoiceRow(row, row),
      ],
      selected: text.isEmpty ? null : text,
      clearLabel: l10n.propertiesClear,
      hint: pick.unlisted ? l10n.propertiesUnlisted : null,
    );
    if (chosen == null) {
      return;
    }
    widget.onPick(
      chosen == propertyChoiceCleared
          ? const PropertyChoice.clear()
          : PropertyChoice.option(text: chosen),
    );
  }

  Future<void> openMoment() async {
    final pick = widget.pick;
    final withTime = pick.kind != PickKind.date;
    final picked = await PropertyMomentSheet.show(
      context,
      title: widget.property.key,
      withTime: withTime,
      moment: pick.moment,
    );
    if (picked == null) {
      return;
    }
    if (picked == clearedMoment) {
      widget.onPick(const PropertyChoice.clear());
      return;
    }
    final inZone = zone;
    widget.onPick(switch (pick.kind) {
      PickKind.date => PropertyChoice.day(moment: picked),
      PickKind.dateTimeZone when inZone != null => PropertyChoice.zoned(
        moment: picked,
        zone: inZone,
      ),
      _ => PropertyChoice.moment(moment: picked),
    });
  }

  Future<void> openZone() async {
    final l10n = context.l10n;
    final zones = await widget.loadZones();
    if (!mounted) {
      return;
    }
    final chosen = await PropertyChoiceSheet.show(
      context,
      title: l10n.propertiesPickZone,
      rows: <PropertyChoiceRow>[
        for (final row in zones) PropertyChoiceRow(row.name, row.label),
      ],
      selected: zone,
      searchHint: l10n.propertiesSearchZones,
      emptyLabel: l10n.propertiesNoZones,
    );
    if (chosen == null || chosen == propertyChoiceCleared) {
      return;
    }
    if (widget.pick.moment == null) {
      // No time to keep yet: the zone waits for one.
      setState(() => waiting = chosen);
      return;
    }
    widget.onPick(PropertyChoice.zone(zone: chosen));
  }

  /// The value of the day and time cells, or null while there is none.
  String? momentText(BuildContext context) {
    final at = widget.pick.moment;
    if (at == null) {
      return null;
    }
    final day = DateFormat.yMMMEd(
      context.l10n.localeName,
    ).format(DateTime(at.year, at.month, at.day));
    return widget.pick.kind == PickKind.date
        ? day
        : '$day · ${formatClockTime(at.hour, at.minute)}';
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final pick = widget.pick;
    final key = widget.property.key;
    final text = widget.property.text;
    switch (pick.kind) {
      case PickKind.select:
      case PickKind.person:
        return _Cell(
          semanticLabel: key,
          icon: KansoIcons.chevronDown,
          value: text.isEmpty ? null : text,
          placeholder: l10n.propertiesChoose,
          warn: pick.unlisted,
          onTap: openList,
        );
      case PickKind.date:
      case PickKind.dateTime:
      case PickKind.dateTimeZone:
        // A value that does not fit is shown as written; the picker starts
        // empty.
        final shown = momentText(context) ?? (text.isEmpty ? null : text);
        final moment = _Cell(
          semanticLabel: pick.label == null ? key : '$key, ${pick.label}',
          icon: pick.kind == PickKind.date
              ? KansoIcons.calendarDays
              : KansoIcons.clock,
          value: shown,
          placeholder: pick.kind == PickKind.date
              ? l10n.propertiesPickDate
              : l10n.propertiesPickDateTime,
          onTap: openMoment,
        );
        if (pick.kind != PickKind.dateTimeZone) {
          return moment;
        }
        final inZone = zone;
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            moment,
            const SizedBox(height: KansoSpacing.xxs),
            _Cell(
              semanticLabel: '$key, ${l10n.propertiesPickZone}',
              icon: KansoIcons.globe,
              value: inZone == null ? null : propertyZoneLabel(inZone),
              placeholder: l10n.propertiesPickZone,
              quiet: !pick.zoneSet && waiting == null,
              onTap: openZone,
            ),
          ],
        );
    }
  }
}

/// A tappable value: a filled field with what it holds (or a hint) and the
/// glyph of what opens.
class _Cell extends StatelessWidget {
  const _Cell({
    required this.semanticLabel,
    required this.icon,
    required this.value,
    required this.placeholder,
    required this.onTap,
    this.warn = false,
    this.quiet = false,
  });

  final String semanticLabel;
  final KansoIcons icon;
  final String? value;
  final String placeholder;
  final VoidCallback onTap;

  /// The value is one the notebook no longer lists.
  final bool warn;

  /// The value is a suggestion (the device's zone), not the note's.
  final bool quiet;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final shown = value;
    final ink = shown == null
        ? colors.textMuted
        : warn
        ? colors.warning
        : quiet
        ? colors.textSecondary
        : colors.text;
    return KansoPressable(
      onTap: onTap,
      semanticLabel: semanticLabel,
      child: Container(
        constraints: const BoxConstraints(minHeight: KansoSizes.searchField),
        padding: const EdgeInsets.symmetric(
          horizontal: KansoSpacing.sm,
          vertical: KansoSpacing.xs,
        ),
        decoration: BoxDecoration(
          color: colors.elevated,
          borderRadius: KansoRadii.mediumBorder,
          border: Border.all(color: warn ? colors.warning : colors.divider),
        ),
        child: ExcludeSemantics(
          child: Row(
            children: <Widget>[
              Expanded(
                child: Text(
                  shown ?? placeholder,
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                  style: context.typography.body.copyWith(color: ink),
                ),
              ),
              const SizedBox(width: KansoSpacing.xs),
              KansoIcon(
                icon,
                size: KansoSizes.iconSmall,
                color: colors.textSecondary,
              ),
            ],
          ),
        ),
      ),
    );
  }
}
