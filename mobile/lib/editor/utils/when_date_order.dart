// Kanso — when_date_order.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

/// Whether `11/14` is the 14th of November in [locale]: the regions that
/// write the month first. Everywhere else the day comes first.
bool monthFirstFor(Locale locale) =>
    _monthFirst.contains(locale.countryCode?.toUpperCase());

const Set<String> _monthFirst = <String>{'US', 'PH', 'FM', 'GU', 'MH', 'PW'};
