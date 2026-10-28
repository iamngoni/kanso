// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'dictation_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$DictationEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DictationEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DictationEvent()';
}


}

/// @nodoc
class $DictationEventCopyWith<$Res>  {
$DictationEventCopyWith(DictationEvent _, $Res Function(DictationEvent) __);
}


/// Adds pattern-matching-related methods to [DictationEvent].
extension DictationEventPatterns on DictationEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( DictationCheckedEvent value)?  checked,TResult Function( DictationStartedEvent value)?  started,TResult Function( DictationStoppedEvent value)?  stopped,TResult Function( DictationFinishRequestedEvent value)?  finishRequested,TResult Function( DictationCancelledEvent value)?  cancelled,TResult Function( DictationSettingsRequestedEvent value)?  settingsRequested,TResult Function( DictationSignalReceivedEvent value)?  signalReceived,required TResult orElse(),}){
final _that = this;
switch (_that) {
case DictationCheckedEvent() when checked != null:
return checked(_that);case DictationStartedEvent() when started != null:
return started(_that);case DictationStoppedEvent() when stopped != null:
return stopped(_that);case DictationFinishRequestedEvent() when finishRequested != null:
return finishRequested(_that);case DictationCancelledEvent() when cancelled != null:
return cancelled(_that);case DictationSettingsRequestedEvent() when settingsRequested != null:
return settingsRequested(_that);case DictationSignalReceivedEvent() when signalReceived != null:
return signalReceived(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( DictationCheckedEvent value)  checked,required TResult Function( DictationStartedEvent value)  started,required TResult Function( DictationStoppedEvent value)  stopped,required TResult Function( DictationFinishRequestedEvent value)  finishRequested,required TResult Function( DictationCancelledEvent value)  cancelled,required TResult Function( DictationSettingsRequestedEvent value)  settingsRequested,required TResult Function( DictationSignalReceivedEvent value)  signalReceived,}){
final _that = this;
switch (_that) {
case DictationCheckedEvent():
return checked(_that);case DictationStartedEvent():
return started(_that);case DictationStoppedEvent():
return stopped(_that);case DictationFinishRequestedEvent():
return finishRequested(_that);case DictationCancelledEvent():
return cancelled(_that);case DictationSettingsRequestedEvent():
return settingsRequested(_that);case DictationSignalReceivedEvent():
return signalReceived(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( DictationCheckedEvent value)?  checked,TResult? Function( DictationStartedEvent value)?  started,TResult? Function( DictationStoppedEvent value)?  stopped,TResult? Function( DictationFinishRequestedEvent value)?  finishRequested,TResult? Function( DictationCancelledEvent value)?  cancelled,TResult? Function( DictationSettingsRequestedEvent value)?  settingsRequested,TResult? Function( DictationSignalReceivedEvent value)?  signalReceived,}){
final _that = this;
switch (_that) {
case DictationCheckedEvent() when checked != null:
return checked(_that);case DictationStartedEvent() when started != null:
return started(_that);case DictationStoppedEvent() when stopped != null:
return stopped(_that);case DictationFinishRequestedEvent() when finishRequested != null:
return finishRequested(_that);case DictationCancelledEvent() when cancelled != null:
return cancelled(_that);case DictationSettingsRequestedEvent() when settingsRequested != null:
return settingsRequested(_that);case DictationSignalReceivedEvent() when signalReceived != null:
return signalReceived(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  checked,TResult Function( String? localeTag,  bool showOverlay)?  started,TResult Function()?  stopped,TResult Function()?  finishRequested,TResult Function()?  cancelled,TResult Function()?  settingsRequested,TResult Function( DictationSignal signal)?  signalReceived,required TResult orElse(),}) {final _that = this;
switch (_that) {
case DictationCheckedEvent() when checked != null:
return checked();case DictationStartedEvent() when started != null:
return started(_that.localeTag,_that.showOverlay);case DictationStoppedEvent() when stopped != null:
return stopped();case DictationFinishRequestedEvent() when finishRequested != null:
return finishRequested();case DictationCancelledEvent() when cancelled != null:
return cancelled();case DictationSettingsRequestedEvent() when settingsRequested != null:
return settingsRequested();case DictationSignalReceivedEvent() when signalReceived != null:
return signalReceived(_that.signal);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  checked,required TResult Function( String? localeTag,  bool showOverlay)  started,required TResult Function()  stopped,required TResult Function()  finishRequested,required TResult Function()  cancelled,required TResult Function()  settingsRequested,required TResult Function( DictationSignal signal)  signalReceived,}) {final _that = this;
switch (_that) {
case DictationCheckedEvent():
return checked();case DictationStartedEvent():
return started(_that.localeTag,_that.showOverlay);case DictationStoppedEvent():
return stopped();case DictationFinishRequestedEvent():
return finishRequested();case DictationCancelledEvent():
return cancelled();case DictationSettingsRequestedEvent():
return settingsRequested();case DictationSignalReceivedEvent():
return signalReceived(_that.signal);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  checked,TResult? Function( String? localeTag,  bool showOverlay)?  started,TResult? Function()?  stopped,TResult? Function()?  finishRequested,TResult? Function()?  cancelled,TResult? Function()?  settingsRequested,TResult? Function( DictationSignal signal)?  signalReceived,}) {final _that = this;
switch (_that) {
case DictationCheckedEvent() when checked != null:
return checked();case DictationStartedEvent() when started != null:
return started(_that.localeTag,_that.showOverlay);case DictationStoppedEvent() when stopped != null:
return stopped();case DictationFinishRequestedEvent() when finishRequested != null:
return finishRequested();case DictationCancelledEvent() when cancelled != null:
return cancelled();case DictationSettingsRequestedEvent() when settingsRequested != null:
return settingsRequested();case DictationSignalReceivedEvent() when signalReceived != null:
return signalReceived(_that.signal);case _:
  return null;

}
}

}

/// @nodoc


class DictationCheckedEvent implements DictationEvent {
  const DictationCheckedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DictationCheckedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DictationEvent.checked()';
}


}




/// @nodoc


class DictationStartedEvent implements DictationEvent {
  const DictationStartedEvent({this.localeTag, this.showOverlay = false});
  

 final  String? localeTag;
@JsonKey() final  bool showOverlay;

/// Create a copy of DictationEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DictationStartedEventCopyWith<DictationStartedEvent> get copyWith => _$DictationStartedEventCopyWithImpl<DictationStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DictationStartedEvent&&(identical(other.localeTag, localeTag) || other.localeTag == localeTag)&&(identical(other.showOverlay, showOverlay) || other.showOverlay == showOverlay));
}


@override
int get hashCode => Object.hash(runtimeType,localeTag,showOverlay);

@override
String toString() {
  return 'DictationEvent.started(localeTag: $localeTag, showOverlay: $showOverlay)';
}


}

/// @nodoc
abstract mixin class $DictationStartedEventCopyWith<$Res> implements $DictationEventCopyWith<$Res> {
  factory $DictationStartedEventCopyWith(DictationStartedEvent value, $Res Function(DictationStartedEvent) _then) = _$DictationStartedEventCopyWithImpl;
@useResult
$Res call({
 String? localeTag, bool showOverlay
});




}
/// @nodoc
class _$DictationStartedEventCopyWithImpl<$Res>
    implements $DictationStartedEventCopyWith<$Res> {
  _$DictationStartedEventCopyWithImpl(this._self, this._then);

  final DictationStartedEvent _self;
  final $Res Function(DictationStartedEvent) _then;

/// Create a copy of DictationEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? localeTag = freezed,Object? showOverlay = null,}) {
  return _then(DictationStartedEvent(
localeTag: freezed == localeTag ? _self.localeTag : localeTag // ignore: cast_nullable_to_non_nullable
as String?,showOverlay: null == showOverlay ? _self.showOverlay : showOverlay // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class DictationStoppedEvent implements DictationEvent {
  const DictationStoppedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DictationStoppedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DictationEvent.stopped()';
}


}




/// @nodoc


class DictationFinishRequestedEvent implements DictationEvent {
  const DictationFinishRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DictationFinishRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DictationEvent.finishRequested()';
}


}




/// @nodoc


class DictationCancelledEvent implements DictationEvent {
  const DictationCancelledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DictationCancelledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DictationEvent.cancelled()';
}


}




/// @nodoc


class DictationSettingsRequestedEvent implements DictationEvent {
  const DictationSettingsRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DictationSettingsRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DictationEvent.settingsRequested()';
}


}




/// @nodoc


class DictationSignalReceivedEvent implements DictationEvent {
  const DictationSignalReceivedEvent(this.signal);
  

 final  DictationSignal signal;

/// Create a copy of DictationEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DictationSignalReceivedEventCopyWith<DictationSignalReceivedEvent> get copyWith => _$DictationSignalReceivedEventCopyWithImpl<DictationSignalReceivedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DictationSignalReceivedEvent&&(identical(other.signal, signal) || other.signal == signal));
}


@override
int get hashCode => Object.hash(runtimeType,signal);

@override
String toString() {
  return 'DictationEvent.signalReceived(signal: $signal)';
}


}

/// @nodoc
abstract mixin class $DictationSignalReceivedEventCopyWith<$Res> implements $DictationEventCopyWith<$Res> {
  factory $DictationSignalReceivedEventCopyWith(DictationSignalReceivedEvent value, $Res Function(DictationSignalReceivedEvent) _then) = _$DictationSignalReceivedEventCopyWithImpl;
@useResult
$Res call({
 DictationSignal signal
});


$DictationSignalCopyWith<$Res> get signal;

}
/// @nodoc
class _$DictationSignalReceivedEventCopyWithImpl<$Res>
    implements $DictationSignalReceivedEventCopyWith<$Res> {
  _$DictationSignalReceivedEventCopyWithImpl(this._self, this._then);

  final DictationSignalReceivedEvent _self;
  final $Res Function(DictationSignalReceivedEvent) _then;

/// Create a copy of DictationEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? signal = null,}) {
  return _then(DictationSignalReceivedEvent(
null == signal ? _self.signal : signal // ignore: cast_nullable_to_non_nullable
as DictationSignal,
  ));
}

/// Create a copy of DictationEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$DictationSignalCopyWith<$Res> get signal {
  
  return $DictationSignalCopyWith<$Res>(_self.signal, (value) {
    return _then(_self.copyWith(signal: value));
  });
}
}

/// @nodoc
mixin _$DictationState {

 DictationStatus get status;/// The phrases the recognizer finished, in the order they were spoken.
 List<String> get segments;/// The words of the phrase being spoken, still changing.
 String get partial;/// The finished phrases as the rules write them (spoken punctuation and
/// commands acted on): what a note would hold, without the words still
/// being spoken.
 String get finishedText;/// Everything heard as the rules write it, the words still being spoken
/// included: what shows is what lands.
 String get text;/// How loud the voice is, 0 to 1.
 double get level;/// Why dictation is unavailable, while [status] is unavailable.
 DictationUnavailableReason? get reason;/// Why the session failed, while [status] is error.
 DictationFailure? get failure;/// Whether the phone's rules, not the person, keep dictation from
/// listening, while [status] is denied.
 bool get restricted;/// Whether the person asked to be done and the session has ended since.
 bool get finished;/// Whether the app-level pill should present this dictation session.
/// Voice-note pages render their own transcript and controls.
 bool get showOverlay;
/// Create a copy of DictationState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DictationStateCopyWith<DictationState> get copyWith => _$DictationStateCopyWithImpl<DictationState>(this as DictationState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DictationState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other.segments, segments)&&(identical(other.partial, partial) || other.partial == partial)&&(identical(other.finishedText, finishedText) || other.finishedText == finishedText)&&(identical(other.text, text) || other.text == text)&&(identical(other.level, level) || other.level == level)&&(identical(other.reason, reason) || other.reason == reason)&&(identical(other.failure, failure) || other.failure == failure)&&(identical(other.restricted, restricted) || other.restricted == restricted)&&(identical(other.finished, finished) || other.finished == finished)&&(identical(other.showOverlay, showOverlay) || other.showOverlay == showOverlay));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(segments),partial,finishedText,text,level,reason,failure,restricted,finished,showOverlay);

@override
String toString() {
  return 'DictationState(status: $status, segments: $segments, partial: $partial, finishedText: $finishedText, text: $text, level: $level, reason: $reason, failure: $failure, restricted: $restricted, finished: $finished, showOverlay: $showOverlay)';
}


}

/// @nodoc
abstract mixin class $DictationStateCopyWith<$Res>  {
  factory $DictationStateCopyWith(DictationState value, $Res Function(DictationState) _then) = _$DictationStateCopyWithImpl;
@useResult
$Res call({
 DictationStatus status, List<String> segments, String partial, String finishedText, String text, double level, DictationUnavailableReason? reason, DictationFailure? failure, bool restricted, bool finished, bool showOverlay
});




}
/// @nodoc
class _$DictationStateCopyWithImpl<$Res>
    implements $DictationStateCopyWith<$Res> {
  _$DictationStateCopyWithImpl(this._self, this._then);

  final DictationState _self;
  final $Res Function(DictationState) _then;

/// Create a copy of DictationState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? segments = null,Object? partial = null,Object? finishedText = null,Object? text = null,Object? level = null,Object? reason = freezed,Object? failure = freezed,Object? restricted = null,Object? finished = null,Object? showOverlay = null,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as DictationStatus,segments: null == segments ? _self.segments : segments // ignore: cast_nullable_to_non_nullable
as List<String>,partial: null == partial ? _self.partial : partial // ignore: cast_nullable_to_non_nullable
as String,finishedText: null == finishedText ? _self.finishedText : finishedText // ignore: cast_nullable_to_non_nullable
as String,text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,level: null == level ? _self.level : level // ignore: cast_nullable_to_non_nullable
as double,reason: freezed == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as DictationUnavailableReason?,failure: freezed == failure ? _self.failure : failure // ignore: cast_nullable_to_non_nullable
as DictationFailure?,restricted: null == restricted ? _self.restricted : restricted // ignore: cast_nullable_to_non_nullable
as bool,finished: null == finished ? _self.finished : finished // ignore: cast_nullable_to_non_nullable
as bool,showOverlay: null == showOverlay ? _self.showOverlay : showOverlay // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [DictationState].
extension DictationStatePatterns on DictationState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _DictationState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _DictationState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _DictationState value)  $default,){
final _that = this;
switch (_that) {
case _DictationState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _DictationState value)?  $default,){
final _that = this;
switch (_that) {
case _DictationState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( DictationStatus status,  List<String> segments,  String partial,  String finishedText,  String text,  double level,  DictationUnavailableReason? reason,  DictationFailure? failure,  bool restricted,  bool finished,  bool showOverlay)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _DictationState() when $default != null:
return $default(_that.status,_that.segments,_that.partial,_that.finishedText,_that.text,_that.level,_that.reason,_that.failure,_that.restricted,_that.finished,_that.showOverlay);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( DictationStatus status,  List<String> segments,  String partial,  String finishedText,  String text,  double level,  DictationUnavailableReason? reason,  DictationFailure? failure,  bool restricted,  bool finished,  bool showOverlay)  $default,) {final _that = this;
switch (_that) {
case _DictationState():
return $default(_that.status,_that.segments,_that.partial,_that.finishedText,_that.text,_that.level,_that.reason,_that.failure,_that.restricted,_that.finished,_that.showOverlay);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( DictationStatus status,  List<String> segments,  String partial,  String finishedText,  String text,  double level,  DictationUnavailableReason? reason,  DictationFailure? failure,  bool restricted,  bool finished,  bool showOverlay)?  $default,) {final _that = this;
switch (_that) {
case _DictationState() when $default != null:
return $default(_that.status,_that.segments,_that.partial,_that.finishedText,_that.text,_that.level,_that.reason,_that.failure,_that.restricted,_that.finished,_that.showOverlay);case _:
  return null;

}
}

}

/// @nodoc


class _DictationState extends DictationState {
  const _DictationState({this.status = DictationStatus.idle, final  List<String> segments = const <String>[], this.partial = '', this.finishedText = '', this.text = '', this.level = 0.0, this.reason, this.failure, this.restricted = false, this.finished = false, this.showOverlay = false}): _segments = segments,super._();
  

@override@JsonKey() final  DictationStatus status;
/// The phrases the recognizer finished, in the order they were spoken.
 final  List<String> _segments;
/// The phrases the recognizer finished, in the order they were spoken.
@override@JsonKey() List<String> get segments {
  if (_segments is EqualUnmodifiableListView) return _segments;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_segments);
}

/// The words of the phrase being spoken, still changing.
@override@JsonKey() final  String partial;
/// The finished phrases as the rules write them (spoken punctuation and
/// commands acted on): what a note would hold, without the words still
/// being spoken.
@override@JsonKey() final  String finishedText;
/// Everything heard as the rules write it, the words still being spoken
/// included: what shows is what lands.
@override@JsonKey() final  String text;
/// How loud the voice is, 0 to 1.
@override@JsonKey() final  double level;
/// Why dictation is unavailable, while [status] is unavailable.
@override final  DictationUnavailableReason? reason;
/// Why the session failed, while [status] is error.
@override final  DictationFailure? failure;
/// Whether the phone's rules, not the person, keep dictation from
/// listening, while [status] is denied.
@override@JsonKey() final  bool restricted;
/// Whether the person asked to be done and the session has ended since.
@override@JsonKey() final  bool finished;
/// Whether the app-level pill should present this dictation session.
/// Voice-note pages render their own transcript and controls.
@override@JsonKey() final  bool showOverlay;

/// Create a copy of DictationState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DictationStateCopyWith<_DictationState> get copyWith => __$DictationStateCopyWithImpl<_DictationState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DictationState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other._segments, _segments)&&(identical(other.partial, partial) || other.partial == partial)&&(identical(other.finishedText, finishedText) || other.finishedText == finishedText)&&(identical(other.text, text) || other.text == text)&&(identical(other.level, level) || other.level == level)&&(identical(other.reason, reason) || other.reason == reason)&&(identical(other.failure, failure) || other.failure == failure)&&(identical(other.restricted, restricted) || other.restricted == restricted)&&(identical(other.finished, finished) || other.finished == finished)&&(identical(other.showOverlay, showOverlay) || other.showOverlay == showOverlay));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(_segments),partial,finishedText,text,level,reason,failure,restricted,finished,showOverlay);

@override
String toString() {
  return 'DictationState(status: $status, segments: $segments, partial: $partial, finishedText: $finishedText, text: $text, level: $level, reason: $reason, failure: $failure, restricted: $restricted, finished: $finished, showOverlay: $showOverlay)';
}


}

/// @nodoc
abstract mixin class _$DictationStateCopyWith<$Res> implements $DictationStateCopyWith<$Res> {
  factory _$DictationStateCopyWith(_DictationState value, $Res Function(_DictationState) _then) = __$DictationStateCopyWithImpl;
@override @useResult
$Res call({
 DictationStatus status, List<String> segments, String partial, String finishedText, String text, double level, DictationUnavailableReason? reason, DictationFailure? failure, bool restricted, bool finished, bool showOverlay
});




}
/// @nodoc
class __$DictationStateCopyWithImpl<$Res>
    implements _$DictationStateCopyWith<$Res> {
  __$DictationStateCopyWithImpl(this._self, this._then);

  final _DictationState _self;
  final $Res Function(_DictationState) _then;

/// Create a copy of DictationState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? segments = null,Object? partial = null,Object? finishedText = null,Object? text = null,Object? level = null,Object? reason = freezed,Object? failure = freezed,Object? restricted = null,Object? finished = null,Object? showOverlay = null,}) {
  return _then(_DictationState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as DictationStatus,segments: null == segments ? _self._segments : segments // ignore: cast_nullable_to_non_nullable
as List<String>,partial: null == partial ? _self.partial : partial // ignore: cast_nullable_to_non_nullable
as String,finishedText: null == finishedText ? _self.finishedText : finishedText // ignore: cast_nullable_to_non_nullable
as String,text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,level: null == level ? _self.level : level // ignore: cast_nullable_to_non_nullable
as double,reason: freezed == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as DictationUnavailableReason?,failure: freezed == failure ? _self.failure : failure // ignore: cast_nullable_to_non_nullable
as DictationFailure?,restricted: null == restricted ? _self.restricted : restricted // ignore: cast_nullable_to_non_nullable
as bool,finished: null == finished ? _self.finished : finished // ignore: cast_nullable_to_non_nullable
as bool,showOverlay: null == showOverlay ? _self.showOverlay : showOverlay // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
