// Kanso — sync_flow_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:kanso/core/views/widgets/kanso_text_field.dart';
import 'package:kanso/editor/views/widgets/kanso_editable_text.dart';

import 'helpers/flow_driver.dart';

// What `scripts/mobile-sync-e2e.sh` set up on its local Worker, as it hands
// it over. Without them there is no server to sync with, and the flow is
// skipped.
const String _server = String.fromEnvironment('KANSO_E2E_SERVER');
const String _user = String.fromEnvironment('KANSO_E2E_USER');
const String _password = String.fromEnvironment('KANSO_E2E_PASSWORD');
const String _canary = String.fromEnvironment('KANSO_E2E_CANARY_PREFIX');
const String _recoveryUser = String.fromEnvironment('KANSO_E2E_RECOVERY_USER');
const String _recoveryKey = String.fromEnvironment(
  'KANSO_E2E_RECOVERY_ACCOUNT_KEY',
);
const String _recoveryCanary = String.fromEnvironment(
  'KANSO_E2E_RECOVERY_CANARY_PREFIX',
);
const String _friend = String.fromEnvironment('KANSO_E2E_FRIEND');
const String _shareTitle = String.fromEnvironment('KANSO_E2E_SHARE_TITLE');
const String _shareBody = String.fromEnvironment('KANSO_E2E_SHARE_BODY');
const String _editText = String.fromEnvironment('KANSO_E2E_EDIT_TEXT');
const String _newPassword = String.fromEnvironment('KANSO_E2E_NEW_PASSWORD');

/// How long the server and the peer may take to answer.
const Duration _slow = Duration(seconds: 90);

/// The text field labelled [label].
Finder _field(String label) => find.descendant(
  of: find.ancestor(
    of: find.text(label),
    matching: find.byType(KansoTextField),
  ),
  matching: find.byType(EditableText),
);

/// Scrolls the page down until [finder] is built: a long page builds only
/// the rows near the screen.
Future<void> _reveal(WidgetTester tester, Finder finder) async {
  if (finder.evaluate().isNotEmpty) {
    return;
  }
  await tester.scrollUntilVisible(
    finder,
    200,
    scrollable: find.byType(Scrollable).last,
  );
}

Future<void> _type(WidgetTester tester, String label, String text) async {
  await _reveal(tester, find.text(label));
  await tester.ensureVisible(_field(label).first);
  await tester.enterText(_field(label).first, text);
  await tester.pump();
}

/// Taps the button or row that says [label], scrolled into view.
Future<void> _tap(WidgetTester tester, String label) async {
  await _reveal(tester, find.text(label));
  final target = find.text(label).last;
  await tester.ensureVisible(target);
  await tester.pumpAndSettle();
  await tester.tap(target);
  await tester.pumpAndSettle();
}

/// Waits for [finder], pumping real time the server's answers need.
Future<void> _await(WidgetTester tester, Finder finder) =>
    pumpUntil(tester, finder, timeout: _slow);

/// Settings, from the library: the drawer of a phone, beside the list on a
/// tablet.
Future<void> _openSettings(WidgetTester tester) async {
  await openLibrary(tester);
  await tester.tap(find.bySemanticsLabel('Settings').first);
  await tester.pumpAndSettle();
}

/// From Settings › Sign in, on the script's server.
Future<void> _toSignIn(WidgetTester tester) async {
  await _openSettings(tester);
  await _tap(tester, 'Sign in');
  await _tap(tester, 'Server');
  await _type(tester, 'Server address', _server);
  await _tap(tester, 'Use this server');
  await _await(tester, _field('Username'));
}

/// Answers the question a library with notes of its own asks when it signs
/// in to an account, if it asks: [choice].
Future<void> _answerBinding(WidgetTester tester, String choice) async {
  final deadline = DateTime.now().add(const Duration(seconds: 15));
  while (DateTime.now().isBefore(deadline)) {
    await tester.pump(const Duration(milliseconds: 200));
    if (find.text(choice).evaluate().isNotEmpty) {
      await _tap(tester, choice);
      return;
    }
    if (find.text('Sign in to Kanso').evaluate().isEmpty &&
        find.text('Reset your password').evaluate().isEmpty) {
      return;
    }
  }
}

/// Back to the note list from Settings and the pages over it. The bottom
/// bar's history arrows are also called Back, so the pages are popped
/// rather than tapped through.
Future<void> _backToNotes(WidgetTester tester) async {
  tester
      .state<NavigatorState>(find.byType(Navigator).first)
      .popUntil((route) => route.isFirst);
  await tester.pumpAndSettle();
  await closeLibrary(tester);
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets(
    'sign in and sync, approve a new device, accept a shared note and edit '
    'it live, then join another account with its recovery key',
    (tester) async {
      if (_server.isEmpty) {
        markTestSkipped(
          'Run it through scripts/mobile-sync-e2e.sh --device <id>, which '
          'starts the Worker and the peer.',
        );
        return;
      }
      await startApp(tester);

      // 1. Sign in to U's account: its notes arrive.
      await _toSignIn(tester);
      await _type(tester, 'Username', _user);
      await _type(tester, 'Password', _password);
      await _tap(tester, 'Sign in');
      await _answerBinding(tester, 'Add them to the account');
      await _backToNotes(tester);
      await _await(tester, find.textContaining('$_canary-0'));

      // 2. The peer asks to join U from another device: approve it here.
      await _await(tester, find.text('Approve a device'));
      await _tap(tester, 'Approve');
      await pumpUntilGone(tester, find.text('Approve a device'));

      // 3. F's note waits in Shared with you: accept it, and type in it.
      await openLibrary(tester);
      await tester.tap(inLibrary('Shared with you'));
      await tester.pumpAndSettle();
      await _await(tester, find.text('A note from $_friend'));
      await _tap(tester, 'Accept');
      await _backToNotes(tester);
      await openLibrary(tester);
      await tester.tap(inLibrary('All Notes'));
      await tester.pumpAndSettle();
      await _await(tester, inList(_shareTitle));
      await tester.tap(inList(_shareTitle));
      await tester.pumpAndSettle();
      await _await(tester, find.textContaining('Shared · Can edit'));
      await tester.tap(find.bySemanticsLabel('Edit').first);
      await tester.pumpAndSettle();
      final text = find.byType(KansoEditableText);
      final now = tester.widget<KansoEditableText>(text).controller.text;
      expect(now, contains(_shareBody));
      await tester.enterText(text, '$now$_editText');
      await tester.pump();
      // Preview saves it; the room carries it to F, whom the peer watches.
      await tester.tap(find.bySemanticsLabel('Preview').first);
      await _await(tester, find.textContaining(_editText.trim()));
      await _await(tester, find.textContaining('· Live'));
      await tester.runAsync(
        () => Future<void>.delayed(const Duration(seconds: 5)),
      );
      await leaveNote(tester);

      // 4. Sign out, and into R's account with its recovery key: R's notes
      // take the place of U's.
      await _openSettings(tester);
      await _tap(tester, _user);
      await _tap(tester, 'Sign out');
      await _tap(tester, 'Sign out');
      await _await(tester, find.text('Sign in'));
      await _toSignIn(tester);
      await _tap(tester, 'Forgot your password? Use your recovery key');
      await _type(tester, 'Username', _recoveryUser);
      await _type(tester, 'Recovery key', _recoveryKey);
      await _type(tester, 'New password', _newPassword);
      await _type(tester, 'Repeat password', _newPassword);
      await _tap(tester, 'Reset password');
      await _answerBinding(tester, 'Replace them with the account’s notes');
      await _await(tester, find.text('Your recovery key'));
      await _tap(tester, 'I’ve saved my recovery key somewhere safe');
      await _tap(tester, 'Done');
      await _backToNotes(tester);
      await _await(tester, find.textContaining('$_recoveryCanary-0'));
      expect(find.textContaining('$_canary-0'), findsNothing);
    },
    timeout: const Timeout(Duration(minutes: 10)),
  );
}
