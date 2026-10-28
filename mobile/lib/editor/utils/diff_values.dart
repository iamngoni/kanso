// Kanso — diff_values.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';

import '../models/data/text_diff.dart';

/// What the person or the input method changed, going from [before] to
/// [after]: the smallest range of [before] that [after] replaces.
///
/// Where the change could be placed in more than one way (an "a" typed
/// beside another "a"), the new caret in [after] decides: the change ends
/// where the caret is, as typing does. A range never starts or ends between
/// the two halves of a surrogate pair, in either text.
TextDiff diffValues(TextEditingValue before, TextEditingValue after) {
  final a = before.text;
  final b = after.text;
  final shortest = a.length < b.length ? a.length : b.length;
  var prefix = 0;
  while (prefix < shortest && a.codeUnitAt(prefix) == b.codeUnitAt(prefix)) {
    prefix++;
  }
  var suffix = 0;
  final room = shortest - prefix;
  while (suffix < room &&
      a.codeUnitAt(a.length - 1 - suffix) ==
          b.codeUnitAt(b.length - 1 - suffix)) {
    suffix++;
  }

  // Slide the change to the caret: it ends there when it can.
  final selection = after.selection;
  if (selection.isValid && selection.isCollapsed) {
    final caret = selection.baseOffset;
    final wanted = b.length - caret;
    if (caret >= 0 && wanted > suffix && wanted <= a.length) {
      final tail = b.substring(caret);
      if (a.endsWith(tail)) {
        suffix = wanted;
        final reach = a.length - suffix;
        if (prefix > reach) {
          prefix = reach;
        }
        if (prefix > caret) {
          prefix = caret;
        }
      }
    }
  }

  // Never cut a surrogate pair in two.
  while (prefix > 0 && (splitsPair(a, prefix) || splitsPair(b, prefix))) {
    prefix--;
  }
  while (suffix > 0 &&
      (splitsPair(a, a.length - suffix) || splitsPair(b, b.length - suffix))) {
    suffix--;
  }
  return TextDiff(
    start: prefix,
    end: a.length - suffix,
    text: b.substring(prefix, b.length - suffix),
  );
}

/// Whether [at] is between the two halves of a surrogate pair in [text].
bool splitsPair(String text, int at) {
  if (at <= 0 || at >= text.length) {
    return false;
  }
  final before = text.codeUnitAt(at - 1);
  final after = text.codeUnitAt(at);
  return before >= 0xD800 &&
      before <= 0xDBFF &&
      after >= 0xDC00 &&
      after <= 0xDFFF;
}
