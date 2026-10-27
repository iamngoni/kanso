// Kanso — device_approval_bloc_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/state/device_approval/device_approval_bloc.dart';

import '../../../helpers/fixture_approvals_repository.dart';

ApprovalPrompt _new() => readyPrompt(
  id: 'n',
  deviceName: null,
  code: null,
  state: PromptState.new_,
);

void main() {
  late FixtureApprovalsRepository approvals;

  setUp(() => approvals = FixtureApprovalsRepository());

  DeviceApprovalBloc make() => DeviceApprovalBloc(approvals);

  DeviceApprovalEvent conditions({
    required bool signedIn,
    required bool foreground,
  }) => DeviceApprovalEvent.conditionsChanged(
    signedIn: signedIn,
    foreground: foreground,
  );

  group('DeviceApprovalBloc, watching', () {
    test('nothing is watched while signed out, or behind', () async {
      final bloc = make()
        ..add(conditions(signedIn: false, foreground: true))
        ..add(conditions(signedIn: true, foreground: false));
      await pumpEventQueue();
      expect(approvals.watches, 0);
      expect(bloc.state.prompts, isEmpty);
      await bloc.close();
    });

    test('requests are watched while signed in and in front', () async {
      approvals.prompts = <ApprovalPrompt>[readyPrompt()];
      final bloc = make()..add(conditions(signedIn: true, foreground: true));
      await bloc.stream.firstWhere((state) => state.prompts.isNotEmpty);
      expect(approvals.watches, 1);
      expect(bloc.state.shown?.id, 'a1');

      approvals.change(<ApprovalPrompt>[
        readyPrompt(),
        readyPrompt(id: 'a2', deviceName: 'Mac'),
      ]);
      await bloc.stream.firstWhere((state) => state.prompts.length == 2);
      expect(bloc.state.shown?.id, 'a1');
      await bloc.close();
    });

    test(
      'going behind forgets the requests, and coming back watches again',
      () async {
        approvals.prompts = <ApprovalPrompt>[readyPrompt()];
        final bloc = make()..add(conditions(signedIn: true, foreground: true));
        await bloc.stream.firstWhere((state) => state.prompts.isNotEmpty);
        bloc.add(conditions(signedIn: true, foreground: false));
        await bloc.stream.firstWhere((state) => state.prompts.isEmpty);
        bloc.add(conditions(signedIn: true, foreground: true));
        await bloc.stream.firstWhere((state) => state.prompts.isNotEmpty);
        expect(approvals.watches, 2);
        await bloc.close();
      },
    );

    test('a request that is not ready yet is not shown', () async {
      approvals.prompts = <ApprovalPrompt>[
        _new(),
        readyPrompt(id: 'c', code: null, state: PromptState.challenged),
      ];
      final bloc = make()..add(conditions(signedIn: true, foreground: true));
      await bloc.stream.firstWhere((state) => state.prompts.length == 2);
      expect(bloc.state.shown, isNull);

      approvals.change(<ApprovalPrompt>[_new(), readyPrompt(id: 'r')]);
      await bloc.stream.firstWhere((state) => state.shown != null);
      expect(bloc.state.shown?.id, 'r');
      await bloc.close();
    });
  });

  group('DeviceApprovalBloc, answering', () {
    Future<DeviceApprovalBloc> watching(List<ApprovalPrompt> prompts) async {
      approvals.prompts = prompts;
      final bloc = make()..add(conditions(signedIn: true, foreground: true));
      await bloc.stream.firstWhere((state) => state.prompts.isNotEmpty);
      return bloc;
    }

    test('approving answers the request and takes it off', () async {
      final bloc = await watching(<ApprovalPrompt>[readyPrompt()]);
      bloc.add(const DeviceApprovalEvent.approved('a1'));
      await bloc.stream.firstWhere((state) => state.prompts.isEmpty);
      expect(approvals.answers, <String>['approve a1']);
      expect(bloc.state.answering, isNull);
      await bloc.close();
    });

    test('declining answers it the other way', () async {
      final bloc = await watching(<ApprovalPrompt>[readyPrompt()]);
      bloc.add(const DeviceApprovalEvent.declined('a1'));
      await bloc.stream.firstWhere((state) => state.prompts.isEmpty);
      expect(approvals.answers, <String>['decline a1']);
      await bloc.close();
    });

    test(
      'a request that changed after it was made is never approved',
      () async {
        final bloc = await watching(<ApprovalPrompt>[
          readyPrompt(state: PromptState.changed),
        ]);
        expect(bloc.state.shown?.state, PromptState.changed);
        bloc.add(const DeviceApprovalEvent.approved('a1'));
        await pumpEventQueue();
        expect(approvals.answers, isEmpty);

        bloc.add(const DeviceApprovalEvent.declined('a1'));
        await bloc.stream.firstWhere((state) => state.prompts.isEmpty);
        expect(approvals.answers, <String>['decline a1']);
        await bloc.close();
      },
    );

    test('a request nobody asked is not answered', () async {
      final bloc = await watching(<ApprovalPrompt>[readyPrompt()]);
      bloc.add(const DeviceApprovalEvent.approved('nope'));
      await pumpEventQueue();
      expect(approvals.answers, isEmpty);
      await bloc.close();
    });

    test('an answer that failed is the error, and the request stays', () async {
      approvals.failure = const ApplicationError.network(
        message: 'Can’t reach the server.',
      );
      final bloc = await watching(<ApprovalPrompt>[readyPrompt()]);
      bloc.add(const DeviceApprovalEvent.approved('a1'));
      await bloc.stream.firstWhere((state) => state.error != null);
      expect(bloc.state.prompts, hasLength(1));
      expect(bloc.state.answering, isNull);
      await bloc.close();
    });
  });
}
