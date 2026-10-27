// Kanso — long_note.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// A long note of at least [words] words, generated: headings, paragraphs,
/// lists, tasks, tables, fenced code, links, wiki-links, a paragraph of CJK
/// and emoji, and one broken table three quarters of the way down, which is
/// a syntax error that makes the highlighter parse all that follows it as
/// one block. The same words every time.
///
/// The blocks are those of `long_note` in the Rust tests
/// (`crates/kanso-mobile/tests/common/mod.rs`), with the broken table added
/// and two tags instead of seven, which a note row on a phone cannot show.
String longNote(int words) {
  final out = StringBuffer();
  var written = 0;
  var n = 0;
  var broken = false;
  while (written < words) {
    final String block;
    if (!broken && written >= words * 3 ~/ 4) {
      broken = true;
      block =
          '|\n 日本語 |\n|:-\n\n\n!A A0-|:--|\n| **a** | [b](c) |\n\n\n'
          'after the broken table\n\n';
    } else {
      block = switch (n % 9) {
        0 => '## Section $n\n\n',
        1 =>
          'Paragraph $n has some *emphasis*, some **strong** words, `code` '
              'and a [link](https://example.com/$n) in it, then '
              '${'lorem ipsum dolor sit amet ' * 4} more plain words after '
              'those.\n\n',
        2 => '- item $n one\n- item $n two\n  - nested $n\n- item $n three\n\n',
        3 => '- [ ] task $n open\n- [x] task $n done\n\n',
        4 => '| a $n | b |\n| --- | --- |\n| one | two |\n| three | four |\n\n',
        5 => '```rust\nfn block_$n() -> u32 {\n    $n\n}\n```\n\n',
        6 => 'See [[Note $n]] and #tag${n % 2} for more.\n\n',
        7 => '日本語の段落 $n です。絵文字 🙂🎉 も入っています。\n\n',
        _ => '> quoted $n line one\n> quoted $n line two\n\n',
      };
      n++;
    }
    written += block.trim().split(RegExp(r'\s+')).length;
    out.write(block);
  }
  return out.toString();
}

/// The note the gate types into: fifty thousand words.
String fiftyThousandWordNote() => longNote(50000);

/// A short, stable fingerprint of [text] (FNV-1a, 64 bits, as 16 hex
/// digits), to say in the results which fixture a run used.
String fixtureHash(String text) {
  // Two 32-bit halves, so that the result is the same in every runtime.
  var high = 0xcbf29ce4;
  var low = 0x84222325;
  const primeLow = 0x1b3;
  const primeHigh = 0x100;
  const mask = 0xffffffff;
  for (final unit in text.codeUnits) {
    low ^= unit;
    final lowProduct = low * primeLow;
    final carry = lowProduct ~/ 0x100000000;
    final newHigh = (high * primeLow + low * primeHigh + carry) & mask;
    low = lowProduct & mask;
    high = newHigh;
  }
  return high.toRadixString(16).padLeft(8, '0') +
      low.toRadixString(16).padLeft(8, '0');
}
