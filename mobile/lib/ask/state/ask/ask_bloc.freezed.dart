// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'ask_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AskEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AskEvent()';
}


}

/// @nodoc
class $AskEventCopyWith<$Res>  {
$AskEventCopyWith(AskEvent _, $Res Function(AskEvent) __);
}


/// Adds pattern-matching-related methods to [AskEvent].
extension AskEventPatterns on AskEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AskStartedEvent value)?  started,TResult Function( AskSettingsChangedEvent value)?  settingsChanged,TResult Function( AskQuestionAskedEvent value)?  questionAsked,TResult Function( AskUpdateReceivedEvent value)?  updateReceived,TResult Function( AskStreamEndedEvent value)?  streamEnded,TResult Function( AskStoppedEvent value)?  stopped,TResult Function( AskClearedEvent value)?  cleared,TResult Function( AskRetriedEvent value)?  retried,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AskStartedEvent() when started != null:
return started(_that);case AskSettingsChangedEvent() when settingsChanged != null:
return settingsChanged(_that);case AskQuestionAskedEvent() when questionAsked != null:
return questionAsked(_that);case AskUpdateReceivedEvent() when updateReceived != null:
return updateReceived(_that);case AskStreamEndedEvent() when streamEnded != null:
return streamEnded(_that);case AskStoppedEvent() when stopped != null:
return stopped(_that);case AskClearedEvent() when cleared != null:
return cleared(_that);case AskRetriedEvent() when retried != null:
return retried(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AskStartedEvent value)  started,required TResult Function( AskSettingsChangedEvent value)  settingsChanged,required TResult Function( AskQuestionAskedEvent value)  questionAsked,required TResult Function( AskUpdateReceivedEvent value)  updateReceived,required TResult Function( AskStreamEndedEvent value)  streamEnded,required TResult Function( AskStoppedEvent value)  stopped,required TResult Function( AskClearedEvent value)  cleared,required TResult Function( AskRetriedEvent value)  retried,}){
final _that = this;
switch (_that) {
case AskStartedEvent():
return started(_that);case AskSettingsChangedEvent():
return settingsChanged(_that);case AskQuestionAskedEvent():
return questionAsked(_that);case AskUpdateReceivedEvent():
return updateReceived(_that);case AskStreamEndedEvent():
return streamEnded(_that);case AskStoppedEvent():
return stopped(_that);case AskClearedEvent():
return cleared(_that);case AskRetriedEvent():
return retried(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AskStartedEvent value)?  started,TResult? Function( AskSettingsChangedEvent value)?  settingsChanged,TResult? Function( AskQuestionAskedEvent value)?  questionAsked,TResult? Function( AskUpdateReceivedEvent value)?  updateReceived,TResult? Function( AskStreamEndedEvent value)?  streamEnded,TResult? Function( AskStoppedEvent value)?  stopped,TResult? Function( AskClearedEvent value)?  cleared,TResult? Function( AskRetriedEvent value)?  retried,}){
final _that = this;
switch (_that) {
case AskStartedEvent() when started != null:
return started(_that);case AskSettingsChangedEvent() when settingsChanged != null:
return settingsChanged(_that);case AskQuestionAskedEvent() when questionAsked != null:
return questionAsked(_that);case AskUpdateReceivedEvent() when updateReceived != null:
return updateReceived(_that);case AskStreamEndedEvent() when streamEnded != null:
return streamEnded(_that);case AskStoppedEvent() when stopped != null:
return stopped(_that);case AskClearedEvent() when cleared != null:
return cleared(_that);case AskRetriedEvent() when retried != null:
return retried(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function( AiSettings settings)?  settingsChanged,TResult Function( String question)?  questionAsked,TResult Function( AskUpdate update)?  updateReceived,TResult Function()?  streamEnded,TResult Function()?  stopped,TResult Function()?  cleared,TResult Function()?  retried,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AskStartedEvent() when started != null:
return started();case AskSettingsChangedEvent() when settingsChanged != null:
return settingsChanged(_that.settings);case AskQuestionAskedEvent() when questionAsked != null:
return questionAsked(_that.question);case AskUpdateReceivedEvent() when updateReceived != null:
return updateReceived(_that.update);case AskStreamEndedEvent() when streamEnded != null:
return streamEnded();case AskStoppedEvent() when stopped != null:
return stopped();case AskClearedEvent() when cleared != null:
return cleared();case AskRetriedEvent() when retried != null:
return retried();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function( AiSettings settings)  settingsChanged,required TResult Function( String question)  questionAsked,required TResult Function( AskUpdate update)  updateReceived,required TResult Function()  streamEnded,required TResult Function()  stopped,required TResult Function()  cleared,required TResult Function()  retried,}) {final _that = this;
switch (_that) {
case AskStartedEvent():
return started();case AskSettingsChangedEvent():
return settingsChanged(_that.settings);case AskQuestionAskedEvent():
return questionAsked(_that.question);case AskUpdateReceivedEvent():
return updateReceived(_that.update);case AskStreamEndedEvent():
return streamEnded();case AskStoppedEvent():
return stopped();case AskClearedEvent():
return cleared();case AskRetriedEvent():
return retried();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function( AiSettings settings)?  settingsChanged,TResult? Function( String question)?  questionAsked,TResult? Function( AskUpdate update)?  updateReceived,TResult? Function()?  streamEnded,TResult? Function()?  stopped,TResult? Function()?  cleared,TResult? Function()?  retried,}) {final _that = this;
switch (_that) {
case AskStartedEvent() when started != null:
return started();case AskSettingsChangedEvent() when settingsChanged != null:
return settingsChanged(_that.settings);case AskQuestionAskedEvent() when questionAsked != null:
return questionAsked(_that.question);case AskUpdateReceivedEvent() when updateReceived != null:
return updateReceived(_that.update);case AskStreamEndedEvent() when streamEnded != null:
return streamEnded();case AskStoppedEvent() when stopped != null:
return stopped();case AskClearedEvent() when cleared != null:
return cleared();case AskRetriedEvent() when retried != null:
return retried();case _:
  return null;

}
}

}

/// @nodoc


class AskStartedEvent implements AskEvent {
  const AskStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AskEvent.started()';
}


}




/// @nodoc


class AskSettingsChangedEvent implements AskEvent {
  const AskSettingsChangedEvent(this.settings);
  

 final  AiSettings settings;

/// Create a copy of AskEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskSettingsChangedEventCopyWith<AskSettingsChangedEvent> get copyWith => _$AskSettingsChangedEventCopyWithImpl<AskSettingsChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskSettingsChangedEvent&&(identical(other.settings, settings) || other.settings == settings));
}


@override
int get hashCode => Object.hash(runtimeType,settings);

@override
String toString() {
  return 'AskEvent.settingsChanged(settings: $settings)';
}


}

/// @nodoc
abstract mixin class $AskSettingsChangedEventCopyWith<$Res> implements $AskEventCopyWith<$Res> {
  factory $AskSettingsChangedEventCopyWith(AskSettingsChangedEvent value, $Res Function(AskSettingsChangedEvent) _then) = _$AskSettingsChangedEventCopyWithImpl;
@useResult
$Res call({
 AiSettings settings
});


$AiSettingsCopyWith<$Res> get settings;

}
/// @nodoc
class _$AskSettingsChangedEventCopyWithImpl<$Res>
    implements $AskSettingsChangedEventCopyWith<$Res> {
  _$AskSettingsChangedEventCopyWithImpl(this._self, this._then);

  final AskSettingsChangedEvent _self;
  final $Res Function(AskSettingsChangedEvent) _then;

/// Create a copy of AskEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? settings = null,}) {
  return _then(AskSettingsChangedEvent(
null == settings ? _self.settings : settings // ignore: cast_nullable_to_non_nullable
as AiSettings,
  ));
}

/// Create a copy of AskEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AiSettingsCopyWith<$Res> get settings {
  
  return $AiSettingsCopyWith<$Res>(_self.settings, (value) {
    return _then(_self.copyWith(settings: value));
  });
}
}

/// @nodoc


class AskQuestionAskedEvent implements AskEvent {
  const AskQuestionAskedEvent(this.question);
  

 final  String question;

/// Create a copy of AskEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskQuestionAskedEventCopyWith<AskQuestionAskedEvent> get copyWith => _$AskQuestionAskedEventCopyWithImpl<AskQuestionAskedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskQuestionAskedEvent&&(identical(other.question, question) || other.question == question));
}


@override
int get hashCode => Object.hash(runtimeType,question);

@override
String toString() {
  return 'AskEvent.questionAsked(question: $question)';
}


}

/// @nodoc
abstract mixin class $AskQuestionAskedEventCopyWith<$Res> implements $AskEventCopyWith<$Res> {
  factory $AskQuestionAskedEventCopyWith(AskQuestionAskedEvent value, $Res Function(AskQuestionAskedEvent) _then) = _$AskQuestionAskedEventCopyWithImpl;
@useResult
$Res call({
 String question
});




}
/// @nodoc
class _$AskQuestionAskedEventCopyWithImpl<$Res>
    implements $AskQuestionAskedEventCopyWith<$Res> {
  _$AskQuestionAskedEventCopyWithImpl(this._self, this._then);

  final AskQuestionAskedEvent _self;
  final $Res Function(AskQuestionAskedEvent) _then;

/// Create a copy of AskEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? question = null,}) {
  return _then(AskQuestionAskedEvent(
null == question ? _self.question : question // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AskUpdateReceivedEvent implements AskEvent {
  const AskUpdateReceivedEvent(this.update);
  

 final  AskUpdate update;

/// Create a copy of AskEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskUpdateReceivedEventCopyWith<AskUpdateReceivedEvent> get copyWith => _$AskUpdateReceivedEventCopyWithImpl<AskUpdateReceivedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskUpdateReceivedEvent&&(identical(other.update, update) || other.update == update));
}


@override
int get hashCode => Object.hash(runtimeType,update);

@override
String toString() {
  return 'AskEvent.updateReceived(update: $update)';
}


}

/// @nodoc
abstract mixin class $AskUpdateReceivedEventCopyWith<$Res> implements $AskEventCopyWith<$Res> {
  factory $AskUpdateReceivedEventCopyWith(AskUpdateReceivedEvent value, $Res Function(AskUpdateReceivedEvent) _then) = _$AskUpdateReceivedEventCopyWithImpl;
@useResult
$Res call({
 AskUpdate update
});


$AskUpdateCopyWith<$Res> get update;

}
/// @nodoc
class _$AskUpdateReceivedEventCopyWithImpl<$Res>
    implements $AskUpdateReceivedEventCopyWith<$Res> {
  _$AskUpdateReceivedEventCopyWithImpl(this._self, this._then);

  final AskUpdateReceivedEvent _self;
  final $Res Function(AskUpdateReceivedEvent) _then;

/// Create a copy of AskEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? update = null,}) {
  return _then(AskUpdateReceivedEvent(
null == update ? _self.update : update // ignore: cast_nullable_to_non_nullable
as AskUpdate,
  ));
}

/// Create a copy of AskEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AskUpdateCopyWith<$Res> get update {
  
  return $AskUpdateCopyWith<$Res>(_self.update, (value) {
    return _then(_self.copyWith(update: value));
  });
}
}

/// @nodoc


class AskStreamEndedEvent implements AskEvent {
  const AskStreamEndedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskStreamEndedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AskEvent.streamEnded()';
}


}




/// @nodoc


class AskStoppedEvent implements AskEvent {
  const AskStoppedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskStoppedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AskEvent.stopped()';
}


}




/// @nodoc


class AskClearedEvent implements AskEvent {
  const AskClearedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskClearedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AskEvent.cleared()';
}


}




/// @nodoc


class AskRetriedEvent implements AskEvent {
  const AskRetriedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskRetriedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AskEvent.retried()';
}


}




/// @nodoc
mixin _$AskState {

/// Whether the settings have been read.
 LoadStatus get status;/// Whether AI is on; with it off there is nothing to ask.
 bool get aiEnabled;/// The line about where a question goes, shown while the conversation is
/// empty.
 String get privacy;/// The conversation, oldest question first.
 List<AskTurn> get turns; AskPhase get phase;
/// Create a copy of AskState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskStateCopyWith<AskState> get copyWith => _$AskStateCopyWithImpl<AskState>(this as AskState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskState&&(identical(other.status, status) || other.status == status)&&(identical(other.aiEnabled, aiEnabled) || other.aiEnabled == aiEnabled)&&(identical(other.privacy, privacy) || other.privacy == privacy)&&const DeepCollectionEquality().equals(other.turns, turns)&&(identical(other.phase, phase) || other.phase == phase));
}


@override
int get hashCode => Object.hash(runtimeType,status,aiEnabled,privacy,const DeepCollectionEquality().hash(turns),phase);

@override
String toString() {
  return 'AskState(status: $status, aiEnabled: $aiEnabled, privacy: $privacy, turns: $turns, phase: $phase)';
}


}

/// @nodoc
abstract mixin class $AskStateCopyWith<$Res>  {
  factory $AskStateCopyWith(AskState value, $Res Function(AskState) _then) = _$AskStateCopyWithImpl;
@useResult
$Res call({
 LoadStatus status, bool aiEnabled, String privacy, List<AskTurn> turns, AskPhase phase
});




}
/// @nodoc
class _$AskStateCopyWithImpl<$Res>
    implements $AskStateCopyWith<$Res> {
  _$AskStateCopyWithImpl(this._self, this._then);

  final AskState _self;
  final $Res Function(AskState) _then;

/// Create a copy of AskState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? aiEnabled = null,Object? privacy = null,Object? turns = null,Object? phase = null,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,aiEnabled: null == aiEnabled ? _self.aiEnabled : aiEnabled // ignore: cast_nullable_to_non_nullable
as bool,privacy: null == privacy ? _self.privacy : privacy // ignore: cast_nullable_to_non_nullable
as String,turns: null == turns ? _self.turns : turns // ignore: cast_nullable_to_non_nullable
as List<AskTurn>,phase: null == phase ? _self.phase : phase // ignore: cast_nullable_to_non_nullable
as AskPhase,
  ));
}

}


/// Adds pattern-matching-related methods to [AskState].
extension AskStatePatterns on AskState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AskState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AskState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AskState value)  $default,){
final _that = this;
switch (_that) {
case _AskState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AskState value)?  $default,){
final _that = this;
switch (_that) {
case _AskState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LoadStatus status,  bool aiEnabled,  String privacy,  List<AskTurn> turns,  AskPhase phase)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AskState() when $default != null:
return $default(_that.status,_that.aiEnabled,_that.privacy,_that.turns,_that.phase);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LoadStatus status,  bool aiEnabled,  String privacy,  List<AskTurn> turns,  AskPhase phase)  $default,) {final _that = this;
switch (_that) {
case _AskState():
return $default(_that.status,_that.aiEnabled,_that.privacy,_that.turns,_that.phase);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LoadStatus status,  bool aiEnabled,  String privacy,  List<AskTurn> turns,  AskPhase phase)?  $default,) {final _that = this;
switch (_that) {
case _AskState() when $default != null:
return $default(_that.status,_that.aiEnabled,_that.privacy,_that.turns,_that.phase);case _:
  return null;

}
}

}

/// @nodoc


class _AskState extends AskState {
  const _AskState({this.status = LoadStatus.loading, this.aiEnabled = false, this.privacy = '', final  List<AskTurn> turns = const <AskTurn>[], this.phase = AskPhase.idle}): _turns = turns,super._();
  

/// Whether the settings have been read.
@override@JsonKey() final  LoadStatus status;
/// Whether AI is on; with it off there is nothing to ask.
@override@JsonKey() final  bool aiEnabled;
/// The line about where a question goes, shown while the conversation is
/// empty.
@override@JsonKey() final  String privacy;
/// The conversation, oldest question first.
 final  List<AskTurn> _turns;
/// The conversation, oldest question first.
@override@JsonKey() List<AskTurn> get turns {
  if (_turns is EqualUnmodifiableListView) return _turns;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_turns);
}

@override@JsonKey() final  AskPhase phase;

/// Create a copy of AskState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AskStateCopyWith<_AskState> get copyWith => __$AskStateCopyWithImpl<_AskState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AskState&&(identical(other.status, status) || other.status == status)&&(identical(other.aiEnabled, aiEnabled) || other.aiEnabled == aiEnabled)&&(identical(other.privacy, privacy) || other.privacy == privacy)&&const DeepCollectionEquality().equals(other._turns, _turns)&&(identical(other.phase, phase) || other.phase == phase));
}


@override
int get hashCode => Object.hash(runtimeType,status,aiEnabled,privacy,const DeepCollectionEquality().hash(_turns),phase);

@override
String toString() {
  return 'AskState(status: $status, aiEnabled: $aiEnabled, privacy: $privacy, turns: $turns, phase: $phase)';
}


}

/// @nodoc
abstract mixin class _$AskStateCopyWith<$Res> implements $AskStateCopyWith<$Res> {
  factory _$AskStateCopyWith(_AskState value, $Res Function(_AskState) _then) = __$AskStateCopyWithImpl;
@override @useResult
$Res call({
 LoadStatus status, bool aiEnabled, String privacy, List<AskTurn> turns, AskPhase phase
});




}
/// @nodoc
class __$AskStateCopyWithImpl<$Res>
    implements _$AskStateCopyWith<$Res> {
  __$AskStateCopyWithImpl(this._self, this._then);

  final _AskState _self;
  final $Res Function(_AskState) _then;

/// Create a copy of AskState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? aiEnabled = null,Object? privacy = null,Object? turns = null,Object? phase = null,}) {
  return _then(_AskState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,aiEnabled: null == aiEnabled ? _self.aiEnabled : aiEnabled // ignore: cast_nullable_to_non_nullable
as bool,privacy: null == privacy ? _self.privacy : privacy // ignore: cast_nullable_to_non_nullable
as String,turns: null == turns ? _self._turns : turns // ignore: cast_nullable_to_non_nullable
as List<AskTurn>,phase: null == phase ? _self.phase : phase // ignore: cast_nullable_to_non_nullable
as AskPhase,
  ));
}


}

// dart format on
