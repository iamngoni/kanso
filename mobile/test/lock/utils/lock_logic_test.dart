// Kanso — lock_logic_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show BoolAny, Glados, Glados2, Glados3, IntAnys, StringAnys, any;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/lock/models/data/lock_mode.dart';
import 'package:kanso/lock/models/data/note_menu_action.dart';
import 'package:kanso/lock/models/data/password_check.dart';
import 'package:kanso/lock/utils/check_new_password.dart';
import 'package:kanso/lock/utils/lock_mode_for.dart';
import 'package:kanso/lock/utils/note_menu_actions.dart';
import 'package:kanso/lock/utils/notebook_lock_rule.dart';

import '../../helpers/property.dart';

NotebookRow book(String id, {String? parent, bool locked = false}) =>
    NotebookRow(
      id: id,
      name: id,
      parentId: parent,
      count: 0,
      locked: locked,
      depth: 0,
      hasChildren: false,
    );

void main() {
  group('lockModeFor', () {
    Glados3<bool, bool, bool>(
      any.bool,
      any.bool,
      any.bool,
      kansoExploreConfig(),
    ).test('a missing lock password is always set first', (has, open, bio) {
      final mode = lockModeFor(
        LockStatus(hasLockPassword: has, unlocked: open, biometricsOn: bio),
      );
      if (!has) {
        expect(mode, LockMode.setPassword);
      } else {
        expect(mode, open ? LockMode.confirm : LockMode.askPassword);
      }
    });

    test("the desktop's three ways", () {
      LockMode of({required bool has, required bool open}) => lockModeFor(
        LockStatus(hasLockPassword: has, unlocked: open, biometricsOn: false),
      );
      expect(of(has: false, open: false), LockMode.setPassword);
      expect(of(has: true, open: false), LockMode.askPassword);
      expect(of(has: true, open: true), LockMode.confirm);
    });
  });

  group('checkNewPassword', () {
    Glados<String>(any.letterOrDigits, kansoExploreConfig()).test(
      'the same text is ok exactly when it is long enough',
      (text) {
        expect(
          checkNewPassword(text, text),
          text.runes.length >= minLockPasswordLength
              ? PasswordCheck.ok
              : PasswordCheck.tooShort,
        );
      },
    );

    Glados2<String, String>(
      any.letterOrDigits,
      any.letterOrDigits,
      kansoExploreConfig(),
    ).test('different texts never match, whatever their length', (a, b) {
      if (a != b) {
        expect(checkNewPassword(a, b), PasswordCheck.mismatch);
      }
    });

    Glados<int>(any.intInRange(0, 40), kansoExploreConfig()).test(
      'the length limit is eight characters',
      (length) {
        final text = 'x' * length;
        expect(
          checkNewPassword(text, text) == PasswordCheck.ok,
          length >= 8,
        );
      },
    );

    test('eight characters of any kind count, not bytes', () {
      expect(checkNewPassword('パスワードです!', 'パスワードです!'), PasswordCheck.ok);
      expect(checkNewPassword('パスワード', 'パスワード'), PasswordCheck.tooShort);
    });
  });

  group('noteMenuActions', () {
    const states = <LockState>[
      LockState.plain(),
      LockState.open(),
      LockState.closed(),
    ];

    Glados2<int, bool>(
      any.intInRange(0, 3),
      any.bool,
      kansoExploreConfig(),
    ).test('the menu follows the note and the session', (index, open) {
      final lock = states[index];
      final actions = noteMenuActions(lock: lock, unlocked: open);
      // A closed locked note offers nothing; its card unlocks it.
      expect(
        actions.isEmpty,
        lock is LockState_Closed,
      );
      // Remove lock only for a locked note that is open.
      expect(
        actions.contains(NoteMenuAction.removeLock),
        lock is LockState_Open,
      );
      // Only a never locked note can be locked.
      expect(
        actions.contains(NoteMenuAction.lockNote),
        lock is LockState_Plain,
      );
      // Lock now whenever the notes are open.
      if (lock is! LockState_Closed) {
        expect(
          actions.contains(NoteMenuAction.lockNow),
          open || lock is LockState_Open,
        );
        // Any note that is read can be written as a PDF or a web page, and
        // the PDF comes first.
        expect(actions.first, NoteMenuAction.exportPdf);
        expect(actions[1], NoteMenuAction.exportHtml);
      }
      expect(actions.toSet().length, actions.length);
    });
  });

  group('notebookLockRule', () {
    test('Lock Notebook when it is not locked', () {
      final all = <NotebookRow>[book('a'), book('b', parent: 'a')];
      expect(notebookLockRule(all[1], all), NotebookLockRule.lock);
    });

    test('Remove Notebook Lock when it has a lock of its own', () {
      final all = <NotebookRow>[book('a', locked: true)];
      expect(notebookLockRule(all.single, all), NotebookLockRule.remove);
    });

    test('nothing when it is locked through a parent', () {
      final all = <NotebookRow>[
        book('a', locked: true),
        book('b', parent: 'a'),
        book('c', parent: 'b'),
      ];
      expect(notebookLockRule(all[1], all), NotebookLockRule.none);
      expect(notebookLockRule(all[2], all), NotebookLockRule.none);
      expect(lockedThroughParent(all[0], all), isFalse);
    });

    test('a parent that is missing or a cycle ends the walk', () {
      final orphan = book('x', parent: 'gone');
      expect(lockedThroughParent(orphan, <NotebookRow>[orphan]), isFalse);
      final a = book('a', parent: 'b');
      final b = book('b', parent: 'a');
      expect(lockedThroughParent(a, <NotebookRow>[a, b]), isFalse);
    });

    Glados2<bool, bool>(any.bool, any.bool, kansoExploreConfig()).test(
      "the rule is a function of own lock and the parent's",
      (own, parent) {
        final all = <NotebookRow>[
          book('p', locked: parent),
          book('c', parent: 'p', locked: own),
        ];
        final rule = notebookLockRule(all[1], all);
        if (own) {
          expect(rule, NotebookLockRule.remove);
        } else if (parent) {
          expect(rule, NotebookLockRule.none);
        } else {
          expect(rule, NotebookLockRule.lock);
        }
      },
    );
  });
}
