// Kanso — superscript.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// [n] written in superscript digits: `12` is `¹²`.
String superscript(int n) {
  const digits = <String>['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
  return <String>[
    for (final unit in n.toString().codeUnits)
      if (unit >= 0x30 && unit <= 0x39) digits[unit - 0x30],
  ].join();
}
