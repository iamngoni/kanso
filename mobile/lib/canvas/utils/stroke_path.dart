// Kanso — stroke_path.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

/// The filled path of a stroke from its outline polygon (`x0, y0, x1, y1,
/// ...`), its corners rounded with quadratic curves through the midpoints of
/// its edges, as the SVG export draws it. An outline of fewer than three
/// points makes an empty path.
Path outlinePath(List<double> outline, {Offset shift = Offset.zero}) {
  final count = outline.length ~/ 2;
  final path = Path();
  if (count < 3) {
    return path;
  }
  Offset at(int i) {
    final k = (i % count) * 2;
    return Offset(outline[k] + shift.dx, outline[k + 1] + shift.dy);
  }

  Offset mid(Offset a, Offset b) =>
      Offset((a.dx + b.dx) / 2, (a.dy + b.dy) / 2);
  final first = mid(at(0), at(1));
  path.moveTo(first.dx, first.dy);
  for (var i = 1; i <= count; i++) {
    final control = at(i);
    final end = mid(at(i), at(i + 1));
    path.quadraticBezierTo(control.dx, control.dy, end.dx, end.dy);
  }
  path.close();
  return path;
}
