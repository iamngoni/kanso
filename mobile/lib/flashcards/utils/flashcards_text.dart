// Kanso — flashcards_text.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../core/bridge/api/cards_types.dart';
import '../../core/bridge/api/draft_types.dart';
import '../../core/utils/ai_trouble_text.dart';
import '../../gen/l10n/app_localizations.dart';

/// What flashcards, Quiz Me and drafting need besides words: the scope that
/// reviews everything and the picker values. Their words are in the ARB
/// files (`flashcards*`).
abstract final class FlashcardsText {
  /// Messages the core already words for a person (not a model's error).
  static const List<String> _plain = <String>[
    'None of these',
    'This note',
    'The model',
    'Nothing came back',
    'Say what',
    "That's more than",
  ];

  /// A failure in words a person can act on: the core's own plain messages
  /// as they are, a model's error read as [aiTroubleText] reads it.
  static String failure(String raw, AppLocalizations l10n) {
    final text = raw.trim();
    if (_plain.any(text.startsWith)) {
      return text;
    }
    return aiTroubleText(text, l10n);
  }

  static DraftShape shapeAt(int i) => DraftShape.values[i];
  static DraftLength lengthAt(int i) => DraftLength.values[i];

  /// The names of the four answers, in the order of their buttons.
  static List<String> grades(AppLocalizations l10n) => <String>[
    l10n.flashcardsAgain,
    l10n.flashcardsHard,
    l10n.flashcardsGood,
    l10n.flashcardsEasy,
  ];

  /// The names of the draft kinds, in the order of [DraftShape].
  static List<String> shapes(AppLocalizations l10n) => <String>[
    l10n.flashcardsShapeReport,
    l10n.flashcardsShapeEssay,
    l10n.flashcardsShapeUpdate,
    l10n.flashcardsShapeOutline,
    l10n.flashcardsShapeCustom,
  ];

  /// The names of the draft lengths, in the order of [DraftLength].
  static List<String> lengths(AppLocalizations l10n) => <String>[
    l10n.flashcardsLengthShort,
    l10n.flashcardsLengthMedium,
    l10n.flashcardsLengthLong,
  ];

  /// The scope that reviews every card.
  static const CardScopeArg everything = CardScopeArg(
    kind: CardScopeKind.all,
    id: '',
  );
}
