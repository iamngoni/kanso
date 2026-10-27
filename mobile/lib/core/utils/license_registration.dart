// Kanso — license_registration.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

/// Adds the licences of the bundled fonts and icons to Flutter's licence
/// page (`showLicensePage`), as their terms require.
void registerBundledLicenses() {
  LicenseRegistry.addLicense(() async* {
    yield LicenseEntryWithLineBreaks(<String>[
      'Instrument Sans',
    ], await rootBundle.loadString('assets/fonts/OFL-InstrumentSans.txt'));
    yield LicenseEntryWithLineBreaks(<String>[
      'JetBrains Mono',
    ], await rootBundle.loadString('assets/fonts/OFL-JetBrainsMono.txt'));
    yield LicenseEntryWithLineBreaks(<String>[
      'Inter',
    ], await rootBundle.loadString('assets/fonts/OFL-Inter.txt'));
    yield LicenseEntryWithLineBreaks(<String>[
      'Source Serif 4',
    ], await rootBundle.loadString('assets/fonts/OFL-SourceSerif4.txt'));
    yield LicenseEntryWithLineBreaks(<String>[
      'Literata',
    ], await rootBundle.loadString('assets/fonts/OFL-Literata.txt'));
    yield LicenseEntryWithLineBreaks(<String>[
      'IBM Plex Mono',
    ], await rootBundle.loadString('assets/fonts/OFL-IBMPlexMono.txt'));
    yield LicenseEntryWithLineBreaks(<String>[
      'Lucide',
    ], await rootBundle.loadString('assets/icons/lucide/LICENSE'));
  });
}
