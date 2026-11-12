// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'agent_session_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AgentSessionEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentSessionEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AgentSessionEvent()';
}


}

/// @nodoc
class $AgentSessionEventCopyWith<$Res>  {
$AgentSessionEventCopyWith(AgentSessionEvent _, $Res Function(AgentSessionEvent) __);
}


/// Adds pattern-matching-related methods to [AgentSessionEvent].
extension AgentSessionEventPatterns on AgentSessionEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AgentSessionStartedEvent value)?  started,TResult Function( AgentSessionPolledEvent value)?  polled,TResult Function( AgentSessionFollowUpSentEvent value)?  followUpSent,TResult Function( AgentSessionStoppedEvent value)?  stopped,TResult Function( AgentSessionEndedEvent value)?  ended,TResult Function( AgentSessionDismissedEvent value)?  dismissed,TResult Function( AgentSessionUndoneEvent value)?  undone,TResult Function( AgentSessionTroubleClearedEvent value)?  troubleCleared,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AgentSessionStartedEvent() when started != null:
return started(_that);case AgentSessionPolledEvent() when polled != null:
return polled(_that);case AgentSessionFollowUpSentEvent() when followUpSent != null:
return followUpSent(_that);case AgentSessionStoppedEvent() when stopped != null:
return stopped(_that);case AgentSessionEndedEvent() when ended != null:
return ended(_that);case AgentSessionDismissedEvent() when dismissed != null:
return dismissed(_that);case AgentSessionUndoneEvent() when undone != null:
return undone(_that);case AgentSessionTroubleClearedEvent() when troubleCleared != null:
return troubleCleared(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AgentSessionStartedEvent value)  started,required TResult Function( AgentSessionPolledEvent value)  polled,required TResult Function( AgentSessionFollowUpSentEvent value)  followUpSent,required TResult Function( AgentSessionStoppedEvent value)  stopped,required TResult Function( AgentSessionEndedEvent value)  ended,required TResult Function( AgentSessionDismissedEvent value)  dismissed,required TResult Function( AgentSessionUndoneEvent value)  undone,required TResult Function( AgentSessionTroubleClearedEvent value)  troubleCleared,}){
final _that = this;
switch (_that) {
case AgentSessionStartedEvent():
return started(_that);case AgentSessionPolledEvent():
return polled(_that);case AgentSessionFollowUpSentEvent():
return followUpSent(_that);case AgentSessionStoppedEvent():
return stopped(_that);case AgentSessionEndedEvent():
return ended(_that);case AgentSessionDismissedEvent():
return dismissed(_that);case AgentSessionUndoneEvent():
return undone(_that);case AgentSessionTroubleClearedEvent():
return troubleCleared(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AgentSessionStartedEvent value)?  started,TResult? Function( AgentSessionPolledEvent value)?  polled,TResult? Function( AgentSessionFollowUpSentEvent value)?  followUpSent,TResult? Function( AgentSessionStoppedEvent value)?  stopped,TResult? Function( AgentSessionEndedEvent value)?  ended,TResult? Function( AgentSessionDismissedEvent value)?  dismissed,TResult? Function( AgentSessionUndoneEvent value)?  undone,TResult? Function( AgentSessionTroubleClearedEvent value)?  troubleCleared,}){
final _that = this;
switch (_that) {
case AgentSessionStartedEvent() when started != null:
return started(_that);case AgentSessionPolledEvent() when polled != null:
return polled(_that);case AgentSessionFollowUpSentEvent() when followUpSent != null:
return followUpSent(_that);case AgentSessionStoppedEvent() when stopped != null:
return stopped(_that);case AgentSessionEndedEvent() when ended != null:
return ended(_that);case AgentSessionDismissedEvent() when dismissed != null:
return dismissed(_that);case AgentSessionUndoneEvent() when undone != null:
return undone(_that);case AgentSessionTroubleClearedEvent() when troubleCleared != null:
return troubleCleared(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String key)?  started,TResult Function()?  polled,TResult Function( String text)?  followUpSent,TResult Function()?  stopped,TResult Function()?  ended,TResult Function()?  dismissed,TResult Function()?  undone,TResult Function()?  troubleCleared,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AgentSessionStartedEvent() when started != null:
return started(_that.key);case AgentSessionPolledEvent() when polled != null:
return polled();case AgentSessionFollowUpSentEvent() when followUpSent != null:
return followUpSent(_that.text);case AgentSessionStoppedEvent() when stopped != null:
return stopped();case AgentSessionEndedEvent() when ended != null:
return ended();case AgentSessionDismissedEvent() when dismissed != null:
return dismissed();case AgentSessionUndoneEvent() when undone != null:
return undone();case AgentSessionTroubleClearedEvent() when troubleCleared != null:
return troubleCleared();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String key)  started,required TResult Function()  polled,required TResult Function( String text)  followUpSent,required TResult Function()  stopped,required TResult Function()  ended,required TResult Function()  dismissed,required TResult Function()  undone,required TResult Function()  troubleCleared,}) {final _that = this;
switch (_that) {
case AgentSessionStartedEvent():
return started(_that.key);case AgentSessionPolledEvent():
return polled();case AgentSessionFollowUpSentEvent():
return followUpSent(_that.text);case AgentSessionStoppedEvent():
return stopped();case AgentSessionEndedEvent():
return ended();case AgentSessionDismissedEvent():
return dismissed();case AgentSessionUndoneEvent():
return undone();case AgentSessionTroubleClearedEvent():
return troubleCleared();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String key)?  started,TResult? Function()?  polled,TResult? Function( String text)?  followUpSent,TResult? Function()?  stopped,TResult? Function()?  ended,TResult? Function()?  dismissed,TResult? Function()?  undone,TResult? Function()?  troubleCleared,}) {final _that = this;
switch (_that) {
case AgentSessionStartedEvent() when started != null:
return started(_that.key);case AgentSessionPolledEvent() when polled != null:
return polled();case AgentSessionFollowUpSentEvent() when followUpSent != null:
return followUpSent(_that.text);case AgentSessionStoppedEvent() when stopped != null:
return stopped();case AgentSessionEndedEvent() when ended != null:
return ended();case AgentSessionDismissedEvent() when dismissed != null:
return dismissed();case AgentSessionUndoneEvent() when undone != null:
return undone();case AgentSessionTroubleClearedEvent() when troubleCleared != null:
return troubleCleared();case _:
  return null;

}
}

}

/// @nodoc


class AgentSessionStartedEvent implements AgentSessionEvent {
  const AgentSessionStartedEvent(this.key);
  

 final  String key;

/// Create a copy of AgentSessionEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentSessionStartedEventCopyWith<AgentSessionStartedEvent> get copyWith => _$AgentSessionStartedEventCopyWithImpl<AgentSessionStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentSessionStartedEvent&&(identical(other.key, key) || other.key == key));
}


@override
int get hashCode => Object.hash(runtimeType,key);

@override
String toString() {
  return 'AgentSessionEvent.started(key: $key)';
}


}

/// @nodoc
abstract mixin class $AgentSessionStartedEventCopyWith<$Res> implements $AgentSessionEventCopyWith<$Res> {
  factory $AgentSessionStartedEventCopyWith(AgentSessionStartedEvent value, $Res Function(AgentSessionStartedEvent) _then) = _$AgentSessionStartedEventCopyWithImpl;
@useResult
$Res call({
 String key
});




}
/// @nodoc
class _$AgentSessionStartedEventCopyWithImpl<$Res>
    implements $AgentSessionStartedEventCopyWith<$Res> {
  _$AgentSessionStartedEventCopyWithImpl(this._self, this._then);

  final AgentSessionStartedEvent _self;
  final $Res Function(AgentSessionStartedEvent) _then;

/// Create a copy of AgentSessionEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? key = null,}) {
  return _then(AgentSessionStartedEvent(
null == key ? _self.key : key // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AgentSessionPolledEvent implements AgentSessionEvent {
  const AgentSessionPolledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentSessionPolledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AgentSessionEvent.polled()';
}


}




/// @nodoc


class AgentSessionFollowUpSentEvent implements AgentSessionEvent {
  const AgentSessionFollowUpSentEvent(this.text);
  

 final  String text;

/// Create a copy of AgentSessionEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentSessionFollowUpSentEventCopyWith<AgentSessionFollowUpSentEvent> get copyWith => _$AgentSessionFollowUpSentEventCopyWithImpl<AgentSessionFollowUpSentEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentSessionFollowUpSentEvent&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,text);

@override
String toString() {
  return 'AgentSessionEvent.followUpSent(text: $text)';
}


}

/// @nodoc
abstract mixin class $AgentSessionFollowUpSentEventCopyWith<$Res> implements $AgentSessionEventCopyWith<$Res> {
  factory $AgentSessionFollowUpSentEventCopyWith(AgentSessionFollowUpSentEvent value, $Res Function(AgentSessionFollowUpSentEvent) _then) = _$AgentSessionFollowUpSentEventCopyWithImpl;
@useResult
$Res call({
 String text
});




}
/// @nodoc
class _$AgentSessionFollowUpSentEventCopyWithImpl<$Res>
    implements $AgentSessionFollowUpSentEventCopyWith<$Res> {
  _$AgentSessionFollowUpSentEventCopyWithImpl(this._self, this._then);

  final AgentSessionFollowUpSentEvent _self;
  final $Res Function(AgentSessionFollowUpSentEvent) _then;

/// Create a copy of AgentSessionEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? text = null,}) {
  return _then(AgentSessionFollowUpSentEvent(
null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AgentSessionStoppedEvent implements AgentSessionEvent {
  const AgentSessionStoppedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentSessionStoppedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AgentSessionEvent.stopped()';
}


}




/// @nodoc


class AgentSessionEndedEvent implements AgentSessionEvent {
  const AgentSessionEndedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentSessionEndedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AgentSessionEvent.ended()';
}


}




/// @nodoc


class AgentSessionDismissedEvent implements AgentSessionEvent {
  const AgentSessionDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentSessionDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AgentSessionEvent.dismissed()';
}


}




/// @nodoc


class AgentSessionUndoneEvent implements AgentSessionEvent {
  const AgentSessionUndoneEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentSessionUndoneEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AgentSessionEvent.undone()';
}


}




/// @nodoc


class AgentSessionTroubleClearedEvent implements AgentSessionEvent {
  const AgentSessionTroubleClearedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentSessionTroubleClearedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AgentSessionEvent.troubleCleared()';
}


}




/// @nodoc
mixin _$AgentSessionState {

 LoadStatus get status; AgentTranscript? get transcript;/// A message is on its way.
 bool get sending;/// The session is no longer listed.
 bool get gone;/// Why the last send or undo didn't happen, as the core words it.
 String? get trouble;
/// Create a copy of AgentSessionState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentSessionStateCopyWith<AgentSessionState> get copyWith => _$AgentSessionStateCopyWithImpl<AgentSessionState>(this as AgentSessionState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentSessionState&&(identical(other.status, status) || other.status == status)&&(identical(other.transcript, transcript) || other.transcript == transcript)&&(identical(other.sending, sending) || other.sending == sending)&&(identical(other.gone, gone) || other.gone == gone)&&(identical(other.trouble, trouble) || other.trouble == trouble));
}


@override
int get hashCode => Object.hash(runtimeType,status,transcript,sending,gone,trouble);

@override
String toString() {
  return 'AgentSessionState(status: $status, transcript: $transcript, sending: $sending, gone: $gone, trouble: $trouble)';
}


}

/// @nodoc
abstract mixin class $AgentSessionStateCopyWith<$Res>  {
  factory $AgentSessionStateCopyWith(AgentSessionState value, $Res Function(AgentSessionState) _then) = _$AgentSessionStateCopyWithImpl;
@useResult
$Res call({
 LoadStatus status, AgentTranscript? transcript, bool sending, bool gone, String? trouble
});


$AgentTranscriptCopyWith<$Res>? get transcript;

}
/// @nodoc
class _$AgentSessionStateCopyWithImpl<$Res>
    implements $AgentSessionStateCopyWith<$Res> {
  _$AgentSessionStateCopyWithImpl(this._self, this._then);

  final AgentSessionState _self;
  final $Res Function(AgentSessionState) _then;

/// Create a copy of AgentSessionState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? transcript = freezed,Object? sending = null,Object? gone = null,Object? trouble = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,transcript: freezed == transcript ? _self.transcript : transcript // ignore: cast_nullable_to_non_nullable
as AgentTranscript?,sending: null == sending ? _self.sending : sending // ignore: cast_nullable_to_non_nullable
as bool,gone: null == gone ? _self.gone : gone // ignore: cast_nullable_to_non_nullable
as bool,trouble: freezed == trouble ? _self.trouble : trouble // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}
/// Create a copy of AgentSessionState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AgentTranscriptCopyWith<$Res>? get transcript {
    if (_self.transcript == null) {
    return null;
  }

  return $AgentTranscriptCopyWith<$Res>(_self.transcript!, (value) {
    return _then(_self.copyWith(transcript: value));
  });
}
}


/// Adds pattern-matching-related methods to [AgentSessionState].
extension AgentSessionStatePatterns on AgentSessionState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AgentSessionState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AgentSessionState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AgentSessionState value)  $default,){
final _that = this;
switch (_that) {
case _AgentSessionState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AgentSessionState value)?  $default,){
final _that = this;
switch (_that) {
case _AgentSessionState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LoadStatus status,  AgentTranscript? transcript,  bool sending,  bool gone,  String? trouble)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AgentSessionState() when $default != null:
return $default(_that.status,_that.transcript,_that.sending,_that.gone,_that.trouble);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LoadStatus status,  AgentTranscript? transcript,  bool sending,  bool gone,  String? trouble)  $default,) {final _that = this;
switch (_that) {
case _AgentSessionState():
return $default(_that.status,_that.transcript,_that.sending,_that.gone,_that.trouble);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LoadStatus status,  AgentTranscript? transcript,  bool sending,  bool gone,  String? trouble)?  $default,) {final _that = this;
switch (_that) {
case _AgentSessionState() when $default != null:
return $default(_that.status,_that.transcript,_that.sending,_that.gone,_that.trouble);case _:
  return null;

}
}

}

/// @nodoc


class _AgentSessionState implements AgentSessionState {
  const _AgentSessionState({this.status = LoadStatus.loading, this.transcript, this.sending = false, this.gone = false, this.trouble});
  

@override@JsonKey() final  LoadStatus status;
@override final  AgentTranscript? transcript;
/// A message is on its way.
@override@JsonKey() final  bool sending;
/// The session is no longer listed.
@override@JsonKey() final  bool gone;
/// Why the last send or undo didn't happen, as the core words it.
@override final  String? trouble;

/// Create a copy of AgentSessionState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AgentSessionStateCopyWith<_AgentSessionState> get copyWith => __$AgentSessionStateCopyWithImpl<_AgentSessionState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AgentSessionState&&(identical(other.status, status) || other.status == status)&&(identical(other.transcript, transcript) || other.transcript == transcript)&&(identical(other.sending, sending) || other.sending == sending)&&(identical(other.gone, gone) || other.gone == gone)&&(identical(other.trouble, trouble) || other.trouble == trouble));
}


@override
int get hashCode => Object.hash(runtimeType,status,transcript,sending,gone,trouble);

@override
String toString() {
  return 'AgentSessionState(status: $status, transcript: $transcript, sending: $sending, gone: $gone, trouble: $trouble)';
}


}

/// @nodoc
abstract mixin class _$AgentSessionStateCopyWith<$Res> implements $AgentSessionStateCopyWith<$Res> {
  factory _$AgentSessionStateCopyWith(_AgentSessionState value, $Res Function(_AgentSessionState) _then) = __$AgentSessionStateCopyWithImpl;
@override @useResult
$Res call({
 LoadStatus status, AgentTranscript? transcript, bool sending, bool gone, String? trouble
});


@override $AgentTranscriptCopyWith<$Res>? get transcript;

}
/// @nodoc
class __$AgentSessionStateCopyWithImpl<$Res>
    implements _$AgentSessionStateCopyWith<$Res> {
  __$AgentSessionStateCopyWithImpl(this._self, this._then);

  final _AgentSessionState _self;
  final $Res Function(_AgentSessionState) _then;

/// Create a copy of AgentSessionState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? transcript = freezed,Object? sending = null,Object? gone = null,Object? trouble = freezed,}) {
  return _then(_AgentSessionState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,transcript: freezed == transcript ? _self.transcript : transcript // ignore: cast_nullable_to_non_nullable
as AgentTranscript?,sending: null == sending ? _self.sending : sending // ignore: cast_nullable_to_non_nullable
as bool,gone: null == gone ? _self.gone : gone // ignore: cast_nullable_to_non_nullable
as bool,trouble: freezed == trouble ? _self.trouble : trouble // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

/// Create a copy of AgentSessionState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AgentTranscriptCopyWith<$Res>? get transcript {
    if (_self.transcript == null) {
    return null;
  }

  return $AgentTranscriptCopyWith<$Res>(_self.transcript!, (value) {
    return _then(_self.copyWith(transcript: value));
  });
}
}

// dart format on
