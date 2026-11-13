// Kanso — property_moment_sheet.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/cupertino.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/clock_time.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/journal_time.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_month_grid.dart';
import '../../../core/views/widgets/kanso_pressable.dart';

/// The time a date-and-time value starts on when only a day is picked.
const int propertyDefaultHour = 9;

/// A calendar, and for a date and time a clock, in a bottom sheet: picks the
/// day (and the minute) of a date property. It pops with a [Moment] to
/// write, or with nothing when it is dismissed; Clear is [clearedMoment].
class PropertyMomentSheet extends StatefulWidget {
  const PropertyMomentSheet({
    required this.title,
    required this.withTime,
    required this.moment,
    this.firstWeekday = DateTime.monday,
    super.key,
  });

  final String title;

  /// Whether a time is picked with the day.
  final bool withTime;

  /// What the value holds, if it holds a day.
  final Moment? moment;

  final int firstWeekday;

  /// Shows the sheet over the whole app. Completes with the [Moment] picked,
  /// [clearedMoment] for Clear, or null.
  static Future<Moment?> show(
    BuildContext context, {
    required String title,
    required bool withTime,
    required Moment? moment,
  }) {
    final firstDay = firstDayOfWeekIndexFor(
      WidgetsBinding.instance.platformDispatcher.locale.countryCode,
    );
    return showKansoBottomSheet<Moment>(
      context,
      builder: (_) => PropertyMomentSheet(
        title: title,
        withTime: withTime,
        moment: moment,
        // Sunday is 0 for the journal's counting and 7 for DateTime's.
        firstWeekday: firstDay == 0 ? DateTime.sunday : firstDay,
      ),
    );
  }

  @override
  State<PropertyMomentSheet> createState() => _PropertyMomentSheetState();
}

/// What the sheet completes with for Clear: no year at all.
const Moment clearedMoment = Moment(
  year: 0,
  month: 0,
  day: 0,
  hour: 0,
  minute: 0,
);

class _PropertyMomentSheetState extends State<PropertyMomentSheet> {
  late DateTime? day = widget.moment == null
      ? null
      : DateTime(
          widget.moment!.year,
          widget.moment!.month,
          widget.moment!.day,
        );
  late int hour = widget.moment?.hour ?? propertyDefaultHour;
  late int minute = widget.moment?.minute ?? 0;
  bool clockOpen = false;

  Moment moment(DateTime of) => Moment(
    year: of.year,
    month: of.month,
    day: of.day,
    hour: widget.withTime ? hour : 0,
    minute: widget.withTime ? minute : 0,
  );

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final picked = day;
    return SingleChildScrollView(
      child: Padding(
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
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                if (widget.moment != null)
                  KansoPressable(
                    onTap: () => Navigator.of(context).pop(clearedMoment),
                    child: Padding(
                      padding: const EdgeInsets.all(KansoSpacing.xs),
                      child: Text(
                        l10n.propertiesClear,
                        style: context.typography.body.copyWith(
                          color: colors.accent,
                        ),
                      ),
                    ),
                  ),
              ],
            ),
            KansoMonthGrid(
              selected: picked,
              firstWeekday: widget.firstWeekday,
              onChanged: (value) => setState(() => day = value),
            ),
            if (widget.withTime) ...<Widget>[
              const SizedBox(height: KansoSpacing.xs),
              KansoPressable(
                onTap: () => setState(() => clockOpen = !clockOpen),
                semanticLabel: l10n.propertiesTime,
                child: Container(
                  height: KansoSizes.field,
                  padding: const EdgeInsets.symmetric(
                    horizontal: KansoSpacing.sm,
                  ),
                  decoration: BoxDecoration(
                    color: colors.elevated,
                    borderRadius: KansoRadii.mediumBorder,
                    border: Border.all(color: colors.divider),
                  ),
                  child: Row(
                    children: <Widget>[
                      KansoIcon(KansoIcons.clock, color: colors.textSecondary),
                      const SizedBox(width: KansoSpacing.xs),
                      Expanded(
                        child: Text(
                          l10n.propertiesTime,
                          style: context.typography.body,
                        ),
                      ),
                      Text(
                        formatClockTime(hour, minute),
                        style: context.typography.body.copyWith(
                          color: colors.accent,
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
              if (clockOpen)
                SizedBox(
                  height: 160,
                  // The wheel follows the app's theme, not the system's.
                  child: CupertinoTheme(
                    data: CupertinoThemeData(brightness: context.brightness),
                    child: CupertinoDatePicker(
                      mode: CupertinoDatePickerMode.time,
                      initialDateTime: DateTime(2000, 1, 1, hour, minute),
                      use24hFormat: true,
                      onDateTimeChanged: (value) => setState(() {
                        hour = value.hour;
                        minute = value.minute;
                      }),
                    ),
                  ),
                ),
            ],
            const SizedBox(height: KansoSpacing.sm),
            KansoButton(
              label: l10n.noteListDone,
              onPressed: picked == null
                  ? null
                  : () => Navigator.of(context).pop(moment(picked)),
            ),
          ],
        ),
      ),
    );
  }
}
