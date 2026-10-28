// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'note_discovery_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$NoteDiscoveryEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteDiscoveryEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteDiscoveryEvent()';
}


}

/// @nodoc
class $NoteDiscoveryEventCopyWith<$Res>  {
$NoteDiscoveryEventCopyWith(NoteDiscoveryEvent _, $Res Function(NoteDiscoveryEvent) __);
}


/// Adds pattern-matching-related methods to [NoteDiscoveryEvent].
extension NoteDiscoveryEventPatterns on NoteDiscoveryEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( NoteDiscoveryStartedEvent value)?  started,TResult Function( NoteDiscoveryTagAddedEvent value)?  tagAdded,TResult Function( NoteDiscoveryTwinDismissedEvent value)?  twinDismissed,TResult Function( NoteDiscoveryNotesMergedEvent value)?  notesMerged,required TResult orElse(),}){
final _that = this;
switch (_that) {
case NoteDiscoveryStartedEvent() when started != null:
return started(_that);case NoteDiscoveryTagAddedEvent() when tagAdded != null:
return tagAdded(_that);case NoteDiscoveryTwinDismissedEvent() when twinDismissed != null:
return twinDismissed(_that);case NoteDiscoveryNotesMergedEvent() when notesMerged != null:
return notesMerged(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( NoteDiscoveryStartedEvent value)  started,required TResult Function( NoteDiscoveryTagAddedEvent value)  tagAdded,required TResult Function( NoteDiscoveryTwinDismissedEvent value)  twinDismissed,required TResult Function( NoteDiscoveryNotesMergedEvent value)  notesMerged,}){
final _that = this;
switch (_that) {
case NoteDiscoveryStartedEvent():
return started(_that);case NoteDiscoveryTagAddedEvent():
return tagAdded(_that);case NoteDiscoveryTwinDismissedEvent():
return twinDismissed(_that);case NoteDiscoveryNotesMergedEvent():
return notesMerged(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( NoteDiscoveryStartedEvent value)?  started,TResult? Function( NoteDiscoveryTagAddedEvent value)?  tagAdded,TResult? Function( NoteDiscoveryTwinDismissedEvent value)?  twinDismissed,TResult? Function( NoteDiscoveryNotesMergedEvent value)?  notesMerged,}){
final _that = this;
switch (_that) {
case NoteDiscoveryStartedEvent() when started != null:
return started(_that);case NoteDiscoveryTagAddedEvent() when tagAdded != null:
return tagAdded(_that);case NoteDiscoveryTwinDismissedEvent() when twinDismissed != null:
return twinDismissed(_that);case NoteDiscoveryNotesMergedEvent() when notesMerged != null:
return notesMerged(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String noteId)?  started,TResult Function( String tagId)?  tagAdded,TResult Function( String otherId)?  twinDismissed,TResult Function( String otherId)?  notesMerged,required TResult orElse(),}) {final _that = this;
switch (_that) {
case NoteDiscoveryStartedEvent() when started != null:
return started(_that.noteId);case NoteDiscoveryTagAddedEvent() when tagAdded != null:
return tagAdded(_that.tagId);case NoteDiscoveryTwinDismissedEvent() when twinDismissed != null:
return twinDismissed(_that.otherId);case NoteDiscoveryNotesMergedEvent() when notesMerged != null:
return notesMerged(_that.otherId);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String noteId)  started,required TResult Function( String tagId)  tagAdded,required TResult Function( String otherId)  twinDismissed,required TResult Function( String otherId)  notesMerged,}) {final _that = this;
switch (_that) {
case NoteDiscoveryStartedEvent():
return started(_that.noteId);case NoteDiscoveryTagAddedEvent():
return tagAdded(_that.tagId);case NoteDiscoveryTwinDismissedEvent():
return twinDismissed(_that.otherId);case NoteDiscoveryNotesMergedEvent():
return notesMerged(_that.otherId);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String noteId)?  started,TResult? Function( String tagId)?  tagAdded,TResult? Function( String otherId)?  twinDismissed,TResult? Function( String otherId)?  notesMerged,}) {final _that = this;
switch (_that) {
case NoteDiscoveryStartedEvent() when started != null:
return started(_that.noteId);case NoteDiscoveryTagAddedEvent() when tagAdded != null:
return tagAdded(_that.tagId);case NoteDiscoveryTwinDismissedEvent() when twinDismissed != null:
return twinDismissed(_that.otherId);case NoteDiscoveryNotesMergedEvent() when notesMerged != null:
return notesMerged(_that.otherId);case _:
  return null;

}
}

}

/// @nodoc


class NoteDiscoveryStartedEvent implements NoteDiscoveryEvent {
  const NoteDiscoveryStartedEvent(this.noteId);
  

 final  String noteId;

/// Create a copy of NoteDiscoveryEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteDiscoveryStartedEventCopyWith<NoteDiscoveryStartedEvent> get copyWith => _$NoteDiscoveryStartedEventCopyWithImpl<NoteDiscoveryStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteDiscoveryStartedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'NoteDiscoveryEvent.started(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $NoteDiscoveryStartedEventCopyWith<$Res> implements $NoteDiscoveryEventCopyWith<$Res> {
  factory $NoteDiscoveryStartedEventCopyWith(NoteDiscoveryStartedEvent value, $Res Function(NoteDiscoveryStartedEvent) _then) = _$NoteDiscoveryStartedEventCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$NoteDiscoveryStartedEventCopyWithImpl<$Res>
    implements $NoteDiscoveryStartedEventCopyWith<$Res> {
  _$NoteDiscoveryStartedEventCopyWithImpl(this._self, this._then);

  final NoteDiscoveryStartedEvent _self;
  final $Res Function(NoteDiscoveryStartedEvent) _then;

/// Create a copy of NoteDiscoveryEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(NoteDiscoveryStartedEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NoteDiscoveryTagAddedEvent implements NoteDiscoveryEvent {
  const NoteDiscoveryTagAddedEvent(this.tagId);
  

 final  String tagId;

/// Create a copy of NoteDiscoveryEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteDiscoveryTagAddedEventCopyWith<NoteDiscoveryTagAddedEvent> get copyWith => _$NoteDiscoveryTagAddedEventCopyWithImpl<NoteDiscoveryTagAddedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteDiscoveryTagAddedEvent&&(identical(other.tagId, tagId) || other.tagId == tagId));
}


@override
int get hashCode => Object.hash(runtimeType,tagId);

@override
String toString() {
  return 'NoteDiscoveryEvent.tagAdded(tagId: $tagId)';
}


}

/// @nodoc
abstract mixin class $NoteDiscoveryTagAddedEventCopyWith<$Res> implements $NoteDiscoveryEventCopyWith<$Res> {
  factory $NoteDiscoveryTagAddedEventCopyWith(NoteDiscoveryTagAddedEvent value, $Res Function(NoteDiscoveryTagAddedEvent) _then) = _$NoteDiscoveryTagAddedEventCopyWithImpl;
@useResult
$Res call({
 String tagId
});




}
/// @nodoc
class _$NoteDiscoveryTagAddedEventCopyWithImpl<$Res>
    implements $NoteDiscoveryTagAddedEventCopyWith<$Res> {
  _$NoteDiscoveryTagAddedEventCopyWithImpl(this._self, this._then);

  final NoteDiscoveryTagAddedEvent _self;
  final $Res Function(NoteDiscoveryTagAddedEvent) _then;

/// Create a copy of NoteDiscoveryEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? tagId = null,}) {
  return _then(NoteDiscoveryTagAddedEvent(
null == tagId ? _self.tagId : tagId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NoteDiscoveryTwinDismissedEvent implements NoteDiscoveryEvent {
  const NoteDiscoveryTwinDismissedEvent(this.otherId);
  

 final  String otherId;

/// Create a copy of NoteDiscoveryEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteDiscoveryTwinDismissedEventCopyWith<NoteDiscoveryTwinDismissedEvent> get copyWith => _$NoteDiscoveryTwinDismissedEventCopyWithImpl<NoteDiscoveryTwinDismissedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteDiscoveryTwinDismissedEvent&&(identical(other.otherId, otherId) || other.otherId == otherId));
}


@override
int get hashCode => Object.hash(runtimeType,otherId);

@override
String toString() {
  return 'NoteDiscoveryEvent.twinDismissed(otherId: $otherId)';
}


}

/// @nodoc
abstract mixin class $NoteDiscoveryTwinDismissedEventCopyWith<$Res> implements $NoteDiscoveryEventCopyWith<$Res> {
  factory $NoteDiscoveryTwinDismissedEventCopyWith(NoteDiscoveryTwinDismissedEvent value, $Res Function(NoteDiscoveryTwinDismissedEvent) _then) = _$NoteDiscoveryTwinDismissedEventCopyWithImpl;
@useResult
$Res call({
 String otherId
});




}
/// @nodoc
class _$NoteDiscoveryTwinDismissedEventCopyWithImpl<$Res>
    implements $NoteDiscoveryTwinDismissedEventCopyWith<$Res> {
  _$NoteDiscoveryTwinDismissedEventCopyWithImpl(this._self, this._then);

  final NoteDiscoveryTwinDismissedEvent _self;
  final $Res Function(NoteDiscoveryTwinDismissedEvent) _then;

/// Create a copy of NoteDiscoveryEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? otherId = null,}) {
  return _then(NoteDiscoveryTwinDismissedEvent(
null == otherId ? _self.otherId : otherId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NoteDiscoveryNotesMergedEvent implements NoteDiscoveryEvent {
  const NoteDiscoveryNotesMergedEvent(this.otherId);
  

 final  String otherId;

/// Create a copy of NoteDiscoveryEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteDiscoveryNotesMergedEventCopyWith<NoteDiscoveryNotesMergedEvent> get copyWith => _$NoteDiscoveryNotesMergedEventCopyWithImpl<NoteDiscoveryNotesMergedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteDiscoveryNotesMergedEvent&&(identical(other.otherId, otherId) || other.otherId == otherId));
}


@override
int get hashCode => Object.hash(runtimeType,otherId);

@override
String toString() {
  return 'NoteDiscoveryEvent.notesMerged(otherId: $otherId)';
}


}

/// @nodoc
abstract mixin class $NoteDiscoveryNotesMergedEventCopyWith<$Res> implements $NoteDiscoveryEventCopyWith<$Res> {
  factory $NoteDiscoveryNotesMergedEventCopyWith(NoteDiscoveryNotesMergedEvent value, $Res Function(NoteDiscoveryNotesMergedEvent) _then) = _$NoteDiscoveryNotesMergedEventCopyWithImpl;
@useResult
$Res call({
 String otherId
});




}
/// @nodoc
class _$NoteDiscoveryNotesMergedEventCopyWithImpl<$Res>
    implements $NoteDiscoveryNotesMergedEventCopyWith<$Res> {
  _$NoteDiscoveryNotesMergedEventCopyWithImpl(this._self, this._then);

  final NoteDiscoveryNotesMergedEvent _self;
  final $Res Function(NoteDiscoveryNotesMergedEvent) _then;

/// Create a copy of NoteDiscoveryEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? otherId = null,}) {
  return _then(NoteDiscoveryNotesMergedEvent(
null == otherId ? _self.otherId : otherId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$NoteDiscoveryState {

 String get noteId; bool get loading; bool get relatedEnabled; List<RelatedRow> get related; NoteMeaning? get meaning;
/// Create a copy of NoteDiscoveryState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteDiscoveryStateCopyWith<NoteDiscoveryState> get copyWith => _$NoteDiscoveryStateCopyWithImpl<NoteDiscoveryState>(this as NoteDiscoveryState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteDiscoveryState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.loading, loading) || other.loading == loading)&&(identical(other.relatedEnabled, relatedEnabled) || other.relatedEnabled == relatedEnabled)&&const DeepCollectionEquality().equals(other.related, related)&&(identical(other.meaning, meaning) || other.meaning == meaning));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,loading,relatedEnabled,const DeepCollectionEquality().hash(related),meaning);

@override
String toString() {
  return 'NoteDiscoveryState(noteId: $noteId, loading: $loading, relatedEnabled: $relatedEnabled, related: $related, meaning: $meaning)';
}


}

/// @nodoc
abstract mixin class $NoteDiscoveryStateCopyWith<$Res>  {
  factory $NoteDiscoveryStateCopyWith(NoteDiscoveryState value, $Res Function(NoteDiscoveryState) _then) = _$NoteDiscoveryStateCopyWithImpl;
@useResult
$Res call({
 String noteId, bool loading, bool relatedEnabled, List<RelatedRow> related, NoteMeaning? meaning
});




}
/// @nodoc
class _$NoteDiscoveryStateCopyWithImpl<$Res>
    implements $NoteDiscoveryStateCopyWith<$Res> {
  _$NoteDiscoveryStateCopyWithImpl(this._self, this._then);

  final NoteDiscoveryState _self;
  final $Res Function(NoteDiscoveryState) _then;

/// Create a copy of NoteDiscoveryState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteId = null,Object? loading = null,Object? relatedEnabled = null,Object? related = null,Object? meaning = freezed,}) {
  return _then(_self.copyWith(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,loading: null == loading ? _self.loading : loading // ignore: cast_nullable_to_non_nullable
as bool,relatedEnabled: null == relatedEnabled ? _self.relatedEnabled : relatedEnabled // ignore: cast_nullable_to_non_nullable
as bool,related: null == related ? _self.related : related // ignore: cast_nullable_to_non_nullable
as List<RelatedRow>,meaning: freezed == meaning ? _self.meaning : meaning // ignore: cast_nullable_to_non_nullable
as NoteMeaning?,
  ));
}

}


/// Adds pattern-matching-related methods to [NoteDiscoveryState].
extension NoteDiscoveryStatePatterns on NoteDiscoveryState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _NoteDiscoveryState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _NoteDiscoveryState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _NoteDiscoveryState value)  $default,){
final _that = this;
switch (_that) {
case _NoteDiscoveryState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _NoteDiscoveryState value)?  $default,){
final _that = this;
switch (_that) {
case _NoteDiscoveryState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String noteId,  bool loading,  bool relatedEnabled,  List<RelatedRow> related,  NoteMeaning? meaning)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _NoteDiscoveryState() when $default != null:
return $default(_that.noteId,_that.loading,_that.relatedEnabled,_that.related,_that.meaning);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String noteId,  bool loading,  bool relatedEnabled,  List<RelatedRow> related,  NoteMeaning? meaning)  $default,) {final _that = this;
switch (_that) {
case _NoteDiscoveryState():
return $default(_that.noteId,_that.loading,_that.relatedEnabled,_that.related,_that.meaning);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String noteId,  bool loading,  bool relatedEnabled,  List<RelatedRow> related,  NoteMeaning? meaning)?  $default,) {final _that = this;
switch (_that) {
case _NoteDiscoveryState() when $default != null:
return $default(_that.noteId,_that.loading,_that.relatedEnabled,_that.related,_that.meaning);case _:
  return null;

}
}

}

/// @nodoc


class _NoteDiscoveryState implements NoteDiscoveryState {
  const _NoteDiscoveryState({this.noteId = '', this.loading = false, this.relatedEnabled = false, final  List<RelatedRow> related = const <RelatedRow>[], this.meaning}): _related = related;
  

@override@JsonKey() final  String noteId;
@override@JsonKey() final  bool loading;
@override@JsonKey() final  bool relatedEnabled;
 final  List<RelatedRow> _related;
@override@JsonKey() List<RelatedRow> get related {
  if (_related is EqualUnmodifiableListView) return _related;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_related);
}

@override final  NoteMeaning? meaning;

/// Create a copy of NoteDiscoveryState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$NoteDiscoveryStateCopyWith<_NoteDiscoveryState> get copyWith => __$NoteDiscoveryStateCopyWithImpl<_NoteDiscoveryState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _NoteDiscoveryState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.loading, loading) || other.loading == loading)&&(identical(other.relatedEnabled, relatedEnabled) || other.relatedEnabled == relatedEnabled)&&const DeepCollectionEquality().equals(other._related, _related)&&(identical(other.meaning, meaning) || other.meaning == meaning));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,loading,relatedEnabled,const DeepCollectionEquality().hash(_related),meaning);

@override
String toString() {
  return 'NoteDiscoveryState(noteId: $noteId, loading: $loading, relatedEnabled: $relatedEnabled, related: $related, meaning: $meaning)';
}


}

/// @nodoc
abstract mixin class _$NoteDiscoveryStateCopyWith<$Res> implements $NoteDiscoveryStateCopyWith<$Res> {
  factory _$NoteDiscoveryStateCopyWith(_NoteDiscoveryState value, $Res Function(_NoteDiscoveryState) _then) = __$NoteDiscoveryStateCopyWithImpl;
@override @useResult
$Res call({
 String noteId, bool loading, bool relatedEnabled, List<RelatedRow> related, NoteMeaning? meaning
});




}
/// @nodoc
class __$NoteDiscoveryStateCopyWithImpl<$Res>
    implements _$NoteDiscoveryStateCopyWith<$Res> {
  __$NoteDiscoveryStateCopyWithImpl(this._self, this._then);

  final _NoteDiscoveryState _self;
  final $Res Function(_NoteDiscoveryState) _then;

/// Create a copy of NoteDiscoveryState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? loading = null,Object? relatedEnabled = null,Object? related = null,Object? meaning = freezed,}) {
  return _then(_NoteDiscoveryState(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,loading: null == loading ? _self.loading : loading // ignore: cast_nullable_to_non_nullable
as bool,relatedEnabled: null == relatedEnabled ? _self.relatedEnabled : relatedEnabled // ignore: cast_nullable_to_non_nullable
as bool,related: null == related ? _self._related : related // ignore: cast_nullable_to_non_nullable
as List<RelatedRow>,meaning: freezed == meaning ? _self.meaning : meaning // ignore: cast_nullable_to_non_nullable
as NoteMeaning?,
  ));
}


}

// dart format on
