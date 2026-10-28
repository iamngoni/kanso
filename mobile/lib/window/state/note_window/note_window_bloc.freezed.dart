// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'note_window_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$NoteWindowEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteWindowEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteWindowEvent()';
}


}

/// @nodoc
class $NoteWindowEventCopyWith<$Res>  {
$NoteWindowEventCopyWith(NoteWindowEvent _, $Res Function(NoteWindowEvent) __);
}


/// Adds pattern-matching-related methods to [NoteWindowEvent].
extension NoteWindowEventPatterns on NoteWindowEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( NoteWindowStartedEvent value)?  started,TResult Function( NoteWindowLookedAtEvent value)?  lookedAt,TResult Function( NoteWindowRestoreRequestedEvent value)?  restoreRequested,required TResult orElse(),}){
final _that = this;
switch (_that) {
case NoteWindowStartedEvent() when started != null:
return started(_that);case NoteWindowLookedAtEvent() when lookedAt != null:
return lookedAt(_that);case NoteWindowRestoreRequestedEvent() when restoreRequested != null:
return restoreRequested(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( NoteWindowStartedEvent value)  started,required TResult Function( NoteWindowLookedAtEvent value)  lookedAt,required TResult Function( NoteWindowRestoreRequestedEvent value)  restoreRequested,}){
final _that = this;
switch (_that) {
case NoteWindowStartedEvent():
return started(_that);case NoteWindowLookedAtEvent():
return lookedAt(_that);case NoteWindowRestoreRequestedEvent():
return restoreRequested(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( NoteWindowStartedEvent value)?  started,TResult? Function( NoteWindowLookedAtEvent value)?  lookedAt,TResult? Function( NoteWindowRestoreRequestedEvent value)?  restoreRequested,}){
final _that = this;
switch (_that) {
case NoteWindowStartedEvent() when started != null:
return started(_that);case NoteWindowLookedAtEvent() when lookedAt != null:
return lookedAt(_that);case NoteWindowRestoreRequestedEvent() when restoreRequested != null:
return restoreRequested(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String noteId)?  started,TResult Function()?  lookedAt,TResult Function()?  restoreRequested,required TResult orElse(),}) {final _that = this;
switch (_that) {
case NoteWindowStartedEvent() when started != null:
return started(_that.noteId);case NoteWindowLookedAtEvent() when lookedAt != null:
return lookedAt();case NoteWindowRestoreRequestedEvent() when restoreRequested != null:
return restoreRequested();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String noteId)  started,required TResult Function()  lookedAt,required TResult Function()  restoreRequested,}) {final _that = this;
switch (_that) {
case NoteWindowStartedEvent():
return started(_that.noteId);case NoteWindowLookedAtEvent():
return lookedAt();case NoteWindowRestoreRequestedEvent():
return restoreRequested();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String noteId)?  started,TResult? Function()?  lookedAt,TResult? Function()?  restoreRequested,}) {final _that = this;
switch (_that) {
case NoteWindowStartedEvent() when started != null:
return started(_that.noteId);case NoteWindowLookedAtEvent() when lookedAt != null:
return lookedAt();case NoteWindowRestoreRequestedEvent() when restoreRequested != null:
return restoreRequested();case _:
  return null;

}
}

}

/// @nodoc


class NoteWindowStartedEvent implements NoteWindowEvent {
  const NoteWindowStartedEvent(this.noteId);
  

 final  String noteId;

/// Create a copy of NoteWindowEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteWindowStartedEventCopyWith<NoteWindowStartedEvent> get copyWith => _$NoteWindowStartedEventCopyWithImpl<NoteWindowStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteWindowStartedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'NoteWindowEvent.started(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $NoteWindowStartedEventCopyWith<$Res> implements $NoteWindowEventCopyWith<$Res> {
  factory $NoteWindowStartedEventCopyWith(NoteWindowStartedEvent value, $Res Function(NoteWindowStartedEvent) _then) = _$NoteWindowStartedEventCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$NoteWindowStartedEventCopyWithImpl<$Res>
    implements $NoteWindowStartedEventCopyWith<$Res> {
  _$NoteWindowStartedEventCopyWithImpl(this._self, this._then);

  final NoteWindowStartedEvent _self;
  final $Res Function(NoteWindowStartedEvent) _then;

/// Create a copy of NoteWindowEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(NoteWindowStartedEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NoteWindowLookedAtEvent implements NoteWindowEvent {
  const NoteWindowLookedAtEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteWindowLookedAtEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteWindowEvent.lookedAt()';
}


}




/// @nodoc


class NoteWindowRestoreRequestedEvent implements NoteWindowEvent {
  const NoteWindowRestoreRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteWindowRestoreRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteWindowEvent.restoreRequested()';
}


}




/// @nodoc
mixin _$NoteWindowState {

 String get noteId; NoteWindowStatus get status;/// The note's title. Titles are not sealed, but a locked note's is kept
/// out of the window's name (see `noteWindowTitle`).
 String get title;/// Whether the note is locked, open or closed.
 bool get locked;/// Whether the note is in the trash.
 bool get trashed;
/// Create a copy of NoteWindowState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteWindowStateCopyWith<NoteWindowState> get copyWith => _$NoteWindowStateCopyWithImpl<NoteWindowState>(this as NoteWindowState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteWindowState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.status, status) || other.status == status)&&(identical(other.title, title) || other.title == title)&&(identical(other.locked, locked) || other.locked == locked)&&(identical(other.trashed, trashed) || other.trashed == trashed));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,status,title,locked,trashed);

@override
String toString() {
  return 'NoteWindowState(noteId: $noteId, status: $status, title: $title, locked: $locked, trashed: $trashed)';
}


}

/// @nodoc
abstract mixin class $NoteWindowStateCopyWith<$Res>  {
  factory $NoteWindowStateCopyWith(NoteWindowState value, $Res Function(NoteWindowState) _then) = _$NoteWindowStateCopyWithImpl;
@useResult
$Res call({
 String noteId, NoteWindowStatus status, String title, bool locked, bool trashed
});




}
/// @nodoc
class _$NoteWindowStateCopyWithImpl<$Res>
    implements $NoteWindowStateCopyWith<$Res> {
  _$NoteWindowStateCopyWithImpl(this._self, this._then);

  final NoteWindowState _self;
  final $Res Function(NoteWindowState) _then;

/// Create a copy of NoteWindowState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteId = null,Object? status = null,Object? title = null,Object? locked = null,Object? trashed = null,}) {
  return _then(_self.copyWith(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as NoteWindowStatus,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,locked: null == locked ? _self.locked : locked // ignore: cast_nullable_to_non_nullable
as bool,trashed: null == trashed ? _self.trashed : trashed // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [NoteWindowState].
extension NoteWindowStatePatterns on NoteWindowState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _NoteWindowState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _NoteWindowState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _NoteWindowState value)  $default,){
final _that = this;
switch (_that) {
case _NoteWindowState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _NoteWindowState value)?  $default,){
final _that = this;
switch (_that) {
case _NoteWindowState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String noteId,  NoteWindowStatus status,  String title,  bool locked,  bool trashed)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _NoteWindowState() when $default != null:
return $default(_that.noteId,_that.status,_that.title,_that.locked,_that.trashed);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String noteId,  NoteWindowStatus status,  String title,  bool locked,  bool trashed)  $default,) {final _that = this;
switch (_that) {
case _NoteWindowState():
return $default(_that.noteId,_that.status,_that.title,_that.locked,_that.trashed);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String noteId,  NoteWindowStatus status,  String title,  bool locked,  bool trashed)?  $default,) {final _that = this;
switch (_that) {
case _NoteWindowState() when $default != null:
return $default(_that.noteId,_that.status,_that.title,_that.locked,_that.trashed);case _:
  return null;

}
}

}

/// @nodoc


class _NoteWindowState implements NoteWindowState {
  const _NoteWindowState({this.noteId = '', this.status = NoteWindowStatus.loading, this.title = '', this.locked = false, this.trashed = false});
  

@override@JsonKey() final  String noteId;
@override@JsonKey() final  NoteWindowStatus status;
/// The note's title. Titles are not sealed, but a locked note's is kept
/// out of the window's name (see `noteWindowTitle`).
@override@JsonKey() final  String title;
/// Whether the note is locked, open or closed.
@override@JsonKey() final  bool locked;
/// Whether the note is in the trash.
@override@JsonKey() final  bool trashed;

/// Create a copy of NoteWindowState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$NoteWindowStateCopyWith<_NoteWindowState> get copyWith => __$NoteWindowStateCopyWithImpl<_NoteWindowState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _NoteWindowState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.status, status) || other.status == status)&&(identical(other.title, title) || other.title == title)&&(identical(other.locked, locked) || other.locked == locked)&&(identical(other.trashed, trashed) || other.trashed == trashed));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,status,title,locked,trashed);

@override
String toString() {
  return 'NoteWindowState(noteId: $noteId, status: $status, title: $title, locked: $locked, trashed: $trashed)';
}


}

/// @nodoc
abstract mixin class _$NoteWindowStateCopyWith<$Res> implements $NoteWindowStateCopyWith<$Res> {
  factory _$NoteWindowStateCopyWith(_NoteWindowState value, $Res Function(_NoteWindowState) _then) = __$NoteWindowStateCopyWithImpl;
@override @useResult
$Res call({
 String noteId, NoteWindowStatus status, String title, bool locked, bool trashed
});




}
/// @nodoc
class __$NoteWindowStateCopyWithImpl<$Res>
    implements _$NoteWindowStateCopyWith<$Res> {
  __$NoteWindowStateCopyWithImpl(this._self, this._then);

  final _NoteWindowState _self;
  final $Res Function(_NoteWindowState) _then;

/// Create a copy of NoteWindowState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? status = null,Object? title = null,Object? locked = null,Object? trashed = null,}) {
  return _then(_NoteWindowState(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as NoteWindowStatus,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,locked: null == locked ? _self.locked : locked // ignore: cast_nullable_to_non_nullable
as bool,trashed: null == trashed ? _self.trashed : trashed // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
