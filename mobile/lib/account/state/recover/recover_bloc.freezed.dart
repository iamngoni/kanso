// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'recover_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$RecoverEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RecoverEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'RecoverEvent()';
}


}

/// @nodoc
class $RecoverEventCopyWith<$Res>  {
$RecoverEventCopyWith(RecoverEvent _, $Res Function(RecoverEvent) __);
}


/// Adds pattern-matching-related methods to [RecoverEvent].
extension RecoverEventPatterns on RecoverEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( RecoverStartedEvent value)?  started,TResult Function( RecoverSubmittedEvent value)?  submitted,TResult Function( RecoverBindingAnsweredEvent value)?  bindingAnswered,required TResult orElse(),}){
final _that = this;
switch (_that) {
case RecoverStartedEvent() when started != null:
return started(_that);case RecoverSubmittedEvent() when submitted != null:
return submitted(_that);case RecoverBindingAnsweredEvent() when bindingAnswered != null:
return bindingAnswered(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( RecoverStartedEvent value)  started,required TResult Function( RecoverSubmittedEvent value)  submitted,required TResult Function( RecoverBindingAnsweredEvent value)  bindingAnswered,}){
final _that = this;
switch (_that) {
case RecoverStartedEvent():
return started(_that);case RecoverSubmittedEvent():
return submitted(_that);case RecoverBindingAnsweredEvent():
return bindingAnswered(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( RecoverStartedEvent value)?  started,TResult? Function( RecoverSubmittedEvent value)?  submitted,TResult? Function( RecoverBindingAnsweredEvent value)?  bindingAnswered,}){
final _that = this;
switch (_that) {
case RecoverStartedEvent() when started != null:
return started(_that);case RecoverSubmittedEvent() when submitted != null:
return submitted(_that);case RecoverBindingAnsweredEvent() when bindingAnswered != null:
return bindingAnswered(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String? server)?  started,TResult Function( String username,  String recoveryKey,  String password,  String repeat,  String deviceName)?  submitted,TResult Function( BindingAnswer? answer)?  bindingAnswered,required TResult orElse(),}) {final _that = this;
switch (_that) {
case RecoverStartedEvent() when started != null:
return started(_that.server);case RecoverSubmittedEvent() when submitted != null:
return submitted(_that.username,_that.recoveryKey,_that.password,_that.repeat,_that.deviceName);case RecoverBindingAnsweredEvent() when bindingAnswered != null:
return bindingAnswered(_that.answer);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String? server)  started,required TResult Function( String username,  String recoveryKey,  String password,  String repeat,  String deviceName)  submitted,required TResult Function( BindingAnswer? answer)  bindingAnswered,}) {final _that = this;
switch (_that) {
case RecoverStartedEvent():
return started(_that.server);case RecoverSubmittedEvent():
return submitted(_that.username,_that.recoveryKey,_that.password,_that.repeat,_that.deviceName);case RecoverBindingAnsweredEvent():
return bindingAnswered(_that.answer);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String? server)?  started,TResult? Function( String username,  String recoveryKey,  String password,  String repeat,  String deviceName)?  submitted,TResult? Function( BindingAnswer? answer)?  bindingAnswered,}) {final _that = this;
switch (_that) {
case RecoverStartedEvent() when started != null:
return started(_that.server);case RecoverSubmittedEvent() when submitted != null:
return submitted(_that.username,_that.recoveryKey,_that.password,_that.repeat,_that.deviceName);case RecoverBindingAnsweredEvent() when bindingAnswered != null:
return bindingAnswered(_that.answer);case _:
  return null;

}
}

}

/// @nodoc


class RecoverStartedEvent implements RecoverEvent {
  const RecoverStartedEvent({this.server});
  

 final  String? server;

/// Create a copy of RecoverEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RecoverStartedEventCopyWith<RecoverStartedEvent> get copyWith => _$RecoverStartedEventCopyWithImpl<RecoverStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RecoverStartedEvent&&(identical(other.server, server) || other.server == server));
}


@override
int get hashCode => Object.hash(runtimeType,server);

@override
String toString() {
  return 'RecoverEvent.started(server: $server)';
}


}

/// @nodoc
abstract mixin class $RecoverStartedEventCopyWith<$Res> implements $RecoverEventCopyWith<$Res> {
  factory $RecoverStartedEventCopyWith(RecoverStartedEvent value, $Res Function(RecoverStartedEvent) _then) = _$RecoverStartedEventCopyWithImpl;
@useResult
$Res call({
 String? server
});




}
/// @nodoc
class _$RecoverStartedEventCopyWithImpl<$Res>
    implements $RecoverStartedEventCopyWith<$Res> {
  _$RecoverStartedEventCopyWithImpl(this._self, this._then);

  final RecoverStartedEvent _self;
  final $Res Function(RecoverStartedEvent) _then;

/// Create a copy of RecoverEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? server = freezed,}) {
  return _then(RecoverStartedEvent(
server: freezed == server ? _self.server : server // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class RecoverSubmittedEvent implements RecoverEvent {
  const RecoverSubmittedEvent({required this.username, required this.recoveryKey, required this.password, required this.repeat, required this.deviceName});
  

 final  String username;
 final  String recoveryKey;
 final  String password;
 final  String repeat;
 final  String deviceName;

/// Create a copy of RecoverEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RecoverSubmittedEventCopyWith<RecoverSubmittedEvent> get copyWith => _$RecoverSubmittedEventCopyWithImpl<RecoverSubmittedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RecoverSubmittedEvent&&(identical(other.username, username) || other.username == username)&&(identical(other.recoveryKey, recoveryKey) || other.recoveryKey == recoveryKey)&&(identical(other.password, password) || other.password == password)&&(identical(other.repeat, repeat) || other.repeat == repeat)&&(identical(other.deviceName, deviceName) || other.deviceName == deviceName));
}


@override
int get hashCode => Object.hash(runtimeType,username,recoveryKey,password,repeat,deviceName);

@override
String toString() {
  return 'RecoverEvent.submitted(username: $username, recoveryKey: $recoveryKey, password: $password, repeat: $repeat, deviceName: $deviceName)';
}


}

/// @nodoc
abstract mixin class $RecoverSubmittedEventCopyWith<$Res> implements $RecoverEventCopyWith<$Res> {
  factory $RecoverSubmittedEventCopyWith(RecoverSubmittedEvent value, $Res Function(RecoverSubmittedEvent) _then) = _$RecoverSubmittedEventCopyWithImpl;
@useResult
$Res call({
 String username, String recoveryKey, String password, String repeat, String deviceName
});




}
/// @nodoc
class _$RecoverSubmittedEventCopyWithImpl<$Res>
    implements $RecoverSubmittedEventCopyWith<$Res> {
  _$RecoverSubmittedEventCopyWithImpl(this._self, this._then);

  final RecoverSubmittedEvent _self;
  final $Res Function(RecoverSubmittedEvent) _then;

/// Create a copy of RecoverEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? username = null,Object? recoveryKey = null,Object? password = null,Object? repeat = null,Object? deviceName = null,}) {
  return _then(RecoverSubmittedEvent(
username: null == username ? _self.username : username // ignore: cast_nullable_to_non_nullable
as String,recoveryKey: null == recoveryKey ? _self.recoveryKey : recoveryKey // ignore: cast_nullable_to_non_nullable
as String,password: null == password ? _self.password : password // ignore: cast_nullable_to_non_nullable
as String,repeat: null == repeat ? _self.repeat : repeat // ignore: cast_nullable_to_non_nullable
as String,deviceName: null == deviceName ? _self.deviceName : deviceName // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class RecoverBindingAnsweredEvent implements RecoverEvent {
  const RecoverBindingAnsweredEvent(this.answer);
  

 final  BindingAnswer? answer;

/// Create a copy of RecoverEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RecoverBindingAnsweredEventCopyWith<RecoverBindingAnsweredEvent> get copyWith => _$RecoverBindingAnsweredEventCopyWithImpl<RecoverBindingAnsweredEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RecoverBindingAnsweredEvent&&(identical(other.answer, answer) || other.answer == answer));
}


@override
int get hashCode => Object.hash(runtimeType,answer);

@override
String toString() {
  return 'RecoverEvent.bindingAnswered(answer: $answer)';
}


}

/// @nodoc
abstract mixin class $RecoverBindingAnsweredEventCopyWith<$Res> implements $RecoverEventCopyWith<$Res> {
  factory $RecoverBindingAnsweredEventCopyWith(RecoverBindingAnsweredEvent value, $Res Function(RecoverBindingAnsweredEvent) _then) = _$RecoverBindingAnsweredEventCopyWithImpl;
@useResult
$Res call({
 BindingAnswer? answer
});




}
/// @nodoc
class _$RecoverBindingAnsweredEventCopyWithImpl<$Res>
    implements $RecoverBindingAnsweredEventCopyWith<$Res> {
  _$RecoverBindingAnsweredEventCopyWithImpl(this._self, this._then);

  final RecoverBindingAnsweredEvent _self;
  final $Res Function(RecoverBindingAnsweredEvent) _then;

/// Create a copy of RecoverEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? answer = freezed,}) {
  return _then(RecoverBindingAnsweredEvent(
freezed == answer ? _self.answer : answer // ignore: cast_nullable_to_non_nullable
as BindingAnswer?,
  ));
}


}

/// @nodoc
mixin _$RecoverState {

/// The server of the account.
 String get server;/// Whether a recovery is being made.
 bool get busy;/// What is wrong with what was typed.
 FormProblem? get problem;/// Why the last call failed.
 ApplicationError? get error;/// The question about the notes on this phone, while it waits.
 BindingQuestion? get question;/// The new recovery key, to write down.
 RecoveryKeyText? get recoveryKey;/// How the recovery ended; null while it has not.
 AccountOutcome? get outcome;
/// Create a copy of RecoverState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RecoverStateCopyWith<RecoverState> get copyWith => _$RecoverStateCopyWithImpl<RecoverState>(this as RecoverState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RecoverState&&(identical(other.server, server) || other.server == server)&&(identical(other.busy, busy) || other.busy == busy)&&(identical(other.problem, problem) || other.problem == problem)&&(identical(other.error, error) || other.error == error)&&(identical(other.question, question) || other.question == question)&&(identical(other.recoveryKey, recoveryKey) || other.recoveryKey == recoveryKey)&&(identical(other.outcome, outcome) || other.outcome == outcome));
}


@override
int get hashCode => Object.hash(runtimeType,server,busy,problem,error,question,recoveryKey,outcome);

@override
String toString() {
  return 'RecoverState(server: $server, busy: $busy, problem: $problem, error: $error, question: $question, recoveryKey: $recoveryKey, outcome: $outcome)';
}


}

/// @nodoc
abstract mixin class $RecoverStateCopyWith<$Res>  {
  factory $RecoverStateCopyWith(RecoverState value, $Res Function(RecoverState) _then) = _$RecoverStateCopyWithImpl;
@useResult
$Res call({
 String server, bool busy, FormProblem? problem, ApplicationError? error, BindingQuestion? question, RecoveryKeyText? recoveryKey, AccountOutcome? outcome
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$RecoverStateCopyWithImpl<$Res>
    implements $RecoverStateCopyWith<$Res> {
  _$RecoverStateCopyWithImpl(this._self, this._then);

  final RecoverState _self;
  final $Res Function(RecoverState) _then;

/// Create a copy of RecoverState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? server = null,Object? busy = null,Object? problem = freezed,Object? error = freezed,Object? question = freezed,Object? recoveryKey = freezed,Object? outcome = freezed,}) {
  return _then(_self.copyWith(
server: null == server ? _self.server : server // ignore: cast_nullable_to_non_nullable
as String,busy: null == busy ? _self.busy : busy // ignore: cast_nullable_to_non_nullable
as bool,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as FormProblem?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,question: freezed == question ? _self.question : question // ignore: cast_nullable_to_non_nullable
as BindingQuestion?,recoveryKey: freezed == recoveryKey ? _self.recoveryKey : recoveryKey // ignore: cast_nullable_to_non_nullable
as RecoveryKeyText?,outcome: freezed == outcome ? _self.outcome : outcome // ignore: cast_nullable_to_non_nullable
as AccountOutcome?,
  ));
}
/// Create a copy of RecoverState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get error {
    if (_self.error == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.error!, (value) {
    return _then(_self.copyWith(error: value));
  });
}
}


/// Adds pattern-matching-related methods to [RecoverState].
extension RecoverStatePatterns on RecoverState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _RecoverState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _RecoverState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _RecoverState value)  $default,){
final _that = this;
switch (_that) {
case _RecoverState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _RecoverState value)?  $default,){
final _that = this;
switch (_that) {
case _RecoverState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String server,  bool busy,  FormProblem? problem,  ApplicationError? error,  BindingQuestion? question,  RecoveryKeyText? recoveryKey,  AccountOutcome? outcome)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _RecoverState() when $default != null:
return $default(_that.server,_that.busy,_that.problem,_that.error,_that.question,_that.recoveryKey,_that.outcome);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String server,  bool busy,  FormProblem? problem,  ApplicationError? error,  BindingQuestion? question,  RecoveryKeyText? recoveryKey,  AccountOutcome? outcome)  $default,) {final _that = this;
switch (_that) {
case _RecoverState():
return $default(_that.server,_that.busy,_that.problem,_that.error,_that.question,_that.recoveryKey,_that.outcome);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String server,  bool busy,  FormProblem? problem,  ApplicationError? error,  BindingQuestion? question,  RecoveryKeyText? recoveryKey,  AccountOutcome? outcome)?  $default,) {final _that = this;
switch (_that) {
case _RecoverState() when $default != null:
return $default(_that.server,_that.busy,_that.problem,_that.error,_that.question,_that.recoveryKey,_that.outcome);case _:
  return null;

}
}

}

/// @nodoc


class _RecoverState implements RecoverState {
  const _RecoverState({this.server = '', this.busy = false, this.problem, this.error, this.question, this.recoveryKey, this.outcome});
  

/// The server of the account.
@override@JsonKey() final  String server;
/// Whether a recovery is being made.
@override@JsonKey() final  bool busy;
/// What is wrong with what was typed.
@override final  FormProblem? problem;
/// Why the last call failed.
@override final  ApplicationError? error;
/// The question about the notes on this phone, while it waits.
@override final  BindingQuestion? question;
/// The new recovery key, to write down.
@override final  RecoveryKeyText? recoveryKey;
/// How the recovery ended; null while it has not.
@override final  AccountOutcome? outcome;

/// Create a copy of RecoverState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$RecoverStateCopyWith<_RecoverState> get copyWith => __$RecoverStateCopyWithImpl<_RecoverState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _RecoverState&&(identical(other.server, server) || other.server == server)&&(identical(other.busy, busy) || other.busy == busy)&&(identical(other.problem, problem) || other.problem == problem)&&(identical(other.error, error) || other.error == error)&&(identical(other.question, question) || other.question == question)&&(identical(other.recoveryKey, recoveryKey) || other.recoveryKey == recoveryKey)&&(identical(other.outcome, outcome) || other.outcome == outcome));
}


@override
int get hashCode => Object.hash(runtimeType,server,busy,problem,error,question,recoveryKey,outcome);

@override
String toString() {
  return 'RecoverState(server: $server, busy: $busy, problem: $problem, error: $error, question: $question, recoveryKey: $recoveryKey, outcome: $outcome)';
}


}

/// @nodoc
abstract mixin class _$RecoverStateCopyWith<$Res> implements $RecoverStateCopyWith<$Res> {
  factory _$RecoverStateCopyWith(_RecoverState value, $Res Function(_RecoverState) _then) = __$RecoverStateCopyWithImpl;
@override @useResult
$Res call({
 String server, bool busy, FormProblem? problem, ApplicationError? error, BindingQuestion? question, RecoveryKeyText? recoveryKey, AccountOutcome? outcome
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$RecoverStateCopyWithImpl<$Res>
    implements _$RecoverStateCopyWith<$Res> {
  __$RecoverStateCopyWithImpl(this._self, this._then);

  final _RecoverState _self;
  final $Res Function(_RecoverState) _then;

/// Create a copy of RecoverState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? server = null,Object? busy = null,Object? problem = freezed,Object? error = freezed,Object? question = freezed,Object? recoveryKey = freezed,Object? outcome = freezed,}) {
  return _then(_RecoverState(
server: null == server ? _self.server : server // ignore: cast_nullable_to_non_nullable
as String,busy: null == busy ? _self.busy : busy // ignore: cast_nullable_to_non_nullable
as bool,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as FormProblem?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,question: freezed == question ? _self.question : question // ignore: cast_nullable_to_non_nullable
as BindingQuestion?,recoveryKey: freezed == recoveryKey ? _self.recoveryKey : recoveryKey // ignore: cast_nullable_to_non_nullable
as RecoveryKeyText?,outcome: freezed == outcome ? _self.outcome : outcome // ignore: cast_nullable_to_non_nullable
as AccountOutcome?,
  ));
}

/// Create a copy of RecoverState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get error {
    if (_self.error == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.error!, (value) {
    return _then(_self.copyWith(error: value));
  });
}
}

// dart format on
