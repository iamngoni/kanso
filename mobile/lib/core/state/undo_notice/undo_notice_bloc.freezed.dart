// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'undo_notice_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$UndoNoticeEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UndoNoticeEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'UndoNoticeEvent()';
}


}

/// @nodoc
class $UndoNoticeEventCopyWith<$Res>  {
$UndoNoticeEventCopyWith(UndoNoticeEvent _, $Res Function(UndoNoticeEvent) __);
}


/// Adds pattern-matching-related methods to [UndoNoticeEvent].
extension UndoNoticeEventPatterns on UndoNoticeEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( UndoNoticeShownEvent value)?  shown,TResult Function( UndoNoticePressedEvent value)?  undoPressed,TResult Function( UndoNoticeDismissedEvent value)?  dismissed,TResult Function( UndoNoticeTimedOutEvent value)?  timedOut,TResult Function( UndoNoticeErrorDismissedEvent value)?  errorDismissed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case UndoNoticeShownEvent() when shown != null:
return shown(_that);case UndoNoticePressedEvent() when undoPressed != null:
return undoPressed(_that);case UndoNoticeDismissedEvent() when dismissed != null:
return dismissed(_that);case UndoNoticeTimedOutEvent() when timedOut != null:
return timedOut(_that);case UndoNoticeErrorDismissedEvent() when errorDismissed != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( UndoNoticeShownEvent value)  shown,required TResult Function( UndoNoticePressedEvent value)  undoPressed,required TResult Function( UndoNoticeDismissedEvent value)  dismissed,required TResult Function( UndoNoticeTimedOutEvent value)  timedOut,required TResult Function( UndoNoticeErrorDismissedEvent value)  errorDismissed,}){
final _that = this;
switch (_that) {
case UndoNoticeShownEvent():
return shown(_that);case UndoNoticePressedEvent():
return undoPressed(_that);case UndoNoticeDismissedEvent():
return dismissed(_that);case UndoNoticeTimedOutEvent():
return timedOut(_that);case UndoNoticeErrorDismissedEvent():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( UndoNoticeShownEvent value)?  shown,TResult? Function( UndoNoticePressedEvent value)?  undoPressed,TResult? Function( UndoNoticeDismissedEvent value)?  dismissed,TResult? Function( UndoNoticeTimedOutEvent value)?  timedOut,TResult? Function( UndoNoticeErrorDismissedEvent value)?  errorDismissed,}){
final _that = this;
switch (_that) {
case UndoNoticeShownEvent() when shown != null:
return shown(_that);case UndoNoticePressedEvent() when undoPressed != null:
return undoPressed(_that);case UndoNoticeDismissedEvent() when dismissed != null:
return dismissed(_that);case UndoNoticeTimedOutEvent() when timedOut != null:
return timedOut(_that);case UndoNoticeErrorDismissedEvent() when errorDismissed != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( int ticket,  String message)?  shown,TResult Function()?  undoPressed,TResult Function()?  dismissed,TResult Function( int serial)?  timedOut,TResult Function()?  errorDismissed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case UndoNoticeShownEvent() when shown != null:
return shown(_that.ticket,_that.message);case UndoNoticePressedEvent() when undoPressed != null:
return undoPressed();case UndoNoticeDismissedEvent() when dismissed != null:
return dismissed();case UndoNoticeTimedOutEvent() when timedOut != null:
return timedOut(_that.serial);case UndoNoticeErrorDismissedEvent() when errorDismissed != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( int ticket,  String message)  shown,required TResult Function()  undoPressed,required TResult Function()  dismissed,required TResult Function( int serial)  timedOut,required TResult Function()  errorDismissed,}) {final _that = this;
switch (_that) {
case UndoNoticeShownEvent():
return shown(_that.ticket,_that.message);case UndoNoticePressedEvent():
return undoPressed();case UndoNoticeDismissedEvent():
return dismissed();case UndoNoticeTimedOutEvent():
return timedOut(_that.serial);case UndoNoticeErrorDismissedEvent():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( int ticket,  String message)?  shown,TResult? Function()?  undoPressed,TResult? Function()?  dismissed,TResult? Function( int serial)?  timedOut,TResult? Function()?  errorDismissed,}) {final _that = this;
switch (_that) {
case UndoNoticeShownEvent() when shown != null:
return shown(_that.ticket,_that.message);case UndoNoticePressedEvent() when undoPressed != null:
return undoPressed();case UndoNoticeDismissedEvent() when dismissed != null:
return dismissed();case UndoNoticeTimedOutEvent() when timedOut != null:
return timedOut(_that.serial);case UndoNoticeErrorDismissedEvent() when errorDismissed != null:
return errorDismissed();case _:
  return null;

}
}

}

/// @nodoc


class UndoNoticeShownEvent implements UndoNoticeEvent {
  const UndoNoticeShownEvent(this.ticket, this.message);
  

 final  int ticket;
 final  String message;

/// Create a copy of UndoNoticeEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$UndoNoticeShownEventCopyWith<UndoNoticeShownEvent> get copyWith => _$UndoNoticeShownEventCopyWithImpl<UndoNoticeShownEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UndoNoticeShownEvent&&(identical(other.ticket, ticket) || other.ticket == ticket)&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,ticket,message);

@override
String toString() {
  return 'UndoNoticeEvent.shown(ticket: $ticket, message: $message)';
}


}

/// @nodoc
abstract mixin class $UndoNoticeShownEventCopyWith<$Res> implements $UndoNoticeEventCopyWith<$Res> {
  factory $UndoNoticeShownEventCopyWith(UndoNoticeShownEvent value, $Res Function(UndoNoticeShownEvent) _then) = _$UndoNoticeShownEventCopyWithImpl;
@useResult
$Res call({
 int ticket, String message
});




}
/// @nodoc
class _$UndoNoticeShownEventCopyWithImpl<$Res>
    implements $UndoNoticeShownEventCopyWith<$Res> {
  _$UndoNoticeShownEventCopyWithImpl(this._self, this._then);

  final UndoNoticeShownEvent _self;
  final $Res Function(UndoNoticeShownEvent) _then;

/// Create a copy of UndoNoticeEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? ticket = null,Object? message = null,}) {
  return _then(UndoNoticeShownEvent(
null == ticket ? _self.ticket : ticket // ignore: cast_nullable_to_non_nullable
as int,null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class UndoNoticePressedEvent implements UndoNoticeEvent {
  const UndoNoticePressedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UndoNoticePressedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'UndoNoticeEvent.undoPressed()';
}


}




/// @nodoc


class UndoNoticeDismissedEvent implements UndoNoticeEvent {
  const UndoNoticeDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UndoNoticeDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'UndoNoticeEvent.dismissed()';
}


}




/// @nodoc


class UndoNoticeTimedOutEvent implements UndoNoticeEvent {
  const UndoNoticeTimedOutEvent(this.serial);
  

 final  int serial;

/// Create a copy of UndoNoticeEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$UndoNoticeTimedOutEventCopyWith<UndoNoticeTimedOutEvent> get copyWith => _$UndoNoticeTimedOutEventCopyWithImpl<UndoNoticeTimedOutEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UndoNoticeTimedOutEvent&&(identical(other.serial, serial) || other.serial == serial));
}


@override
int get hashCode => Object.hash(runtimeType,serial);

@override
String toString() {
  return 'UndoNoticeEvent.timedOut(serial: $serial)';
}


}

/// @nodoc
abstract mixin class $UndoNoticeTimedOutEventCopyWith<$Res> implements $UndoNoticeEventCopyWith<$Res> {
  factory $UndoNoticeTimedOutEventCopyWith(UndoNoticeTimedOutEvent value, $Res Function(UndoNoticeTimedOutEvent) _then) = _$UndoNoticeTimedOutEventCopyWithImpl;
@useResult
$Res call({
 int serial
});




}
/// @nodoc
class _$UndoNoticeTimedOutEventCopyWithImpl<$Res>
    implements $UndoNoticeTimedOutEventCopyWith<$Res> {
  _$UndoNoticeTimedOutEventCopyWithImpl(this._self, this._then);

  final UndoNoticeTimedOutEvent _self;
  final $Res Function(UndoNoticeTimedOutEvent) _then;

/// Create a copy of UndoNoticeEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? serial = null,}) {
  return _then(UndoNoticeTimedOutEvent(
null == serial ? _self.serial : serial // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class UndoNoticeErrorDismissedEvent implements UndoNoticeEvent {
  const UndoNoticeErrorDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UndoNoticeErrorDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'UndoNoticeEvent.errorDismissed()';
}


}




/// @nodoc
mixin _$UndoNoticeState {

/// The notice showing; null when there is none.
 UndoNotice? get notice;/// Whether Undo is being carried out.
 bool get undoing;/// Why the last undo failed, until it is dismissed.
 ApplicationError? get error;
/// Create a copy of UndoNoticeState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$UndoNoticeStateCopyWith<UndoNoticeState> get copyWith => _$UndoNoticeStateCopyWithImpl<UndoNoticeState>(this as UndoNoticeState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UndoNoticeState&&(identical(other.notice, notice) || other.notice == notice)&&(identical(other.undoing, undoing) || other.undoing == undoing)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,notice,undoing,error);

@override
String toString() {
  return 'UndoNoticeState(notice: $notice, undoing: $undoing, error: $error)';
}


}

/// @nodoc
abstract mixin class $UndoNoticeStateCopyWith<$Res>  {
  factory $UndoNoticeStateCopyWith(UndoNoticeState value, $Res Function(UndoNoticeState) _then) = _$UndoNoticeStateCopyWithImpl;
@useResult
$Res call({
 UndoNotice? notice, bool undoing, ApplicationError? error
});


$UndoNoticeCopyWith<$Res>? get notice;$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$UndoNoticeStateCopyWithImpl<$Res>
    implements $UndoNoticeStateCopyWith<$Res> {
  _$UndoNoticeStateCopyWithImpl(this._self, this._then);

  final UndoNoticeState _self;
  final $Res Function(UndoNoticeState) _then;

/// Create a copy of UndoNoticeState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? notice = freezed,Object? undoing = null,Object? error = freezed,}) {
  return _then(_self.copyWith(
notice: freezed == notice ? _self.notice : notice // ignore: cast_nullable_to_non_nullable
as UndoNotice?,undoing: null == undoing ? _self.undoing : undoing // ignore: cast_nullable_to_non_nullable
as bool,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of UndoNoticeState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$UndoNoticeCopyWith<$Res>? get notice {
    if (_self.notice == null) {
    return null;
  }

  return $UndoNoticeCopyWith<$Res>(_self.notice!, (value) {
    return _then(_self.copyWith(notice: value));
  });
}/// Create a copy of UndoNoticeState
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


/// Adds pattern-matching-related methods to [UndoNoticeState].
extension UndoNoticeStatePatterns on UndoNoticeState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _UndoNoticeState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _UndoNoticeState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _UndoNoticeState value)  $default,){
final _that = this;
switch (_that) {
case _UndoNoticeState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _UndoNoticeState value)?  $default,){
final _that = this;
switch (_that) {
case _UndoNoticeState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( UndoNotice? notice,  bool undoing,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _UndoNoticeState() when $default != null:
return $default(_that.notice,_that.undoing,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( UndoNotice? notice,  bool undoing,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _UndoNoticeState():
return $default(_that.notice,_that.undoing,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( UndoNotice? notice,  bool undoing,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _UndoNoticeState() when $default != null:
return $default(_that.notice,_that.undoing,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _UndoNoticeState implements UndoNoticeState {
  const _UndoNoticeState({this.notice, this.undoing = false, this.error});
  

/// The notice showing; null when there is none.
@override final  UndoNotice? notice;
/// Whether Undo is being carried out.
@override@JsonKey() final  bool undoing;
/// Why the last undo failed, until it is dismissed.
@override final  ApplicationError? error;

/// Create a copy of UndoNoticeState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$UndoNoticeStateCopyWith<_UndoNoticeState> get copyWith => __$UndoNoticeStateCopyWithImpl<_UndoNoticeState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _UndoNoticeState&&(identical(other.notice, notice) || other.notice == notice)&&(identical(other.undoing, undoing) || other.undoing == undoing)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,notice,undoing,error);

@override
String toString() {
  return 'UndoNoticeState(notice: $notice, undoing: $undoing, error: $error)';
}


}

/// @nodoc
abstract mixin class _$UndoNoticeStateCopyWith<$Res> implements $UndoNoticeStateCopyWith<$Res> {
  factory _$UndoNoticeStateCopyWith(_UndoNoticeState value, $Res Function(_UndoNoticeState) _then) = __$UndoNoticeStateCopyWithImpl;
@override @useResult
$Res call({
 UndoNotice? notice, bool undoing, ApplicationError? error
});


@override $UndoNoticeCopyWith<$Res>? get notice;@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$UndoNoticeStateCopyWithImpl<$Res>
    implements _$UndoNoticeStateCopyWith<$Res> {
  __$UndoNoticeStateCopyWithImpl(this._self, this._then);

  final _UndoNoticeState _self;
  final $Res Function(_UndoNoticeState) _then;

/// Create a copy of UndoNoticeState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? notice = freezed,Object? undoing = null,Object? error = freezed,}) {
  return _then(_UndoNoticeState(
notice: freezed == notice ? _self.notice : notice // ignore: cast_nullable_to_non_nullable
as UndoNotice?,undoing: null == undoing ? _self.undoing : undoing // ignore: cast_nullable_to_non_nullable
as bool,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of UndoNoticeState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$UndoNoticeCopyWith<$Res>? get notice {
    if (_self.notice == null) {
    return null;
  }

  return $UndoNoticeCopyWith<$Res>(_self.notice!, (value) {
    return _then(_self.copyWith(notice: value));
  });
}/// Create a copy of UndoNoticeState
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
