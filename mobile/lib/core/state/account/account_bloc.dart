// Kanso — account_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../bridge/api/types.dart';
import '../../models/repos/abs/account_repository.dart';

part 'account_bloc.freezed.dart';
part 'account_event.dart';
part 'account_state.dart';

/// Whether an account is signed in on this phone, signed in but locked, or
/// being signed in: what Settings, the router and the sign-in screens read.
///
/// It follows the library's account, so it changes when a screen of any
/// flow changes it, and when the server ends the session.
@injectable
class AccountBloc extends Bloc<AccountEvent, AccountBlocState> {
  AccountBloc(this._account) : super(const AccountBlocState()) {
    on<AccountStartedEvent>(_onStarted, transformer: restartable());
  }

  final AccountRepository _account;

  Future<void> _onStarted(
    AccountStartedEvent event,
    Emitter<AccountBlocState> emit,
  ) {
    return emit.onEach<AccountState>(
      _account.stateStream(),
      onData: (account) => emit(state.copyWith(account: account)),
    );
  }
}
