// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'account_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AccountEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AccountEvent()';
}


}

/// @nodoc
class $AccountEventCopyWith<$Res>  {
$AccountEventCopyWith(AccountEvent _, $Res Function(AccountEvent) __);
}


/// Adds pattern-matching-related methods to [AccountEvent].
extension AccountEventPatterns on AccountEvent {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AccountStartedEvent value)?  started,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AccountStartedEvent() when started != null:
return started(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AccountStartedEvent value)  started,}){
final _that = this;
switch (_that) {
case AccountStartedEvent():
return started(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AccountStartedEvent value)?  started,}){
final _that = this;
switch (_that) {
case AccountStartedEvent() when started != null:
return started(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AccountStartedEvent() when started != null:
return started();case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,}) {final _that = this;
switch (_that) {
case AccountStartedEvent():
return started();}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,}) {final _that = this;
switch (_that) {
case AccountStartedEvent() when started != null:
return started();case _:
  return null;

}
}

}

/// @nodoc


class AccountStartedEvent implements AccountEvent {
  const AccountStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AccountEvent.started()';
}


}




/// @nodoc
mixin _$AccountBlocState {

/// The account now; null until it is first read.
 AccountState? get account;
/// Create a copy of AccountBlocState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AccountBlocStateCopyWith<AccountBlocState> get copyWith => _$AccountBlocStateCopyWithImpl<AccountBlocState>(this as AccountBlocState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountBlocState&&(identical(other.account, account) || other.account == account));
}


@override
int get hashCode => Object.hash(runtimeType,account);

@override
String toString() {
  return 'AccountBlocState(account: $account)';
}


}

/// @nodoc
abstract mixin class $AccountBlocStateCopyWith<$Res>  {
  factory $AccountBlocStateCopyWith(AccountBlocState value, $Res Function(AccountBlocState) _then) = _$AccountBlocStateCopyWithImpl;
@useResult
$Res call({
 AccountState? account
});


$AccountStateCopyWith<$Res>? get account;

}
/// @nodoc
class _$AccountBlocStateCopyWithImpl<$Res>
    implements $AccountBlocStateCopyWith<$Res> {
  _$AccountBlocStateCopyWithImpl(this._self, this._then);

  final AccountBlocState _self;
  final $Res Function(AccountBlocState) _then;

/// Create a copy of AccountBlocState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? account = freezed,}) {
  return _then(_self.copyWith(
account: freezed == account ? _self.account : account // ignore: cast_nullable_to_non_nullable
as AccountState?,
  ));
}
/// Create a copy of AccountBlocState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AccountStateCopyWith<$Res>? get account {
    if (_self.account == null) {
    return null;
  }

  return $AccountStateCopyWith<$Res>(_self.account!, (value) {
    return _then(_self.copyWith(account: value));
  });
}
}


/// Adds pattern-matching-related methods to [AccountBlocState].
extension AccountBlocStatePatterns on AccountBlocState {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AccountBlocState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AccountBlocState() when $default != null:
return $default(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AccountBlocState value)  $default,){
final _that = this;
switch (_that) {
case _AccountBlocState():
return $default(_that);case _:
  throw StateError('Unexpected subclass');

}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AccountBlocState value)?  $default,){
final _that = this;
switch (_that) {
case _AccountBlocState() when $default != null:
return $default(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( AccountState? account)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AccountBlocState() when $default != null:
return $default(_that.account);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( AccountState? account)  $default,) {final _that = this;
switch (_that) {
case _AccountBlocState():
return $default(_that.account);case _:
  throw StateError('Unexpected subclass');

}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( AccountState? account)?  $default,) {final _that = this;
switch (_that) {
case _AccountBlocState() when $default != null:
return $default(_that.account);case _:
  return null;

}
}

}

/// @nodoc


class _AccountBlocState extends AccountBlocState {
  const _AccountBlocState({this.account}): super._();
  

/// The account now; null until it is first read.
@override final  AccountState? account;

/// Create a copy of AccountBlocState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AccountBlocStateCopyWith<_AccountBlocState> get copyWith => __$AccountBlocStateCopyWithImpl<_AccountBlocState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AccountBlocState&&(identical(other.account, account) || other.account == account));
}


@override
int get hashCode => Object.hash(runtimeType,account);

@override
String toString() {
  return 'AccountBlocState(account: $account)';
}


}

/// @nodoc
abstract mixin class _$AccountBlocStateCopyWith<$Res> implements $AccountBlocStateCopyWith<$Res> {
  factory _$AccountBlocStateCopyWith(_AccountBlocState value, $Res Function(_AccountBlocState) _then) = __$AccountBlocStateCopyWithImpl;
@override @useResult
$Res call({
 AccountState? account
});


@override $AccountStateCopyWith<$Res>? get account;

}
/// @nodoc
class __$AccountBlocStateCopyWithImpl<$Res>
    implements _$AccountBlocStateCopyWith<$Res> {
  __$AccountBlocStateCopyWithImpl(this._self, this._then);

  final _AccountBlocState _self;
  final $Res Function(_AccountBlocState) _then;

/// Create a copy of AccountBlocState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? account = freezed,}) {
  return _then(_AccountBlocState(
account: freezed == account ? _self.account : account // ignore: cast_nullable_to_non_nullable
as AccountState?,
  ));
}

/// Create a copy of AccountBlocState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AccountStateCopyWith<$Res>? get account {
    if (_self.account == null) {
    return null;
  }

  return $AccountStateCopyWith<$Res>(_self.account!, (value) {
    return _then(_self.copyWith(account: value));
  });
}
}

// dart format on
