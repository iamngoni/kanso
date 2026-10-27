// Kanso — syntax_theme_id.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../state/appearance/appearance_bloc.dart';

/// The id of the theme the Rust core highlights a note's Markdown in: the
/// syntax theme chosen (the interface's own while there is none), for the
/// light or dark slot as [context]'s brightness says.
///
/// The caller builds again when the choice changes, so it can only be
/// called while building.
String syntaxThemeId(BuildContext context) {
  final appearance = context.watch<AppearanceBloc>().state;
  return appearance.syntaxThemeId(
    dark: Theme.brightnessOf(context) == Brightness.dark,
  );
}
