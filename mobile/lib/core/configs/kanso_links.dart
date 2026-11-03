// Kanso — kanso_links.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// The addresses outside the app that it points to.
abstract final class KansoLinks {
  /// Kanso's website.
  static final Uri website = Uri.parse('https://kansoapp.app');

  /// What changed in each version of Kanso, on the website.
  static final Uri releaseNotes = Uri.parse('https://kansoapp.app/releases');

  /// Help with using Kanso, on the website.
  static final Uri help = Uri.parse('https://kansoapp.app/help');

  /// Bringing Apple Notes across: import on a Mac and sync, share notes one
  /// at a time, or choose exported files.
  /// A topic on the help page (site/src/routes/help.tsx, id `apple-notes`).
  static final Uri importAppleNotes = Uri.parse(
    'https://kansoapp.app/help#apple-notes',
  );

  /// Exactly what the anonymous usage count sends, and what it does not.
  static final Uri metrics = Uri.parse('https://kansoapp.app/metrics');

  /// The source and rules behind that count.
  static final Uri metricsCode = Uri.parse(
    'https://github.com/iamngoni/kanso/blob/master/docs/METRICS.md',
  );
}
