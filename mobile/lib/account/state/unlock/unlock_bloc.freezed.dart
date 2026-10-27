// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'unlock_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$UnlockEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UnlockEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'UnlockEvent()';
}


}

/// @nodoc
class $UnlockEventCopyWith<$Res>  {
$UnlockEventCopyWith(UnlockEvent _, $Res Function(UnlockEvent) __);
}


/// Adds pattern-matching-related methods to [UnlockEvent].
extension UnlockEventPatterns on UnlockEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( UnlockStartedEvent value)?  started,TResult Function( UnlockPasswordSubmittedEvent value)?  passwordSubmitted,TResult Function( UnlockApprovalRequestedEvent value)?  approvalRequested,TResult Function( UnlockApprovalCancelledEvent value)?  approvalCancelled,TResult Function( UnlockSignedOutEvent value)?  signedOut,TResult Function( UnlockBindingAnsweredEvent value)?  bindingAnswered,required TResult orElse(),}){
final _that = this;
switch (_that) {
case UnlockStartedEvent() when started != null:
return started(_that);case UnlockPasswordSubmittedEvent() when passwordSubmitted != null:
return passwordSubmitted(_that);case UnlockApprovalRequestedEvent() when approvalRequested != null:
return approvalRequested(_that);case UnlockApprovalCancelledEvent() when approvalCancelled != null:
return approvalCancelled(_that);case UnlockSignedOutEvent() when signedOut != null:
return signedOut(_that);case UnlockBindingAnsweredEvent() when bindingAnswered != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( UnlockStartedEvent value)  started,required TResult Function( UnlockPasswordSubmittedEvent value)  passwordSubmitted,required TResult Function( UnlockApprovalRequestedEvent value)  approvalRequested,required TResult Function( UnlockApprovalCancelledEvent value)  approvalCancelled,required TResult Function( UnlockSignedOutEvent value)  signedOut,required TResult Function( UnlockBindingAnsweredEvent value)  bindingAnswered,}){
final _that = this;
switch (_that) {
case UnlockStartedEvent():
return started(_that);case UnlockPasswordSubmittedEvent():
return passwordSubmitted(_that);case UnlockApprovalRequestedEvent():
return approvalRequested(_that);case UnlockApprovalCancelledEvent():
return approvalCancelled(_that);case UnlockSignedOutEvent():
return signedOut(_that);case UnlockBindingAnsweredEvent():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( UnlockStartedEvent value)?  started,TResult? Function( UnlockPasswordSubmittedEvent value)?  passwordSubmitted,TResult? Function( UnlockApprovalRequestedEvent value)?  approvalRequested,TResult? Function( UnlockApprovalCancelledEvent value)?  approvalCancelled,TResult? Function( UnlockSignedOutEvent value)?  signedOut,TResult? Function( UnlockBindingAnsweredEvent value)?  bindingAnswered,}){
final _that = this;
switch (_that) {
case UnlockStartedEvent() when started != null:
return started(_that);case UnlockPasswordSubmittedEvent() when passwordSubmitted != null:
return passwordSubmitted(_that);case UnlockApprovalRequestedEvent() when approvalRequested != null:
return approvalRequested(_that);case UnlockApprovalCancelledEvent() when approvalCancelled != null:
return approvalCancelled(_that);case UnlockSignedOutEvent() when signedOut != null:
return signedOut(_that);case UnlockBindingAnsweredEvent() when bindingAnswered != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( bool resumeApproval,  String deviceName)?  started,TResult Function( String username,  String password)?  passwordSubmitted,TResult Function( String deviceName)?  approvalRequested,TResult Function()?  approvalCancelled,TResult Function()?  signedOut,TResult Function( BindingAnswer? answer)?  bindingAnswered,required TResult orElse(),}) {final _that = this;
switch (_that) {
case UnlockStartedEvent() when started != null:
return started(_that.resumeApproval,_that.deviceName);case UnlockPasswordSubmittedEvent() when passwordSubmitted != null:
return passwordSubmitted(_that.username,_that.password);case UnlockApprovalRequestedEvent() when approvalRequested != null:
return approvalRequested(_that.deviceName);case UnlockApprovalCancelledEvent() when approvalCancelled != null:
return approvalCancelled();case UnlockSignedOutEvent() when signedOut != null:
return signedOut();case UnlockBindingAnsweredEvent() when bindingAnswered != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( bool resumeApproval,  String deviceName)  started,required TResult Function( String username,  String password)  passwordSubmitted,required TResult Function( String deviceName)  approvalRequested,required TResult Function()  approvalCancelled,required TResult Function()  signedOut,required TResult Function( BindingAnswer? answer)  bindingAnswered,}) {final _that = this;
switch (_that) {
case UnlockStartedEvent():
return started(_that.resumeApproval,_that.deviceName);case UnlockPasswordSubmittedEvent():
return passwordSubmitted(_that.username,_that.password);case UnlockApprovalRequestedEvent():
return approvalRequested(_that.deviceName);case UnlockApprovalCancelledEvent():
return approvalCancelled();case UnlockSignedOutEvent():
return signedOut();case UnlockBindingAnsweredEvent():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( bool resumeApproval,  String deviceName)?  started,TResult? Function( String username,  String password)?  passwordSubmitted,TResult? Function( String deviceName)?  approvalRequested,TResult? Function()?  approvalCancelled,TResult? Function()?  signedOut,TResult? Function( BindingAnswer? answer)?  bindingAnswered,}) {final _that = this;
switch (_that) {
case UnlockStartedEvent() when started != null:
return started(_that.resumeApproval,_that.deviceName);case UnlockPasswordSubmittedEvent() when passwordSubmitted != null:
return passwordSubmitted(_that.username,_that.password);case UnlockApprovalRequestedEvent() when approvalRequested != null:
return approvalRequested(_that.deviceName);case UnlockApprovalCancelledEvent() when approvalCancelled != null:
return approvalCancelled();case UnlockSignedOutEvent() when signedOut != null:
return signedOut();case UnlockBindingAnsweredEvent() when bindingAnswered != null:
return bindingAnswered(_that.answer);case _:
  return null;

}
}

}

/// @nodoc


class UnlockStartedEvent implements UnlockEvent {
  const UnlockStartedEvent({required this.resumeApproval, required this.deviceName});
  

 final  bool resumeApproval;
 final  String deviceName;

/// Create a copy of UnlockEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$UnlockStartedEventCopyWith<UnlockStartedEvent> get copyWith => _$UnlockStartedEventCopyWithImpl<UnlockStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UnlockStartedEvent&&(identical(other.resumeApproval, resumeApproval) || other.resumeApproval == resumeApproval)&&(identical(other.deviceName, deviceName) || other.deviceName == deviceName));
}


@override
int get hashCode => Object.hash(runtimeType,resumeApproval,deviceName);

@override
String toString() {
  return 'UnlockEvent.started(resumeApproval: $resumeApproval, deviceName: $deviceName)';
}


}

/// @nodoc
abstract mixin class $UnlockStartedEventCopyWith<$Res> implements $UnlockEventCopyWith<$Res> {
  factory $UnlockStartedEventCopyWith(UnlockStartedEvent value, $Res Function(UnlockStartedEvent) _then) = _$UnlockStartedEventCopyWithImpl;
@useResult
$Res call({
 bool resumeApproval, String deviceName
});




}
/// @nodoc
class _$UnlockStartedEventCopyWithImpl<$Res>
    implements $UnlockStartedEventCopyWith<$Res> {
  _$UnlockStartedEventCopyWithImpl(this._self, this._then);

  final UnlockStartedEvent _self;
  final $Res Function(UnlockStartedEvent) _then;

/// Create a copy of UnlockEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? resumeApproval = null,Object? deviceName = null,}) {
  return _then(UnlockStartedEvent(
resumeApproval: null == resumeApproval ? _self.resumeApproval : resumeApproval // ignore: cast_nullable_to_non_nullable
as bool,deviceName: null == deviceName ? _self.deviceName : deviceName // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class UnlockPasswordSubmittedEvent implements UnlockEvent {
  const UnlockPasswordSubmittedEvent({required this.username, required this.password});
  

 final  String username;
 final  String password;

/// Create a copy of UnlockEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$UnlockPasswordSubmittedEventCopyWith<UnlockPasswordSubmittedEvent> get copyWith => _$UnlockPasswordSubmittedEventCopyWithImpl<UnlockPasswordSubmittedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UnlockPasswordSubmittedEvent&&(identical(other.username, username) || other.username == username)&&(identical(other.password, password) || other.password == password));
}


@override
int get hashCode => Object.hash(runtimeType,username,password);

@override
String toString() {
  return 'UnlockEvent.passwordSubmitted(username: $username, password: $password)';
}


}

/// @nodoc
abstract mixin class $UnlockPasswordSubmittedEventCopyWith<$Res> implements $UnlockEventCopyWith<$Res> {
  factory $UnlockPasswordSubmittedEventCopyWith(UnlockPasswordSubmittedEvent value, $Res Function(UnlockPasswordSubmittedEvent) _then) = _$UnlockPasswordSubmittedEventCopyWithImpl;
@useResult
$Res call({
 String username, String password
});




}
/// @nodoc
class _$UnlockPasswordSubmittedEventCopyWithImpl<$Res>
    implements $UnlockPasswordSubmittedEventCopyWith<$Res> {
  _$UnlockPasswordSubmittedEventCopyWithImpl(this._self, this._then);

  final UnlockPasswordSubmittedEvent _self;
  final $Res Function(UnlockPasswordSubmittedEvent) _then;

/// Create a copy of UnlockEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? username = null,Object? password = null,}) {
  return _then(UnlockPasswordSubmittedEvent(
username: null == username ? _self.username : username // ignore: cast_nullable_to_non_nullable
as String,password: null == password ? _self.password : password // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class UnlockApprovalRequestedEvent implements UnlockEvent {
  const UnlockApprovalRequestedEvent({required this.deviceName});
  

 final  String deviceName;

/// Create a copy of UnlockEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$UnlockApprovalRequestedEventCopyWith<UnlockApprovalRequestedEvent> get copyWith => _$UnlockApprovalRequestedEventCopyWithImpl<UnlockApprovalRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UnlockApprovalRequestedEvent&&(identical(other.deviceName, deviceName) || other.deviceName == deviceName));
}


@override
int get hashCode => Object.hash(runtimeType,deviceName);

@override
String toString() {
  return 'UnlockEvent.approvalRequested(deviceName: $deviceName)';
}


}

/// @nodoc
abstract mixin class $UnlockApprovalRequestedEventCopyWith<$Res> implements $UnlockEventCopyWith<$Res> {
  factory $UnlockApprovalRequestedEventCopyWith(UnlockApprovalRequestedEvent value, $Res Function(UnlockApprovalRequestedEvent) _then) = _$UnlockApprovalRequestedEventCopyWithImpl;
@useResult
$Res call({
 String deviceName
});




}
/// @nodoc
class _$UnlockApprovalRequestedEventCopyWithImpl<$Res>
    implements $UnlockApprovalRequestedEventCopyWith<$Res> {
  _$UnlockApprovalRequestedEventCopyWithImpl(this._self, this._then);

  final UnlockApprovalRequestedEvent _self;
  final $Res Function(UnlockApprovalRequestedEvent) _then;

/// Create a copy of UnlockEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? deviceName = null,}) {
  return _then(UnlockApprovalRequestedEvent(
deviceName: null == deviceName ? _self.deviceName : deviceName // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class UnlockApprovalCancelledEvent implements UnlockEvent {
  const UnlockApprovalCancelledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UnlockApprovalCancelledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'UnlockEvent.approvalCancelled()';
}


}




/// @nodoc


class UnlockSignedOutEvent implements UnlockEvent {
  const UnlockSignedOutEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UnlockSignedOutEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'UnlockEvent.signedOut()';
}


}




/// @nodoc


class UnlockBindingAnsweredEvent implements UnlockEvent {
  const UnlockBindingAnsweredEvent(this.answer);
  

 final  BindingAnswer? answer;

/// Create a copy of UnlockEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$UnlockBindingAnsweredEventCopyWith<UnlockBindingAnsweredEvent> get copyWith => _$UnlockBindingAnsweredEventCopyWithImpl<UnlockBindingAnsweredEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UnlockBindingAnsweredEvent&&(identical(other.answer, answer) || other.answer == answer));
}


@override
int get hashCode => Object.hash(runtimeType,answer);

@override
String toString() {
  return 'UnlockEvent.bindingAnswered(answer: $answer)';
}


}

/// @nodoc
abstract mixin class $UnlockBindingAnsweredEventCopyWith<$Res> implements $UnlockEventCopyWith<$Res> {
  factory $UnlockBindingAnsweredEventCopyWith(UnlockBindingAnsweredEvent value, $Res Function(UnlockBindingAnsweredEvent) _then) = _$UnlockBindingAnsweredEventCopyWithImpl;
@useResult
$Res call({
 BindingAnswer? answer
});




}
/// @nodoc
class _$UnlockBindingAnsweredEventCopyWithImpl<$Res>
    implements $UnlockBindingAnsweredEventCopyWith<$Res> {
  _$UnlockBindingAnsweredEventCopyWithImpl(this._self, this._then);

  final UnlockBindingAnsweredEvent _self;
  final $Res Function(UnlockBindingAnsweredEvent) _then;

/// Create a copy of UnlockEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? answer = freezed,}) {
  return _then(UnlockBindingAnsweredEvent(
freezed == answer ? _self.answer : answer // ignore: cast_nullable_to_non_nullable
as BindingAnswer?,
  ));
}


}

/// @nodoc
mixin _$UnlockState {

/// Whether a password is being tried, or the phone is signing out.
 bool get busy;/// What is wrong with the password typed.
 FormProblem? get problem;/// Why the last call failed.
 ApplicationError? get error;/// The wait for another device; non-null while the screen shows it.
 ApprovalWait? get approval;/// The question about the notes on this phone, while it waits.
 BindingQuestion? get question;/// How the unlock ended; null while it has not.
 AccountOutcome? get outcome;
/// Create a copy of UnlockState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$UnlockStateCopyWith<UnlockState> get copyWith => _$UnlockStateCopyWithImpl<UnlockState>(this as UnlockState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UnlockState&&(identical(other.busy, busy) || other.busy == busy)&&(identical(other.problem, problem) || other.problem == problem)&&(identical(other.error, error) || other.error == error)&&(identical(other.approval, approval) || other.approval == approval)&&(identical(other.question, question) || other.question == question)&&(identical(other.outcome, outcome) || other.outcome == outcome));
}


@override
int get hashCode => Object.hash(runtimeType,busy,problem,error,approval,question,outcome);

@override
String toString() {
  return 'UnlockState(busy: $busy, problem: $problem, error: $error, approval: $approval, question: $question, outcome: $outcome)';
}


}

/// @nodoc
abstract mixin class $UnlockStateCopyWith<$Res>  {
  factory $UnlockStateCopyWith(UnlockState value, $Res Function(UnlockState) _then) = _$UnlockStateCopyWithImpl;
@useResult
$Res call({
 bool busy, FormProblem? problem, ApplicationError? error, ApprovalWait? approval, BindingQuestion? question, AccountOutcome? outcome
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$UnlockStateCopyWithImpl<$Res>
    implements $UnlockStateCopyWith<$Res> {
  _$UnlockStateCopyWithImpl(this._self, this._then);

  final UnlockState _self;
  final $Res Function(UnlockState) _then;

/// Create a copy of UnlockState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? busy = null,Object? problem = freezed,Object? error = freezed,Object? approval = freezed,Object? question = freezed,Object? outcome = freezed,}) {
  return _then(_self.copyWith(
busy: null == busy ? _self.busy : busy // ignore: cast_nullable_to_non_nullable
as bool,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as FormProblem?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,approval: freezed == approval ? _self.approval : approval // ignore: cast_nullable_to_non_nullable
as ApprovalWait?,question: freezed == question ? _self.question : question // ignore: cast_nullable_to_non_nullable
as BindingQuestion?,outcome: freezed == outcome ? _self.outcome : outcome // ignore: cast_nullable_to_non_nullable
as AccountOutcome?,
  ));
}
/// Create a copy of UnlockState
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


/// Adds pattern-matching-related methods to [UnlockState].
extension UnlockStatePatterns on UnlockState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _UnlockState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _UnlockState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _UnlockState value)  $default,){
final _that = this;
switch (_that) {
case _UnlockState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _UnlockState value)?  $default,){
final _that = this;
switch (_that) {
case _UnlockState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool busy,  FormProblem? problem,  ApplicationError? error,  ApprovalWait? approval,  BindingQuestion? question,  AccountOutcome? outcome)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _UnlockState() when $default != null:
return $default(_that.busy,_that.problem,_that.error,_that.approval,_that.question,_that.outcome);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool busy,  FormProblem? problem,  ApplicationError? error,  ApprovalWait? approval,  BindingQuestion? question,  AccountOutcome? outcome)  $default,) {final _that = this;
switch (_that) {
case _UnlockState():
return $default(_that.busy,_that.problem,_that.error,_that.approval,_that.question,_that.outcome);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool busy,  FormProblem? problem,  ApplicationError? error,  ApprovalWait? approval,  BindingQuestion? question,  AccountOutcome? outcome)?  $default,) {final _that = this;
switch (_that) {
case _UnlockState() when $default != null:
return $default(_that.busy,_that.problem,_that.error,_that.approval,_that.question,_that.outcome);case _:
  return null;

}
}

}

/// @nodoc


class _UnlockState implements UnlockState {
  const _UnlockState({this.busy = false, this.problem, this.error, this.approval, this.question, this.outcome});
  

/// Whether a password is being tried, or the phone is signing out.
@override@JsonKey() final  bool busy;
/// What is wrong with the password typed.
@override final  FormProblem? problem;
/// Why the last call failed.
@override final  ApplicationError? error;
/// The wait for another device; non-null while the screen shows it.
@override final  ApprovalWait? approval;
/// The question about the notes on this phone, while it waits.
@override final  BindingQuestion? question;
/// How the unlock ended; null while it has not.
@override final  AccountOutcome? outcome;

/// Create a copy of UnlockState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$UnlockStateCopyWith<_UnlockState> get copyWith => __$UnlockStateCopyWithImpl<_UnlockState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _UnlockState&&(identical(other.busy, busy) || other.busy == busy)&&(identical(other.problem, problem) || other.problem == problem)&&(identical(other.error, error) || other.error == error)&&(identical(other.approval, approval) || other.approval == approval)&&(identical(other.question, question) || other.question == question)&&(identical(other.outcome, outcome) || other.outcome == outcome));
}


@override
int get hashCode => Object.hash(runtimeType,busy,problem,error,approval,question,outcome);

@override
String toString() {
  return 'UnlockState(busy: $busy, problem: $problem, error: $error, approval: $approval, question: $question, outcome: $outcome)';
}


}

/// @nodoc
abstract mixin class _$UnlockStateCopyWith<$Res> implements $UnlockStateCopyWith<$Res> {
  factory _$UnlockStateCopyWith(_UnlockState value, $Res Function(_UnlockState) _then) = __$UnlockStateCopyWithImpl;
@override @useResult
$Res call({
 bool busy, FormProblem? problem, ApplicationError? error, ApprovalWait? approval, BindingQuestion? question, AccountOutcome? outcome
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$UnlockStateCopyWithImpl<$Res>
    implements _$UnlockStateCopyWith<$Res> {
  __$UnlockStateCopyWithImpl(this._self, this._then);

  final _UnlockState _self;
  final $Res Function(_UnlockState) _then;

/// Create a copy of UnlockState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? busy = null,Object? problem = freezed,Object? error = freezed,Object? approval = freezed,Object? question = freezed,Object? outcome = freezed,}) {
  return _then(_UnlockState(
busy: null == busy ? _self.busy : busy // ignore: cast_nullable_to_non_nullable
as bool,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as FormProblem?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,approval: freezed == approval ? _self.approval : approval // ignore: cast_nullable_to_non_nullable
as ApprovalWait?,question: freezed == question ? _self.question : question // ignore: cast_nullable_to_non_nullable
as BindingQuestion?,outcome: freezed == outcome ? _self.outcome : outcome // ignore: cast_nullable_to_non_nullable
as AccountOutcome?,
  ));
}

/// Create a copy of UnlockState
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
