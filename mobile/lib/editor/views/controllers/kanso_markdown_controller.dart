// Kanso — kanso_markdown_controller.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../models/data/editor_window.dart';
import '../../models/data/outside_side.dart';
import '../../models/repos/abs/editor_repository.dart';
import '../../utils/diff_values.dart';
import '../../utils/edit_log.dart';
import '../../utils/editor_spans.dart';
import '../../utils/editor_timing.dart';
import '../../utils/editor_window.dart';
import '../../utils/shift_runs.dart';

/// The controller of the note's text field: the only thing that changes the
/// text.
///
/// The text is Rust's. Every change the field makes (a key, the input
/// method, a paste) comes through [value], is told to the open
/// [EditorDocument] as a [TextChange] and, when Rust accepts it, shown;
/// what Rust decides (a newline that continues a list, a format, an undo, a
/// completion chosen) comes back as a replacement that is applied in the
/// same turn, so no keystroke can land between the two.
///
/// It also keeps the colours of the text: the last runs the core sent,
/// moved along as the text changes ([shiftRuns]) until fresh ones arrive,
/// asked for once a frame at most, and drawn by [buildTextSpan].
///
/// A text that arrives from outside ([reload]) waits for a composition to
/// end; the text is never replaced under one.
///
/// A long note is edited a window at a time. The field's text, its
/// selection and everything the `TextEditingController` says are those of
/// the window (whole lines of the note, [windowStart] units into it); the
/// rest of the note is kept as [before] and [after], for the screen to draw
/// read-only, and the window moves along as the caret does. Everything said
/// to Rust and heard from it is in units of the whole note, and the
/// `note…` members are the note's. A note no longer than [windowUnits] is
/// one window, and nothing is any different.
class KansoMarkdownController extends TextEditingController {
  KansoMarkdownController({
    required this.document,
    required EditorSnapshot snapshot,
    required this.themeId,
    int? caret,
    this.windowUnits = WindowSizes.cap,
    this.windowMargin = WindowSizes.margin,
    this.onContext,
    this.onEdited,
    this.onLocked,
    this.onFailure,
    this.misspelledStyle = const TextStyle(
      decoration: TextDecoration.underline,
      decorationStyle: TextDecorationStyle.wavy,
    ),
    this.composingStyle = const TextStyle(decoration: TextDecoration.underline),
  }) : _version = snapshot.version,
       _history = snapshot.history,
       super(
         text: _firstWindow(
           snapshot.text,
           caret,
           windowUnits,
           windowMargin,
         ).slice(snapshot.text),
       ) {
    final note = snapshot.text;
    final first = _firstWindow(note, caret, windowUnits, windowMargin);
    _before = note.substring(0, first.start);
    _after = note.substring(first.end);
    hasBefore.value = _before.isNotEmpty;
    if (caret != null) {
      final at = caret.clamp(first.start, first.end) - first.start;
      super.value = TextEditingValue(
        text: super.value.text,
        selection: TextSelection.collapsed(offset: at),
      );
    }
    // Rust starts with the caret at the start; tell it where the field's is.
    final at = caret == null ? 0 : caret.clamp(0, note.length);
    document.select(Range16(start: at, end: at), composing: false);
    requestRuns();
  }

  /// The first window of [note]: all of it when it is short, otherwise
  /// the window around [caret] (the start without one).
  static EditorWindow _firstWindow(
    String note,
    int? caret,
    int cap,
    int margin,
  ) {
    if (note.length <= cap) {
      return wholeNote(note.length);
    }
    final at = (caret ?? 0).clamp(0, note.length);
    return windowAround(note, at, at, cap: cap, margin: margin);
  }

  /// The note open for editing.
  final EditorDocument document;

  /// The most units a window holds, and how close to its edge the selection
  /// may come before it moves.
  final int windowUnits;
  final int windowMargin;

  /// The built-in theme the text is coloured in. Asks for fresh colours when
  /// it changes.
  String themeId;

  /// Told what is under the caret after every change to the text or the
  /// selection.
  final ValueChanged<EditorContext>? onContext;

  /// Told after each change to the text, whoever made it.
  final VoidCallback? onEdited;

  /// Told when Rust says the note's lock has closed.
  final VoidCallback? onLocked;

  /// Told of an error that is neither of the above.
  final ValueChanged<ApplicationError>? onFailure;

  /// How a misspelled word is drawn; the screen sets its colour.
  TextStyle misspelledStyle;

  /// How the text being composed is drawn.
  final TextStyle composingStyle;

  /// The completion being offered, and the item Return would take; set by
  /// the screen. A newline typed while there is one chooses it.
  Completion? completion;
  int completionIndex = 0;

  int _version;
  HistoryState _history;
  List<EditorRun> _runs = const <EditorRun>[];
  Range16? _lastSelection;
  bool _lastComposing = false;
  bool _syncing = false;
  bool _disposed = false;
  bool _runsDue = false;
  bool _runsScheduled = false;
  bool _runsInFlight = false;
  EditorSnapshot? _waiting;

  // The changes made since the colours on screen were asked for: colours
  // that come back for an older text are moved along them and shown.
  final EditLog _log = EditLog();

  // The note outside the window: the text before it, which ends with the
  // line break that ends the line before the window, and the text after
  // it, which starts with the line break that ends the window's last line.
  String _before = '';
  String _after = '';
  int _windowRevision = 0;
  int _outsideGeneration = 0;
  final Map<(OutsideSide, int), List<EditorRun>> _outsideRuns =
      <(OutsideSide, int), List<EditorRun>>{};
  final Set<(OutsideSide, int)> _outsideAsked = <(OutsideSide, int)>{};
  final List<(OutsideSide, int)> _outsideQueue = <(OutsideSide, int)>[];
  bool _outsideBusy = false;

  /// Bumped when the text outside the window, or its colours, change: what
  /// the screen's drawing of it listens to.
  final ValueNotifier<int> outsideChanged = ValueNotifier<int>(0);

  /// Whether there is note text before the window: what the screen draws
  /// its scroll view from the editor outward for.
  final ValueNotifier<bool> hasBefore = ValueNotifier<bool>(false);

  /// Told, just before the window moves, where in the note the caret is (or
  /// will be), for the screen to remember where that is drawn. Not told of
  /// a window that is replaced by a text from outside.
  ValueChanged<int>? onWindowWillMove;

  // The versions of the text not yet shown in fresh colours, and when they
  // were made: kept only while a performance test is listening.
  final List<(int, int)> _unseen = <(int, int)>[];

  // The last spans built, to build them again only for new input.
  String? _spanText;
  List<EditorRun>? _spanRuns;
  TextRange? _spanComposing;
  List<TextRange>? _spanMisspellings;
  TextStyle? _spanBase;
  TextStyle? _spanMisspelled;
  TextSpan? _span;

  /// The version of the text Rust has, as of the last thing it said.
  int get version => _version;

  /// Whether there is anything to undo and redo, as of the last thing Rust
  /// said.
  HistoryState get history => _history;

  /// The colours the text is drawn in now.
  List<EditorRun> get runs => _runs;

  /// Whether an input method is composing text now.
  bool get isComposing => _composingIn(value);

  /// Where the window starts in the note, in units.
  int get windowStart => _before.length;

  /// How many units the whole note has.
  int get noteLength =>
      _before.length + super.value.text.length + _after.length;

  /// How many times the window has moved or been replaced.
  int get windowRevision => _windowRevision;

  /// Whether the field holds only a part of the note.
  bool get isWindowed => _before.isNotEmpty || _after.isNotEmpty;

  /// The note's text before the window, ending with a line break, or empty.
  String get before => _before;

  /// The note's text after the window, starting with a line break, or empty.
  String get after => _after;

  /// The whole note's text. Puts it together, which costs the note's size:
  /// not for every keystroke.
  String get noteText => _before + super.value.text + _after;

  /// The selection in units of the whole note.
  TextSelection get noteSelection {
    final at = super.value.selection;
    if (!at.isValid) {
      return at;
    }
    final origin = _before.length;
    return TextSelection(
      baseOffset: at.baseOffset + origin,
      extentOffset: at.extentOffset + origin,
    );
  }

  /// Where [noteOffset] of the note is in the field, or null when it is
  /// outside the window.
  int? fieldOffsetOf(int noteOffset) {
    final at = noteOffset - _before.length;
    return at >= 0 && at <= super.value.text.length ? at : null;
  }

  /// Whether the text at [offset] is code, an address or other text that is
  /// not spell checked.
  bool isQuietAt(int offset) {
    var low = 0;
    var high = _runs.length - 1;
    while (low <= high) {
      final middle = (low + high) >> 1;
      final run = _runs[middle];
      if (offset < run.range.start) {
        high = middle - 1;
      } else if (offset >= run.range.end) {
        low = middle + 1;
      } else {
        return run.quiet;
      }
    }
    return false;
  }

  // MARK: Changes the field makes

  @override
  set value(TextEditingValue newValue) {
    if (_syncing || _disposed) {
      super.value = newValue;
      return;
    }
    final old = super.value;
    if (old.text == newValue.text) {
      super.value = newValue;
      _moved(newValue);
      return;
    }
    _typed(old, newValue);
  }

  void _typed(TextEditingValue old, TextEditingValue now) {
    final diff = diffValues(old, now);
    final composingAfter = _composingIn(now);
    if (diff.isInsertion && diff.text == '\n' && !composingAfter) {
      // Return: a list continues, a table gets its next row; or the
      // completion is chosen. A keyboard that was composing the last word
      // commits it with the newline in one update (Gboard does), so a
      // composition before is no reason to type it as it is.
      final offered = completion;
      if (offered != null &&
          completionIndex >= 0 &&
          completionIndex < offered.items.length) {
        _tell(old);
        insert(offered.items[completionIndex]);
        return;
      }
      _tell(old);
      if (_command(
        EditCommand.enter(plain: HardwareKeyboard.instance.isShiftPressed),
      )) {
        return;
      }
    }
    final length = now.text.length;
    final origin = _before.length;
    final change = TextChange(
      baseVersion: _version,
      start: origin + diff.start,
      end: origin + diff.end,
      text: diff.text,
      selection: _rangeOf(now.selection, length),
      composing: composingAfter
          ? Range16(
              start: origin + now.composing.start.clamp(0, length),
              end: origin + now.composing.end.clamp(0, length),
            )
          : null,
    );
    EditorTiming.measure('kanso.apply', () => document.apply(change)).fold(
      (error) {
        // The field has the person's text and Rust refused it. Put the
        // field back as it was; it is told to the input method too.
        notifyListeners();
        _fail(error);
      },
      (outcome) {
        _log.record(
          from: _version,
          to: outcome.version,
          start: diff.start,
          end: diff.end,
          inserted: diff.text.length,
        );
        _version = outcome.version;
        _history = outcome.history;
        _stamp();
        _runs = shiftRuns(
          _runs,
          start: diff.start,
          end: diff.end,
          inserted: diff.text.length,
        );
        super.value = now;
        final ended = _lastComposing && !composingAfter;
        _tell(now);
        requestRuns();
        onEdited?.call();
        if (ended) {
          _showWaiting();
        }
        _rewindow();
      },
    );
  }

  /// A selection or composition change with the text as it was.
  void _moved(TextEditingValue now) {
    final selection = _rangeOf(now.selection, now.text.length);
    final composing = _composingIn(now);
    if (selection == _lastSelection && composing == _lastComposing) {
      return;
    }
    final ended = _lastComposing && !composing;
    _tell(now);
    if (ended) {
      _showWaiting();
    }
    _rewindow();
  }

  /// Shows the text that waited for a composition to end.
  void _showWaiting() {
    final waiting = _waiting;
    if (waiting != null) {
      _waiting = null;
      _show(waiting);
    }
  }

  /// Tells Rust where the selection is, and the screen what is under it.
  void _tell(TextEditingValue now) {
    final selection = _rangeOf(now.selection, now.text.length);
    final composing = _composingIn(now);
    _lastSelection = selection;
    _lastComposing = composing;
    document.select(selection, composing: composing).fold(_fail, (context) {
      _history = context.history;
      onContext?.call(context);
    });
  }

  // MARK: Changes Rust decides

  /// Runs [command] and shows what it did. False when there was nothing for
  /// it to do.
  bool run(EditCommand command) => _command(command);

  bool _command(EditCommand command) {
    return document.command(command).fold((error) {
      _fail(error);
      return false;
    }, _adopt);
  }

  /// Undoes the last step; false when there is none.
  bool undo() {
    return document.undo().fold((error) {
      _fail(error);
      return false;
    }, _adopt);
  }

  /// Redoes the last undone step; false when there is none.
  bool redo() {
    return document.redo().fold((error) {
      _fail(error);
      return false;
    }, _adopt);
  }

  /// Shows [offered] as what completes at the caret, with its first row
  /// active; null for none.
  void offer(Completion? offered) {
    if (offered == completion && completionIndex == 0) {
      return;
    }
    completion = offered;
    completionIndex = 0;
    notifyListeners();
  }

  /// Moves the active row of the completions [delta] rows, within them.
  void stepCompletion(int delta) {
    final offered = completion;
    if (offered == null || offered.items.isEmpty) {
      return;
    }
    final next = (completionIndex + delta).clamp(0, offered.items.length - 1);
    if (next != completionIndex) {
      completionIndex = next;
      notifyListeners();
    }
  }

  /// Chooses the active row of the completions; false when there is none.
  bool chooseCompletion() {
    final offered = completion;
    if (offered == null ||
        completionIndex < 0 ||
        completionIndex >= offered.items.length) {
      return false;
    }
    return insert(offered.items[completionIndex]);
  }

  /// Puts the completion [item] in the place the completion names, as one
  /// step.
  bool insert(CompletionItem item) {
    final offered = completion;
    if (offered == null) {
      return false;
    }
    completion = null;
    return _command(
      EditCommand.insert(range: offered.replace, text: item.insert),
    );
  }

  /// Shows what Rust did: the replacement, then the selection. False for no
  /// outcome.
  bool _adopt(EditOutcome? outcome) {
    if (outcome == null) {
      return false;
    }
    final replace = outcome.replace;
    final text = super.value.text;
    final origin = _before.length;
    final previous = _version;
    _version = outcome.version;
    _history = outcome.history;
    _stamp();
    final selection = outcome.selection;
    final inside =
        (replace == null ||
            (replace.start >= origin && replace.end <= origin + text.length)) &&
        selection.start >= origin &&
        selection.end <= origin + text.length;
    if (!inside) {
      // Rust changed, or put the caret, somewhere the window is not (an
      // undo of text far from the caret): the window goes there.
      var note = noteText;
      if (replace != null) {
        note = note.replaceRange(replace.start, replace.end, replace.text);
      }
      final base = selection.start.clamp(0, note.length);
      final extent = selection.end.clamp(0, note.length);
      _replaceWindow(
        note,
        TextSelection(baseOffset: base, extentOffset: extent),
      );
      _tell(super.value);
      requestRuns();
      onEdited?.call();
      return true;
    }
    _log.record(
      from: previous,
      to: outcome.version,
      start: replace == null ? 0 : replace.start - origin,
      end: replace == null ? 0 : replace.end - origin,
      inserted: replace == null ? 0 : replace.text.length,
    );
    var next = text;
    if (replace != null) {
      next = next.replaceRange(
        replace.start - origin,
        replace.end - origin,
        replace.text,
      );
      _runs = shiftRuns(
        _runs,
        start: replace.start - origin,
        end: replace.end - origin,
        inserted: replace.text.length,
      );
    }
    final shownSelection = TextSelection(
      baseOffset: (selection.start - origin).clamp(0, next.length),
      extentOffset: (selection.end - origin).clamp(0, next.length),
    );
    final shown = TextEditingValue(text: next, selection: shownSelection);
    _syncing = true;
    try {
      super.value = shown;
    } finally {
      _syncing = false;
    }
    _tell(shown);
    requestRuns();
    onEdited?.call();
    _rewindow();
    return true;
  }

  // MARK: Text from outside

  /// Shows [snapshot]'s text in place of the field's (the note was changed
  /// elsewhere, or the person took the other version), after a composition
  /// ends if one is open. The caret stays where it was, as far as the text
  /// reaches.
  void reload(EditorSnapshot snapshot) {
    if (isComposing) {
      _waiting = snapshot;
      return;
    }
    _show(snapshot);
  }

  void _show(EditorSnapshot snapshot) {
    final old = noteSelection;
    final length = snapshot.text.length;
    _version = snapshot.version;
    _history = snapshot.history;
    _log.reset();
    _replaceWindow(
      snapshot.text,
      old.isValid
          ? TextSelection(
              baseOffset: old.baseOffset.clamp(0, length),
              extentOffset: old.extentOffset.clamp(0, length),
            )
          : const TextSelection.collapsed(offset: 0),
    );
    _tell(super.value);
    requestRuns();
  }

  // MARK: The window

  /// Moves the window if the selection has come near an edge of it (or
  /// grown past it), unless a composition is open.
  void _rewindow() {
    if (_disposed ||
        (_before.isEmpty &&
            _after.isEmpty &&
            super.value.text.length <= windowUnits)) {
      return;
    }
    final value = super.value;
    final selection = value.selection;
    if (!selection.isValid || _composingIn(value)) {
      return;
    }
    final origin = _before.length;
    final current = EditorWindow(
      start: origin,
      end: origin + value.text.length,
    );
    final first = origin + selection.start;
    final last = origin + selection.end;
    if (!needsNewWindow(
      current,
      first,
      last,
      noteLength,
      cap: windowUnits,
      margin: windowMargin,
    )) {
      return;
    }
    final note = noteText;
    final next = nextWindow(
      note,
      current,
      first,
      last,
      cap: windowUnits,
      margin: windowMargin,
    );
    if (next != null) {
      onWindowWillMove?.call(origin + selection.extentOffset);
      _putWindow(
        note,
        next,
        TextSelection(
          baseOffset: origin + selection.baseOffset,
          extentOffset: origin + selection.extentOffset,
        ),
      );
    }
  }

  /// Makes [note] the text, with [selection] (in its units) in a window of
  /// its own around it, as the text arrives from outside or an undo reaches
  /// far from the window. The colours are asked for afresh.
  void _replaceWindow(String note, TextSelection selection) {
    final window = _firstWindow(
      note,
      selection.extentOffset,
      windowUnits,
      windowMargin,
    );
    final wanted = selection.isCollapsed || note.length <= windowUnits
        ? window
        : windowAround(
            note,
            selection.start,
            selection.end,
            cap: windowUnits,
            margin: windowMargin,
          );
    _runs = const <EditorRun>[];
    _putWindow(note, wanted, selection, keepRuns: false);
  }

  /// Puts the window [next] of [note] in the field, with the selection
  /// [selection] in units of the note.
  void _putWindow(
    String note,
    EditorWindow next,
    TextSelection selection, {
    bool keepRuns = true,
  }) {
    final oldOrigin = _before.length;
    _before = note.substring(0, next.start);
    _after = note.substring(next.end);
    _windowRevision++;
    hasBefore.value = _before.isNotEmpty;
    _outsideGeneration++;
    _outsideRuns.clear();
    _outsideAsked.clear();
    _outsideQueue.clear();
    _log.reset();
    final text = next.slice(note);
    if (keepRuns) {
      // The colours that fall in the new window, in its units.
      final delta = oldOrigin - next.start;
      _runs = <EditorRun>[
        for (final run in _runs)
          if (run.range.end + delta > 0 &&
              run.range.start + delta < text.length)
            EditorRun(
              range: Range16(
                start: (run.range.start + delta).clamp(0, text.length),
                end: (run.range.end + delta).clamp(0, text.length),
              ),
              color: run.color,
              italic: run.italic,
              weight: run.weight,
              quiet: run.quiet,
            ),
      ];
    }
    final shown = TextEditingValue(
      text: text,
      selection: selection.isValid
          ? TextSelection(
              baseOffset: (selection.baseOffset - next.start).clamp(
                0,
                text.length,
              ),
              extentOffset: (selection.extentOffset - next.start).clamp(
                0,
                text.length,
              ),
            )
          : const TextSelection.collapsed(offset: 0),
    );
    _syncing = true;
    try {
      super.value = shown;
    } finally {
      _syncing = false;
    }
    outsideChanged.value++;
    requestRuns();
  }

  /// Puts the caret at [noteOffset] of the note, moving the window to it if
  /// it is outside; false when a composition is open (the window does not
  /// move under one) or the offset is no place.
  bool placeCaret(int noteOffset) {
    if (_disposed || isComposing) {
      return false;
    }
    final at = noteOffset.clamp(0, noteLength);
    final inside = fieldOffsetOf(at);
    if (inside != null) {
      selection = TextSelection.collapsed(offset: inside);
      return true;
    }
    final note = noteText;
    final window = windowAround(
      note,
      at,
      at,
      cap: windowUnits,
      margin: windowMargin,
    );
    onWindowWillMove?.call(at);
    _putWindow(note, window, TextSelection.collapsed(offset: at));
    _tell(super.value);
    return true;
  }

  /// Makes the window the whole note and selects all of it: Select All. One
  /// slow layout, for something the person asked for. False when a
  /// composition is open.
  bool selectEverything() {
    if (_disposed || isComposing) {
      return false;
    }
    final note = noteText;
    onWindowWillMove?.call(
      noteSelection.isValid ? noteSelection.extentOffset : 0,
    );
    _putWindow(
      note,
      wholeNote(note.length),
      TextSelection(baseOffset: 0, extentOffset: note.length),
    );
    _tell(super.value);
    return true;
  }

  // MARK: The text outside the window

  /// How many units of text outside the window are coloured at a time.
  static const int outsideChunk = 4096;

  /// The colours of the text outside the window from [from] to [to], in
  /// units of that side's text ([before] without its last line break;
  /// [after] without its first), clipped to them. Colours not yet known are
  /// asked for, and [outsideChanged] says when they arrive.
  List<EditorRun> outsideRunsOf(OutsideSide side, int from, int to) {
    final found = <EditorRun>[];
    final last = to > from ? (to - 1) ~/ outsideChunk : from ~/ outsideChunk;
    for (var chunk = from ~/ outsideChunk; chunk <= last; chunk++) {
      final runs = _outsideRuns[(side, chunk)];
      if (runs == null) {
        _askOutside(side, chunk);
        continue;
      }
      for (final run in runs) {
        if (run.range.end <= from || run.range.start >= to) {
          continue;
        }
        found.add(
          run.range.start >= from && run.range.end <= to
              ? run
              : EditorRun(
                  range: Range16(
                    start: run.range.start < from ? from : run.range.start,
                    end: run.range.end > to ? to : run.range.end,
                  ),
                  color: run.color,
                  italic: run.italic,
                  weight: run.weight,
                  quiet: run.quiet,
                ),
        );
      }
    }
    return found;
  }

  void _askOutside(OutsideSide side, int chunk) {
    if (_disposed || !_outsideAsked.add((side, chunk))) {
      return;
    }
    _outsideQueue.add((side, chunk));
    if (!_outsideBusy) {
      _outsideBusy = true;
      scheduleMicrotask(_loadOutside);
    }
  }

  Future<void> _loadOutside() async {
    while (_outsideQueue.isNotEmpty && !_disposed) {
      final (side, chunk) = _outsideQueue.removeAt(0);
      final generation = _outsideGeneration;
      final origin = _before.length;
      final length = super.value.text.length;
      // The side's text without the line break that joins it to the window.
      final sideLength = side == OutsideSide.before
          ? _before.length - 1
          : _after.length - 1;
      final from = chunk * outsideChunk;
      final to = from + outsideChunk < sideLength
          ? from + outsideChunk
          : sideLength;
      if (to <= from) {
        continue;
      }
      // Where that is in the note.
      final base = side == OutsideSide.before ? 0 : origin + length + 1;
      final result = await document.styledRuns(
        start: base + from,
        end: base + to,
        themeId: themeId,
      );
      if (_disposed) {
        return;
      }
      if (generation != _outsideGeneration) {
        // The window moved: this is asked for again if it is needed.
        continue;
      }
      result.fold(
        (error) {
          if (error is LockedError) {
            onLocked?.call();
          }
          _outsideAsked.remove((side, chunk));
        },
        (batch) {
          _outsideRuns[(side, chunk)] = <EditorRun>[
            for (final run in batch.runs)
              EditorRun(
                range: Range16(
                  start: run.range.start - base,
                  end: run.range.end - base,
                ),
                color: run.color,
                italic: run.italic,
                weight: run.weight,
                quiet: run.quiet,
              ),
          ];
          outsideChanged.value++;
        },
      );
    }
    _outsideBusy = false;
  }

  /// Takes the text Rust has: after it refused a change made against an
  /// older text.
  Future<void> resync() async {
    final result = await document.snapshot();
    if (_disposed) {
      return;
    }
    result.fold(_fail, reload);
  }

  void _fail(ApplicationError error) {
    switch (error) {
      case LockedError():
        onLocked?.call();
      case InvalidError():
        unawaited(resync());
      default:
        onFailure?.call(error);
    }
  }

  // MARK: Colours

  /// Notes that the text is now at [_version], to time how long its colours
  /// take.
  void _stamp() {
    if (EditorTiming.observed) {
      _unseen.add((_version, EditorTiming.nowMicros));
    }
  }

  /// Asks for fresh colours: once a frame at most, one request at a time,
  /// and never one for a version of the text that has been left behind.
  void requestRuns() {
    _runsDue = true;
    if (_runsScheduled || _runsInFlight || _disposed) {
      return;
    }
    _runsScheduled = true;
    SchedulerBinding.instance
      ..addPostFrameCallback((_) {
        _runsScheduled = false;
        unawaited(_fetchRuns());
      })
      ..ensureVisualUpdate();
  }

  Future<void> _fetchRuns() async {
    if (_disposed || !_runsDue) {
      return;
    }
    _runsDue = false;
    _runsInFlight = true;
    final origin = _before.length;
    final Either<ApplicationError, RunBatch> result = await document.styledRuns(
      start: origin,
      end: origin + super.value.text.length,
      themeId: themeId,
    );
    _runsInFlight = false;
    if (_disposed) {
      return;
    }
    result.fold(
      (error) {
        if (error is LockedError) {
          onLocked?.call();
        }
        // Any other error leaves the text as it is coloured: plain is fine.
      },
      (batch) {
        // Colours of a text a few keys old are moved along the keys since;
        // only a window that has moved, or a text that came from outside,
        // leaves them behind.
        final List<EditorRun>? carried = origin == _before.length
            ? _log.carry(
                origin == 0
                    ? batch.runs
                    : <EditorRun>[
                        for (final run in batch.runs)
                          EditorRun(
                            range: Range16(
                              start: run.range.start - origin,
                              end: run.range.end - origin,
                            ),
                            color: run.color,
                            italic: run.italic,
                            weight: run.weight,
                            quiet: run.quiet,
                          ),
                      ],
                from: batch.version,
                now: _version,
              )
            : null;
        if (carried == null) {
          // Left behind by a change that cannot be followed.
          _runsDue = true;
          return;
        }
        _runs = carried;
        _log.forget(batch.version);
        if (batch.version != _version) {
          // Not the latest text: the colours of that are asked for next.
          _runsDue = true;
        }
        notifyListeners();
        for (final (version, stamped) in _unseen) {
          if (version <= batch.version) {
            EditorTiming.since('kanso.freshRuns', stamped);
          }
        }
        _unseen.removeWhere((entry) => entry.$1 <= batch.version);
      },
    );
    if (_runsDue) {
      requestRuns();
    }
  }

  // MARK: Drawing

  @override
  TextSpan buildTextSpan({
    required BuildContext context,
    TextStyle? style,
    // ignore: always_put_required_named_parameters_first
    required bool withComposing,
  }) {
    return spanFor(
      style: style ?? const TextStyle(),
      withComposing: withComposing,
    );
  }

  /// The text as spans in [style], with the misspellings in [misspellings]
  /// marked (outside code and addresses).
  TextSpan spanFor({
    required TextStyle style,
    required bool withComposing,
    List<TextRange> misspellings = const <TextRange>[],
  }) {
    final text = super.value.text;
    final composing = withComposing ? super.value.composing : TextRange.empty;
    final cached = _span;
    if (cached != null &&
        identical(text, _spanText) &&
        identical(_runs, _spanRuns) &&
        composing == _spanComposing &&
        identical(misspellings, _spanMisspellings) &&
        style == _spanBase &&
        misspelledStyle == _spanMisspelled) {
      return cached;
    }
    final span = EditorTiming.measure(
      'kanso.buildTextSpan',
      () => editorSpans(
        text: text,
        runs: _runs,
        base: style,
        composing: composing,
        misspellings: misspellings,
        composingStyle: composingStyle,
        misspelledStyle: misspelledStyle,
      ),
    );
    _spanText = text;
    _spanRuns = _runs;
    _spanComposing = composing;
    _spanMisspellings = misspellings;
    _spanBase = style;
    _spanMisspelled = misspelledStyle;
    _span = span;
    return span;
  }

  @override
  void dispose() {
    _disposed = true;
    outsideChanged.dispose();
    hasBefore.dispose();
    super.dispose();
  }

  // MARK: Units

  static bool _composingIn(TextEditingValue value) =>
      value.composing.isValid && !value.composing.isCollapsed;

  /// [selection] of the field, a text [length] long, as units of the note;
  /// the end of the field's text for a selection the field has not set yet.
  Range16 _rangeOf(TextSelection selection, int length) {
    final origin = _before.length;
    if (!selection.isValid) {
      return Range16(start: origin + length, end: origin + length);
    }
    return Range16(
      start: origin + selection.start.clamp(0, length),
      end: origin + selection.end.clamp(0, length),
    );
  }
}
