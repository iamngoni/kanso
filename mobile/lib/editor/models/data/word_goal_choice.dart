// Kanso — word_goal_choice.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// A choice made in the editor's word-goal picker.
sealed class WordGoalChoice {
  const WordGoalChoice();
}

/// Let the note follow the library default.
final class WordGoalDefault extends WordGoalChoice {
  const WordGoalDefault();
}

/// Turn off the goal for this note, even when a default exists.
final class WordGoalOff extends WordGoalChoice {
  const WordGoalOff();
}

/// Give this note a word-count target.
final class WordGoalWords extends WordGoalChoice {
  const WordGoalWords(this.words);

  final int words;
}
