// Kanso — flashcards_text.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../core/bridge/api/cards_types.dart';
import '../../core/bridge/api/draft_types.dart';

/// The words of flashcards, Quiz Me and drafting.
///
/// Kept here, not in the ARB files yet: the phrases are English only, like
/// the desktop's, until they are translated together.
abstract final class FlashcardsText {
  static const String reviewTitle = 'Review';
  static const String showAnswer = 'Show Answer';
  static const String again = 'Again';
  static const String hard = 'Hard';
  static const String good = 'Good';
  static const String easy = 'Easy';
  static const List<String> grades = <String>[again, hard, good, easy];
  static const String swipeHint = 'Swipe left for Again, right for Good';
  static const String doneForToday = 'Done for today.';
  static const String doneBody =
      'Come back when more cards are due. Cards are lines written as '
      'question :: answer in your notes.';
  static const String reviewMore = 'Review More';
  static const String noCardsTitle = 'No cards yet';
  static const String noCardsBody =
      'Write a line like  What is FSRS? :: A scheduler  in any note. Each '
      'such line is a flashcard.';
  static const String stats = 'Stats';
  static const String settings = 'Flashcards';
  static const String suspend = 'Suspend Card';
  static const String suspended = 'Card suspended.';
  static const String openNote = 'Open Note';

  static String progress(int at, int total, int fresh) =>
      '$at of $total${fresh > 0 ? ' · $fresh new' : ''}';

  static String interval(String name, String span) => '$name · $span';

  // Stats.
  static const String statsTotal = 'Cards';
  static const String statsDue = 'Due today';
  static const String statsNew = 'New';
  static const String statsSuspended = 'Suspended';
  static const String statsPerDay = 'Reviews, last 30 days';
  static const String statsRetention = 'Retention, last 30 days';
  static const String statsForecast = 'Due, next 7 days';
  static const String statsHardest = 'Hardest cards';
  static const String statsNone = 'Nothing reviewed yet.';

  // Settings.
  static const String newPerDay = 'New cards a day';
  static const String reviewsPerDay = 'Reviews a day';
  static const String targetRecall = 'Target recall';
  static const String dailyReminder = 'Daily reminder';
  static const String reminderOff = 'Off';
  static const String showIcons = 'Show card icons in the editor';
  static const String resetHistory = 'Reset Review History…';
  static const String resetTitle = 'Reset review history?';
  static const String resetBody =
      'Every card starts again as new, on all your devices. The cards in '
      'your notes stay.';
  static const String reset = 'Reset';

  // Make Flashcards.
  static const String makeCards = 'Make Flashcards…';
  static const String makeBlockedExcluded = 'This note is excluded from AI.';
  static const String makeReading = 'Reading the note…';
  static const String sourceNotFound = 'Source not found';
  static const String noSuggestions = 'No cards came back. Try again.';
  static String addCards(int n) => n == 1 ? 'Add 1 Card' : 'Add $n Cards';
  static String added(int n) => n == 1 ? 'Added 1 card.' : 'Added $n cards.';

  // Quiz Me.
  static const String quizMe = 'Quiz Me';
  static const String quizHint = 'Type your answer';
  static const String quizCheck = 'Check';
  static const String quizNext = 'Next';
  static const String quizSaveCard = 'Save as Card';
  static const String quizDone = 'That was the round.';
  static const String quizAgain = 'Another Round';
  static String quizProgress(int at, int total) => 'Question $at of $total';

  // Drafting.
  static const String draftFrom = 'Draft from This Note…';
  static const String draftTitle = 'Draft';
  static const String draftSources = 'Sources';
  static const String draftWhat = 'What should it say, and for whom?';
  static const String draftWrite = 'Write Draft';
  static const String draftStop = 'Stop';
  static const String draftTryAgain = 'Try Again';
  static const String draftCopy = 'Copy';
  static const String draftSave = 'Save as Note';
  static const String draftDiscard = 'Discard';
  static const String draftDiscardTitle = 'Discard this draft?';
  static const String draftDiscardBody =
      'It has not been saved. Drafts are not kept.';
  static const String draftNoSource = 'No source in your notes';
  static const String draftNone =
      'None of these notes can be sent to AI. Locked notes and notes tagged '
      '#ai-exclude are never sent.';
  static const String draftNoAi =
      'Drafting needs an AI provider. Choose one in Settings › AI.';
  static const String draftSaved = 'Saved the draft as a note.';
  static const String draftReviewTitle = 'Review';
  static const String draftNotebook = 'Notebook';
  static String draftSent(int n) =>
      'Only these ${n == 1 ? 'note is' : '$n notes are'} sent to your AI.';
  static const List<String> shapes = <String>[
    'Report',
    'Essay',
    'Update',
    'Outline',
    'Custom',
  ];
  static const List<String> lengths = <String>['Short', 'Medium', 'Long'];

  static DraftShape shapeAt(int i) => DraftShape.values[i];
  static DraftLength lengthAt(int i) => DraftLength.values[i];

  /// The scope that reviews every card.
  static const CardScopeArg everything = CardScopeArg(
    kind: CardScopeKind.all,
    id: '',
  );
}
