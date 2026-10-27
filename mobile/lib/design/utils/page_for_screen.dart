// Kanso — page_for_screen.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../models/data/design_layout.dart';
import '../models/data/design_screen.dart';
import '../views/pages/error_mock_page.dart';
import '../views/pages/tablet_mock_page.dart';

/// The page that draws [screen] for [layout].
Widget pageForScreen(DesignScreen screen, DesignLayout layout) {
  if (layout == DesignLayout.tablet) {
    return TabletMockPage(screen: screen);
  }
  return switch (screen) {
    DesignScreen.error => const ErrorMockPage(),
  };
}
