// Kanso — kanso_text_size.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// How large the text of notes is, relative to its designed size.
///
/// It scales note text only (the editor, or the rendered preview, which has
/// a size of its own); the interface is untouched. It is applied to the font
/// size and the device's own text scale (dynamic type) multiplies the result,
/// so the two add up and neither is ever clamped.
enum KansoTextSize {
  /// A little smaller than designed.
  small(0.9),

  /// The size the type scale was designed at.
  standard(1),

  /// A little larger.
  large(1.15),

  /// The largest.
  larger(1.3);

  const KansoTextSize(this.scale);

  /// The multiplier applied to a designed font size.
  final double scale;

  /// The size of a note's text for a designed [fontSize].
  double scaled(double fontSize) => fontSize * scale;

  /// The factor note text is finally drawn at once the device's own text
  /// scale [deviceScale] is applied on top.
  double withDeviceScale(double deviceScale) => deviceScale * scale;
}
