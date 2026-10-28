// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'voice_note_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$VoiceNoteEvent {

 String get transcript;
/// Create a copy of VoiceNoteEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$VoiceNoteEventCopyWith<VoiceNoteEvent> get copyWith => _$VoiceNoteEventCopyWithImpl<VoiceNoteEvent>(this as VoiceNoteEvent, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is VoiceNoteEvent&&(identical(other.transcript, transcript) || other.transcript == transcript));
}


@override
int get hashCode => Object.hash(runtimeType,transcript);

@override
String toString() {
  return 'VoiceNoteEvent(transcript: $transcript)';
}


}

/// @nodoc
abstract mixin class $VoiceNoteEventCopyWith<$Res>  {
  factory $VoiceNoteEventCopyWith(VoiceNoteEvent value, $Res Function(VoiceNoteEvent) _then) = _$VoiceNoteEventCopyWithImpl;
@useResult
$Res call({
 String transcript
});




}
/// @nodoc
class _$VoiceNoteEventCopyWithImpl<$Res>
    implements $VoiceNoteEventCopyWith<$Res> {
  _$VoiceNoteEventCopyWithImpl(this._self, this._then);

  final VoiceNoteEvent _self;
  final $Res Function(VoiceNoteEvent) _then;

/// Create a copy of VoiceNoteEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? transcript = null,}) {
  return _then(_self.copyWith(
transcript: null == transcript ? _self.transcript : transcript // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [VoiceNoteEvent].
extension VoiceNoteEventPatterns on VoiceNoteEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( VoiceNoteSaveRequestedEvent value)?  saveRequested,required TResult orElse(),}){
final _that = this;
switch (_that) {
case VoiceNoteSaveRequestedEvent() when saveRequested != null:
return saveRequested(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( VoiceNoteSaveRequestedEvent value)  saveRequested,}){
final _that = this;
switch (_that) {
case VoiceNoteSaveRequestedEvent():
return saveRequested(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( VoiceNoteSaveRequestedEvent value)?  saveRequested,}){
final _that = this;
switch (_that) {
case VoiceNoteSaveRequestedEvent() when saveRequested != null:
return saveRequested(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String transcript)?  saveRequested,required TResult orElse(),}) {final _that = this;
switch (_that) {
case VoiceNoteSaveRequestedEvent() when saveRequested != null:
return saveRequested(_that.transcript);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String transcript)  saveRequested,}) {final _that = this;
switch (_that) {
case VoiceNoteSaveRequestedEvent():
return saveRequested(_that.transcript);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String transcript)?  saveRequested,}) {final _that = this;
switch (_that) {
case VoiceNoteSaveRequestedEvent() when saveRequested != null:
return saveRequested(_that.transcript);case _:
  return null;

}
}

}

/// @nodoc


class VoiceNoteSaveRequestedEvent implements VoiceNoteEvent {
  const VoiceNoteSaveRequestedEvent(this.transcript);
  

@override final  String transcript;

/// Create a copy of VoiceNoteEvent
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$VoiceNoteSaveRequestedEventCopyWith<VoiceNoteSaveRequestedEvent> get copyWith => _$VoiceNoteSaveRequestedEventCopyWithImpl<VoiceNoteSaveRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is VoiceNoteSaveRequestedEvent&&(identical(other.transcript, transcript) || other.transcript == transcript));
}


@override
int get hashCode => Object.hash(runtimeType,transcript);

@override
String toString() {
  return 'VoiceNoteEvent.saveRequested(transcript: $transcript)';
}


}

/// @nodoc
abstract mixin class $VoiceNoteSaveRequestedEventCopyWith<$Res> implements $VoiceNoteEventCopyWith<$Res> {
  factory $VoiceNoteSaveRequestedEventCopyWith(VoiceNoteSaveRequestedEvent value, $Res Function(VoiceNoteSaveRequestedEvent) _then) = _$VoiceNoteSaveRequestedEventCopyWithImpl;
@override @useResult
$Res call({
 String transcript
});




}
/// @nodoc
class _$VoiceNoteSaveRequestedEventCopyWithImpl<$Res>
    implements $VoiceNoteSaveRequestedEventCopyWith<$Res> {
  _$VoiceNoteSaveRequestedEventCopyWithImpl(this._self, this._then);

  final VoiceNoteSaveRequestedEvent _self;
  final $Res Function(VoiceNoteSaveRequestedEvent) _then;

/// Create a copy of VoiceNoteEvent
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? transcript = null,}) {
  return _then(VoiceNoteSaveRequestedEvent(
null == transcript ? _self.transcript : transcript // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$VoiceNoteState {

 VoiceNoteStatus get status;/// The note made, while [status] is saved.
 String? get noteId;/// Why saving failed, while [status] is failed.
 ApplicationError? get error;
/// Create a copy of VoiceNoteState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$VoiceNoteStateCopyWith<VoiceNoteState> get copyWith => _$VoiceNoteStateCopyWithImpl<VoiceNoteState>(this as VoiceNoteState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is VoiceNoteState&&(identical(other.status, status) || other.status == status)&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,noteId,error);

@override
String toString() {
  return 'VoiceNoteState(status: $status, noteId: $noteId, error: $error)';
}


}

/// @nodoc
abstract mixin class $VoiceNoteStateCopyWith<$Res>  {
  factory $VoiceNoteStateCopyWith(VoiceNoteState value, $Res Function(VoiceNoteState) _then) = _$VoiceNoteStateCopyWithImpl;
@useResult
$Res call({
 VoiceNoteStatus status, String? noteId, ApplicationError? error
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$VoiceNoteStateCopyWithImpl<$Res>
    implements $VoiceNoteStateCopyWith<$Res> {
  _$VoiceNoteStateCopyWithImpl(this._self, this._then);

  final VoiceNoteState _self;
  final $Res Function(VoiceNoteState) _then;

/// Create a copy of VoiceNoteState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? noteId = freezed,Object? error = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as VoiceNoteStatus,noteId: freezed == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of VoiceNoteState
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


/// Adds pattern-matching-related methods to [VoiceNoteState].
extension VoiceNoteStatePatterns on VoiceNoteState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _VoiceNoteState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _VoiceNoteState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _VoiceNoteState value)  $default,){
final _that = this;
switch (_that) {
case _VoiceNoteState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _VoiceNoteState value)?  $default,){
final _that = this;
switch (_that) {
case _VoiceNoteState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( VoiceNoteStatus status,  String? noteId,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _VoiceNoteState() when $default != null:
return $default(_that.status,_that.noteId,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( VoiceNoteStatus status,  String? noteId,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _VoiceNoteState():
return $default(_that.status,_that.noteId,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( VoiceNoteStatus status,  String? noteId,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _VoiceNoteState() when $default != null:
return $default(_that.status,_that.noteId,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _VoiceNoteState implements VoiceNoteState {
  const _VoiceNoteState({this.status = VoiceNoteStatus.idle, this.noteId, this.error});
  

@override@JsonKey() final  VoiceNoteStatus status;
/// The note made, while [status] is saved.
@override final  String? noteId;
/// Why saving failed, while [status] is failed.
@override final  ApplicationError? error;

/// Create a copy of VoiceNoteState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$VoiceNoteStateCopyWith<_VoiceNoteState> get copyWith => __$VoiceNoteStateCopyWithImpl<_VoiceNoteState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _VoiceNoteState&&(identical(other.status, status) || other.status == status)&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,noteId,error);

@override
String toString() {
  return 'VoiceNoteState(status: $status, noteId: $noteId, error: $error)';
}


}

/// @nodoc
abstract mixin class _$VoiceNoteStateCopyWith<$Res> implements $VoiceNoteStateCopyWith<$Res> {
  factory _$VoiceNoteStateCopyWith(_VoiceNoteState value, $Res Function(_VoiceNoteState) _then) = __$VoiceNoteStateCopyWithImpl;
@override @useResult
$Res call({
 VoiceNoteStatus status, String? noteId, ApplicationError? error
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$VoiceNoteStateCopyWithImpl<$Res>
    implements _$VoiceNoteStateCopyWith<$Res> {
  __$VoiceNoteStateCopyWithImpl(this._self, this._then);

  final _VoiceNoteState _self;
  final $Res Function(_VoiceNoteState) _then;

/// Create a copy of VoiceNoteState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? noteId = freezed,Object? error = freezed,}) {
  return _then(_VoiceNoteState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as VoiceNoteStatus,noteId: freezed == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of VoiceNoteState
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
