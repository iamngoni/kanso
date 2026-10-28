// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'journal_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$JournalEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is JournalEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'JournalEvent()';
}


}

/// @nodoc
class $JournalEventCopyWith<$Res>  {
$JournalEventCopyWith(JournalEvent _, $Res Function(JournalEvent) __);
}


/// Adds pattern-matching-related methods to [JournalEvent].
extension JournalEventPatterns on JournalEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( JournalTodayRequestedEvent value)?  todayRequested,TResult Function( JournalStepRequestedEvent value)?  stepRequested,TResult Function( JournalDayRequestedEvent value)?  dayRequested,TResult Function( JournalFromTemplateRequestedEvent value)?  fromTemplateRequested,TResult Function( JournalNewTemplateRequestedEvent value)?  newTemplateRequested,TResult Function( JournalErrorDismissedEvent value)?  errorDismissed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case JournalTodayRequestedEvent() when todayRequested != null:
return todayRequested(_that);case JournalStepRequestedEvent() when stepRequested != null:
return stepRequested(_that);case JournalDayRequestedEvent() when dayRequested != null:
return dayRequested(_that);case JournalFromTemplateRequestedEvent() when fromTemplateRequested != null:
return fromTemplateRequested(_that);case JournalNewTemplateRequestedEvent() when newTemplateRequested != null:
return newTemplateRequested(_that);case JournalErrorDismissedEvent() when errorDismissed != null:
return errorDismissed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( JournalTodayRequestedEvent value)  todayRequested,required TResult Function( JournalStepRequestedEvent value)  stepRequested,required TResult Function( JournalDayRequestedEvent value)  dayRequested,required TResult Function( JournalFromTemplateRequestedEvent value)  fromTemplateRequested,required TResult Function( JournalNewTemplateRequestedEvent value)  newTemplateRequested,required TResult Function( JournalErrorDismissedEvent value)  errorDismissed,}){
final _that = this;
switch (_that) {
case JournalTodayRequestedEvent():
return todayRequested(_that);case JournalStepRequestedEvent():
return stepRequested(_that);case JournalDayRequestedEvent():
return dayRequested(_that);case JournalFromTemplateRequestedEvent():
return fromTemplateRequested(_that);case JournalNewTemplateRequestedEvent():
return newTemplateRequested(_that);case JournalErrorDismissedEvent():
return errorDismissed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( JournalTodayRequestedEvent value)?  todayRequested,TResult? Function( JournalStepRequestedEvent value)?  stepRequested,TResult? Function( JournalDayRequestedEvent value)?  dayRequested,TResult? Function( JournalFromTemplateRequestedEvent value)?  fromTemplateRequested,TResult? Function( JournalNewTemplateRequestedEvent value)?  newTemplateRequested,TResult? Function( JournalErrorDismissedEvent value)?  errorDismissed,}){
final _that = this;
switch (_that) {
case JournalTodayRequestedEvent() when todayRequested != null:
return todayRequested(_that);case JournalStepRequestedEvent() when stepRequested != null:
return stepRequested(_that);case JournalDayRequestedEvent() when dayRequested != null:
return dayRequested(_that);case JournalFromTemplateRequestedEvent() when fromTemplateRequested != null:
return fromTemplateRequested(_that);case JournalNewTemplateRequestedEvent() when newTemplateRequested != null:
return newTemplateRequested(_that);case JournalErrorDismissedEvent() when errorDismissed != null:
return errorDismissed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  todayRequested,TResult Function( String? openNote,  bool forward)?  stepRequested,TResult Function( DayDate day)?  dayRequested,TResult Function( String templateId,  String? notebookId)?  fromTemplateRequested,TResult Function()?  newTemplateRequested,TResult Function()?  errorDismissed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case JournalTodayRequestedEvent() when todayRequested != null:
return todayRequested();case JournalStepRequestedEvent() when stepRequested != null:
return stepRequested(_that.openNote,_that.forward);case JournalDayRequestedEvent() when dayRequested != null:
return dayRequested(_that.day);case JournalFromTemplateRequestedEvent() when fromTemplateRequested != null:
return fromTemplateRequested(_that.templateId,_that.notebookId);case JournalNewTemplateRequestedEvent() when newTemplateRequested != null:
return newTemplateRequested();case JournalErrorDismissedEvent() when errorDismissed != null:
return errorDismissed();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  todayRequested,required TResult Function( String? openNote,  bool forward)  stepRequested,required TResult Function( DayDate day)  dayRequested,required TResult Function( String templateId,  String? notebookId)  fromTemplateRequested,required TResult Function()  newTemplateRequested,required TResult Function()  errorDismissed,}) {final _that = this;
switch (_that) {
case JournalTodayRequestedEvent():
return todayRequested();case JournalStepRequestedEvent():
return stepRequested(_that.openNote,_that.forward);case JournalDayRequestedEvent():
return dayRequested(_that.day);case JournalFromTemplateRequestedEvent():
return fromTemplateRequested(_that.templateId,_that.notebookId);case JournalNewTemplateRequestedEvent():
return newTemplateRequested();case JournalErrorDismissedEvent():
return errorDismissed();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  todayRequested,TResult? Function( String? openNote,  bool forward)?  stepRequested,TResult? Function( DayDate day)?  dayRequested,TResult? Function( String templateId,  String? notebookId)?  fromTemplateRequested,TResult? Function()?  newTemplateRequested,TResult? Function()?  errorDismissed,}) {final _that = this;
switch (_that) {
case JournalTodayRequestedEvent() when todayRequested != null:
return todayRequested();case JournalStepRequestedEvent() when stepRequested != null:
return stepRequested(_that.openNote,_that.forward);case JournalDayRequestedEvent() when dayRequested != null:
return dayRequested(_that.day);case JournalFromTemplateRequestedEvent() when fromTemplateRequested != null:
return fromTemplateRequested(_that.templateId,_that.notebookId);case JournalNewTemplateRequestedEvent() when newTemplateRequested != null:
return newTemplateRequested();case JournalErrorDismissedEvent() when errorDismissed != null:
return errorDismissed();case _:
  return null;

}
}

}

/// @nodoc


class JournalTodayRequestedEvent implements JournalEvent {
  const JournalTodayRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is JournalTodayRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'JournalEvent.todayRequested()';
}


}




/// @nodoc


class JournalStepRequestedEvent implements JournalEvent {
  const JournalStepRequestedEvent({required this.openNote, required this.forward});
  

 final  String? openNote;
 final  bool forward;

/// Create a copy of JournalEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$JournalStepRequestedEventCopyWith<JournalStepRequestedEvent> get copyWith => _$JournalStepRequestedEventCopyWithImpl<JournalStepRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is JournalStepRequestedEvent&&(identical(other.openNote, openNote) || other.openNote == openNote)&&(identical(other.forward, forward) || other.forward == forward));
}


@override
int get hashCode => Object.hash(runtimeType,openNote,forward);

@override
String toString() {
  return 'JournalEvent.stepRequested(openNote: $openNote, forward: $forward)';
}


}

/// @nodoc
abstract mixin class $JournalStepRequestedEventCopyWith<$Res> implements $JournalEventCopyWith<$Res> {
  factory $JournalStepRequestedEventCopyWith(JournalStepRequestedEvent value, $Res Function(JournalStepRequestedEvent) _then) = _$JournalStepRequestedEventCopyWithImpl;
@useResult
$Res call({
 String? openNote, bool forward
});




}
/// @nodoc
class _$JournalStepRequestedEventCopyWithImpl<$Res>
    implements $JournalStepRequestedEventCopyWith<$Res> {
  _$JournalStepRequestedEventCopyWithImpl(this._self, this._then);

  final JournalStepRequestedEvent _self;
  final $Res Function(JournalStepRequestedEvent) _then;

/// Create a copy of JournalEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? openNote = freezed,Object? forward = null,}) {
  return _then(JournalStepRequestedEvent(
openNote: freezed == openNote ? _self.openNote : openNote // ignore: cast_nullable_to_non_nullable
as String?,forward: null == forward ? _self.forward : forward // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class JournalDayRequestedEvent implements JournalEvent {
  const JournalDayRequestedEvent(this.day);
  

 final  DayDate day;

/// Create a copy of JournalEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$JournalDayRequestedEventCopyWith<JournalDayRequestedEvent> get copyWith => _$JournalDayRequestedEventCopyWithImpl<JournalDayRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is JournalDayRequestedEvent&&(identical(other.day, day) || other.day == day));
}


@override
int get hashCode => Object.hash(runtimeType,day);

@override
String toString() {
  return 'JournalEvent.dayRequested(day: $day)';
}


}

/// @nodoc
abstract mixin class $JournalDayRequestedEventCopyWith<$Res> implements $JournalEventCopyWith<$Res> {
  factory $JournalDayRequestedEventCopyWith(JournalDayRequestedEvent value, $Res Function(JournalDayRequestedEvent) _then) = _$JournalDayRequestedEventCopyWithImpl;
@useResult
$Res call({
 DayDate day
});




}
/// @nodoc
class _$JournalDayRequestedEventCopyWithImpl<$Res>
    implements $JournalDayRequestedEventCopyWith<$Res> {
  _$JournalDayRequestedEventCopyWithImpl(this._self, this._then);

  final JournalDayRequestedEvent _self;
  final $Res Function(JournalDayRequestedEvent) _then;

/// Create a copy of JournalEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? day = null,}) {
  return _then(JournalDayRequestedEvent(
null == day ? _self.day : day // ignore: cast_nullable_to_non_nullable
as DayDate,
  ));
}


}

/// @nodoc


class JournalFromTemplateRequestedEvent implements JournalEvent {
  const JournalFromTemplateRequestedEvent({required this.templateId, this.notebookId});
  

 final  String templateId;
 final  String? notebookId;

/// Create a copy of JournalEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$JournalFromTemplateRequestedEventCopyWith<JournalFromTemplateRequestedEvent> get copyWith => _$JournalFromTemplateRequestedEventCopyWithImpl<JournalFromTemplateRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is JournalFromTemplateRequestedEvent&&(identical(other.templateId, templateId) || other.templateId == templateId)&&(identical(other.notebookId, notebookId) || other.notebookId == notebookId));
}


@override
int get hashCode => Object.hash(runtimeType,templateId,notebookId);

@override
String toString() {
  return 'JournalEvent.fromTemplateRequested(templateId: $templateId, notebookId: $notebookId)';
}


}

/// @nodoc
abstract mixin class $JournalFromTemplateRequestedEventCopyWith<$Res> implements $JournalEventCopyWith<$Res> {
  factory $JournalFromTemplateRequestedEventCopyWith(JournalFromTemplateRequestedEvent value, $Res Function(JournalFromTemplateRequestedEvent) _then) = _$JournalFromTemplateRequestedEventCopyWithImpl;
@useResult
$Res call({
 String templateId, String? notebookId
});




}
/// @nodoc
class _$JournalFromTemplateRequestedEventCopyWithImpl<$Res>
    implements $JournalFromTemplateRequestedEventCopyWith<$Res> {
  _$JournalFromTemplateRequestedEventCopyWithImpl(this._self, this._then);

  final JournalFromTemplateRequestedEvent _self;
  final $Res Function(JournalFromTemplateRequestedEvent) _then;

/// Create a copy of JournalEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? templateId = null,Object? notebookId = freezed,}) {
  return _then(JournalFromTemplateRequestedEvent(
templateId: null == templateId ? _self.templateId : templateId // ignore: cast_nullable_to_non_nullable
as String,notebookId: freezed == notebookId ? _self.notebookId : notebookId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class JournalNewTemplateRequestedEvent implements JournalEvent {
  const JournalNewTemplateRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is JournalNewTemplateRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'JournalEvent.newTemplateRequested()';
}


}




/// @nodoc


class JournalErrorDismissedEvent implements JournalEvent {
  const JournalErrorDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is JournalErrorDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'JournalEvent.errorDismissed()';
}


}




/// @nodoc
mixin _$JournalState {

/// Whether a note is being found or made.
 bool get working;/// The note to open; null until one is made, and again while the next
/// is being made.
 MadeNote? get opened;/// Why the last attempt failed, until it is dismissed.
 ApplicationError? get error;
/// Create a copy of JournalState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$JournalStateCopyWith<JournalState> get copyWith => _$JournalStateCopyWithImpl<JournalState>(this as JournalState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is JournalState&&(identical(other.working, working) || other.working == working)&&(identical(other.opened, opened) || other.opened == opened)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,working,opened,error);

@override
String toString() {
  return 'JournalState(working: $working, opened: $opened, error: $error)';
}


}

/// @nodoc
abstract mixin class $JournalStateCopyWith<$Res>  {
  factory $JournalStateCopyWith(JournalState value, $Res Function(JournalState) _then) = _$JournalStateCopyWithImpl;
@useResult
$Res call({
 bool working, MadeNote? opened, ApplicationError? error
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$JournalStateCopyWithImpl<$Res>
    implements $JournalStateCopyWith<$Res> {
  _$JournalStateCopyWithImpl(this._self, this._then);

  final JournalState _self;
  final $Res Function(JournalState) _then;

/// Create a copy of JournalState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? working = null,Object? opened = freezed,Object? error = freezed,}) {
  return _then(_self.copyWith(
working: null == working ? _self.working : working // ignore: cast_nullable_to_non_nullable
as bool,opened: freezed == opened ? _self.opened : opened // ignore: cast_nullable_to_non_nullable
as MadeNote?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of JournalState
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


/// Adds pattern-matching-related methods to [JournalState].
extension JournalStatePatterns on JournalState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _JournalState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _JournalState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _JournalState value)  $default,){
final _that = this;
switch (_that) {
case _JournalState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _JournalState value)?  $default,){
final _that = this;
switch (_that) {
case _JournalState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool working,  MadeNote? opened,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _JournalState() when $default != null:
return $default(_that.working,_that.opened,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool working,  MadeNote? opened,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _JournalState():
return $default(_that.working,_that.opened,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool working,  MadeNote? opened,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _JournalState() when $default != null:
return $default(_that.working,_that.opened,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _JournalState implements JournalState {
  const _JournalState({this.working = false, this.opened, this.error});
  

/// Whether a note is being found or made.
@override@JsonKey() final  bool working;
/// The note to open; null until one is made, and again while the next
/// is being made.
@override final  MadeNote? opened;
/// Why the last attempt failed, until it is dismissed.
@override final  ApplicationError? error;

/// Create a copy of JournalState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$JournalStateCopyWith<_JournalState> get copyWith => __$JournalStateCopyWithImpl<_JournalState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _JournalState&&(identical(other.working, working) || other.working == working)&&(identical(other.opened, opened) || other.opened == opened)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,working,opened,error);

@override
String toString() {
  return 'JournalState(working: $working, opened: $opened, error: $error)';
}


}

/// @nodoc
abstract mixin class _$JournalStateCopyWith<$Res> implements $JournalStateCopyWith<$Res> {
  factory _$JournalStateCopyWith(_JournalState value, $Res Function(_JournalState) _then) = __$JournalStateCopyWithImpl;
@override @useResult
$Res call({
 bool working, MadeNote? opened, ApplicationError? error
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$JournalStateCopyWithImpl<$Res>
    implements _$JournalStateCopyWith<$Res> {
  __$JournalStateCopyWithImpl(this._self, this._then);

  final _JournalState _self;
  final $Res Function(_JournalState) _then;

/// Create a copy of JournalState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? working = null,Object? opened = freezed,Object? error = freezed,}) {
  return _then(_JournalState(
working: null == working ? _self.working : working // ignore: cast_nullable_to_non_nullable
as bool,opened: freezed == opened ? _self.opened : opened // ignore: cast_nullable_to_non_nullable
as MadeNote?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of JournalState
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
