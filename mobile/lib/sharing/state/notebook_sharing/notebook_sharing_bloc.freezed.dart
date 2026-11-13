// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'notebook_sharing_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$NotebookSharingEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotebookSharingEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NotebookSharingEvent()';
}


}

/// @nodoc
class $NotebookSharingEventCopyWith<$Res>  {
$NotebookSharingEventCopyWith(NotebookSharingEvent _, $Res Function(NotebookSharingEvent) __);
}


/// Adds pattern-matching-related methods to [NotebookSharingEvent].
extension NotebookSharingEventPatterns on NotebookSharingEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( NotebookSharingStartedEvent value)?  started,TResult Function( NotebookSharingRefreshedEvent value)?  refreshed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case NotebookSharingStartedEvent() when started != null:
return started(_that);case NotebookSharingRefreshedEvent() when refreshed != null:
return refreshed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( NotebookSharingStartedEvent value)  started,required TResult Function( NotebookSharingRefreshedEvent value)  refreshed,}){
final _that = this;
switch (_that) {
case NotebookSharingStartedEvent():
return started(_that);case NotebookSharingRefreshedEvent():
return refreshed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( NotebookSharingStartedEvent value)?  started,TResult? Function( NotebookSharingRefreshedEvent value)?  refreshed,}){
final _that = this;
switch (_that) {
case NotebookSharingStartedEvent() when started != null:
return started(_that);case NotebookSharingRefreshedEvent() when refreshed != null:
return refreshed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String notebookId)?  started,TResult Function()?  refreshed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case NotebookSharingStartedEvent() when started != null:
return started(_that.notebookId);case NotebookSharingRefreshedEvent() when refreshed != null:
return refreshed();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String notebookId)  started,required TResult Function()  refreshed,}) {final _that = this;
switch (_that) {
case NotebookSharingStartedEvent():
return started(_that.notebookId);case NotebookSharingRefreshedEvent():
return refreshed();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String notebookId)?  started,TResult? Function()?  refreshed,}) {final _that = this;
switch (_that) {
case NotebookSharingStartedEvent() when started != null:
return started(_that.notebookId);case NotebookSharingRefreshedEvent() when refreshed != null:
return refreshed();case _:
  return null;

}
}

}

/// @nodoc


class NotebookSharingStartedEvent implements NotebookSharingEvent {
  const NotebookSharingStartedEvent(this.notebookId);
  

 final  String notebookId;

/// Create a copy of NotebookSharingEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NotebookSharingStartedEventCopyWith<NotebookSharingStartedEvent> get copyWith => _$NotebookSharingStartedEventCopyWithImpl<NotebookSharingStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotebookSharingStartedEvent&&(identical(other.notebookId, notebookId) || other.notebookId == notebookId));
}


@override
int get hashCode => Object.hash(runtimeType,notebookId);

@override
String toString() {
  return 'NotebookSharingEvent.started(notebookId: $notebookId)';
}


}

/// @nodoc
abstract mixin class $NotebookSharingStartedEventCopyWith<$Res> implements $NotebookSharingEventCopyWith<$Res> {
  factory $NotebookSharingStartedEventCopyWith(NotebookSharingStartedEvent value, $Res Function(NotebookSharingStartedEvent) _then) = _$NotebookSharingStartedEventCopyWithImpl;
@useResult
$Res call({
 String notebookId
});




}
/// @nodoc
class _$NotebookSharingStartedEventCopyWithImpl<$Res>
    implements $NotebookSharingStartedEventCopyWith<$Res> {
  _$NotebookSharingStartedEventCopyWithImpl(this._self, this._then);

  final NotebookSharingStartedEvent _self;
  final $Res Function(NotebookSharingStartedEvent) _then;

/// Create a copy of NotebookSharingEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? notebookId = null,}) {
  return _then(NotebookSharingStartedEvent(
null == notebookId ? _self.notebookId : notebookId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NotebookSharingRefreshedEvent implements NotebookSharingEvent {
  const NotebookSharingRefreshedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotebookSharingRefreshedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NotebookSharingEvent.refreshed()';
}


}




/// @nodoc
mixin _$NotebookSharingState {

 String get notebookId;/// Null while the notebook isn't shared (or isn't read yet).
 BookSyncStatus? get status;/// Who last edited each note, by note ID.
 Map<String, NoteEditRow> get edits;
/// Create a copy of NotebookSharingState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NotebookSharingStateCopyWith<NotebookSharingState> get copyWith => _$NotebookSharingStateCopyWithImpl<NotebookSharingState>(this as NotebookSharingState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotebookSharingState&&(identical(other.notebookId, notebookId) || other.notebookId == notebookId)&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other.edits, edits));
}


@override
int get hashCode => Object.hash(runtimeType,notebookId,status,const DeepCollectionEquality().hash(edits));

@override
String toString() {
  return 'NotebookSharingState(notebookId: $notebookId, status: $status, edits: $edits)';
}


}

/// @nodoc
abstract mixin class $NotebookSharingStateCopyWith<$Res>  {
  factory $NotebookSharingStateCopyWith(NotebookSharingState value, $Res Function(NotebookSharingState) _then) = _$NotebookSharingStateCopyWithImpl;
@useResult
$Res call({
 String notebookId, BookSyncStatus? status, Map<String, NoteEditRow> edits
});




}
/// @nodoc
class _$NotebookSharingStateCopyWithImpl<$Res>
    implements $NotebookSharingStateCopyWith<$Res> {
  _$NotebookSharingStateCopyWithImpl(this._self, this._then);

  final NotebookSharingState _self;
  final $Res Function(NotebookSharingState) _then;

/// Create a copy of NotebookSharingState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? notebookId = null,Object? status = freezed,Object? edits = null,}) {
  return _then(_self.copyWith(
notebookId: null == notebookId ? _self.notebookId : notebookId // ignore: cast_nullable_to_non_nullable
as String,status: freezed == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as BookSyncStatus?,edits: null == edits ? _self.edits : edits // ignore: cast_nullable_to_non_nullable
as Map<String, NoteEditRow>,
  ));
}

}


/// Adds pattern-matching-related methods to [NotebookSharingState].
extension NotebookSharingStatePatterns on NotebookSharingState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _NotebookSharingState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _NotebookSharingState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _NotebookSharingState value)  $default,){
final _that = this;
switch (_that) {
case _NotebookSharingState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _NotebookSharingState value)?  $default,){
final _that = this;
switch (_that) {
case _NotebookSharingState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String notebookId,  BookSyncStatus? status,  Map<String, NoteEditRow> edits)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _NotebookSharingState() when $default != null:
return $default(_that.notebookId,_that.status,_that.edits);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String notebookId,  BookSyncStatus? status,  Map<String, NoteEditRow> edits)  $default,) {final _that = this;
switch (_that) {
case _NotebookSharingState():
return $default(_that.notebookId,_that.status,_that.edits);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String notebookId,  BookSyncStatus? status,  Map<String, NoteEditRow> edits)?  $default,) {final _that = this;
switch (_that) {
case _NotebookSharingState() when $default != null:
return $default(_that.notebookId,_that.status,_that.edits);case _:
  return null;

}
}

}

/// @nodoc


class _NotebookSharingState implements NotebookSharingState {
  const _NotebookSharingState({this.notebookId = '', this.status, final  Map<String, NoteEditRow> edits = const <String, NoteEditRow>{}}): _edits = edits;
  

@override@JsonKey() final  String notebookId;
/// Null while the notebook isn't shared (or isn't read yet).
@override final  BookSyncStatus? status;
/// Who last edited each note, by note ID.
 final  Map<String, NoteEditRow> _edits;
/// Who last edited each note, by note ID.
@override@JsonKey() Map<String, NoteEditRow> get edits {
  if (_edits is EqualUnmodifiableMapView) return _edits;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableMapView(_edits);
}


/// Create a copy of NotebookSharingState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$NotebookSharingStateCopyWith<_NotebookSharingState> get copyWith => __$NotebookSharingStateCopyWithImpl<_NotebookSharingState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _NotebookSharingState&&(identical(other.notebookId, notebookId) || other.notebookId == notebookId)&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other._edits, _edits));
}


@override
int get hashCode => Object.hash(runtimeType,notebookId,status,const DeepCollectionEquality().hash(_edits));

@override
String toString() {
  return 'NotebookSharingState(notebookId: $notebookId, status: $status, edits: $edits)';
}


}

/// @nodoc
abstract mixin class _$NotebookSharingStateCopyWith<$Res> implements $NotebookSharingStateCopyWith<$Res> {
  factory _$NotebookSharingStateCopyWith(_NotebookSharingState value, $Res Function(_NotebookSharingState) _then) = __$NotebookSharingStateCopyWithImpl;
@override @useResult
$Res call({
 String notebookId, BookSyncStatus? status, Map<String, NoteEditRow> edits
});




}
/// @nodoc
class __$NotebookSharingStateCopyWithImpl<$Res>
    implements _$NotebookSharingStateCopyWith<$Res> {
  __$NotebookSharingStateCopyWithImpl(this._self, this._then);

  final _NotebookSharingState _self;
  final $Res Function(_NotebookSharingState) _then;

/// Create a copy of NotebookSharingState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? notebookId = null,Object? status = freezed,Object? edits = null,}) {
  return _then(_NotebookSharingState(
notebookId: null == notebookId ? _self.notebookId : notebookId // ignore: cast_nullable_to_non_nullable
as String,status: freezed == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as BookSyncStatus?,edits: null == edits ? _self._edits : edits // ignore: cast_nullable_to_non_nullable
as Map<String, NoteEditRow>,
  ));
}


}

// dart format on
