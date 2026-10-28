// Kanso — account_labels_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

// ignore_for_file: prefer_interpolation_to_compose_strings

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, StringAnys, any;
import 'package:kanso/account/models/data/account_notice.dart';
import 'package:kanso/account/models/data/form_problem.dart';
import 'package:kanso/account/utils/account_labels.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/gen/l10n/app_localizations_en.dart';

import '../../helpers/property.dart';

void main() {
  final l10n = AppLocalizationsEn();

  group('AccountLabels', () {
    test('every problem of a form has its own words', () {
      final words = <String>{
        for (final problem in FormProblem.values) l10n.formProblem(problem),
      };
      expect(words, hasLength(FormProblem.values.length));
      expect(words, everyElement(isNotEmpty));
      expect(
        l10n.formProblem(FormProblem.passwordsDiffer),
        'The passwords don’t match.',
      );
      expect(
        l10n.formProblem(FormProblem.passwordTooShort),
        'Use at least 12 characters. A few random words work well.',
      );
    });

    test('every provider has a name and a button', () {
      expect(l10n.providerName(ProviderKind.apple), 'Apple');
      expect(l10n.continueWith(ProviderKind.google), 'Continue with Google');
      expect(l10n.continueWith(ProviderKind.github), 'Continue with GitHub');
    });

    test('every phase of the wait has words, and a failure its own', () {
      for (final phase in <ApprovalPhase>[
        const ApprovalPhase.requesting(),
        const ApprovalPhase.waiting(),
        const ApprovalPhase.compare(),
        const ApprovalPhase.unlocking(),
        const ApprovalPhase.approved(),
        const ApprovalPhase.noDevices(),
        const ApprovalPhase.cancelled(),
        const ApprovalPhase.expired(),
      ]) {
        expect(l10n.approvalPhase(phase), isNotEmpty, reason: phase.toString());
      }
      expect(
        l10n.approvalPhase(const ApprovalPhase.waiting()),
        l10n.approvalPhase(const ApprovalPhase.compare()),
      );
      expect(
        l10n.approvalPhase(const ApprovalPhase.failed(message: 'Offline.')),
        'Offline.',
      );
      expect(
        l10n.approvalPhase(const ApprovalPhase.failed(message: ' ')),
        l10n.accountApprovalFailed,
      );
    });

    test('a change that went through says what it did', () {
      expect(
        l10n.accountNotice(const AccountNotice.deviceSignedOut(name: 'Mac')),
        'Signed out Mac.',
      );
      expect(
        l10n.accountNotice(const AccountNotice.passkeyAdded(canUnlock: true)),
        contains('can unlock'),
      );
      expect(
        l10n.accountNotice(const AccountNotice.passkeyAdded(canUnlock: false)),
        contains('signs in only'),
      );
      expect(
        l10n.accountNotice(const AccountNotice.passkeyRemoved()),
        'Removed the passkey.',
      );
      expect(
        l10n.accountNotice(
          const AccountNotice.providerLinked(provider: ProviderKind.github),
        ),
        contains('Linked'),
      );
      expect(
        l10n.accountNotice(
          const AccountNotice.providerUnlinked(provider: ProviderKind.github),
        ),
        contains('Unlinked'),
      );
    });

    test('the default server is named, any other by its host', () {
      const base = 'https://sync.kansoapp.app';
      expect(l10n.serverName(base, defaultServer: base), 'Kanso sync');
      expect(l10n.serverName('', defaultServer: base), 'Kanso sync');
      expect(
        l10n.serverName('https://sync.example.com/', defaultServer: base),
        'sync.example.com',
      );
      expect(
        l10n.serverName('http://127.0.0.1:8787', defaultServer: base),
        '127.0.0.1:8787',
      );
    });

    Glados<String>(any.letterOrDigits, kansoExploreConfig()).test(
      'a host has no scheme and no trailing slash',
      (name) {
        final host = hostOf('https://' + name + '.example.com/');
        expect(host, name + '.example.com');
        expect(hostOf(host), host);
      },
    );
  });
}
