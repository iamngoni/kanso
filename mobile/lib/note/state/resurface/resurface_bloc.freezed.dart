// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'resurface_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ResurfaceEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ResurfaceEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ResurfaceEvent()';
}


}

/// @nodoc
class $ResurfaceEventCopyWith<$Res>  {
$ResurfaceEventCopyWith(ResurfaceEvent _, $Res Function(ResurfaceEvent) __);
}


/// Adds pattern-matching-related methods to [ResurfaceEvent].
extension ResurfaceEventPatterns on ResurfaceEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ResurfaceStartedEvent value)?  started,TResult Function( ResurfaceEditedEvent value)?  edited,TResult Function( ResurfaceDismissedEvent value)?  dismissed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ResurfaceStartedEvent() when started != null:
return started(_that);case ResurfaceEditedEvent() when edited != null:
return edited(_that);case ResurfaceDismissedEvent() when dismissed != null:
return dismissed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ResurfaceStartedEvent value)  started,required TResult Function( ResurfaceEditedEvent value)  edited,required TResult Function( ResurfaceDismissedEvent value)  dismissed,}){
final _that = this;
switch (_that) {
case ResurfaceStartedEvent():
return started(_that);case ResurfaceEditedEvent():
return edited(_that);case ResurfaceDismissedEvent():
return dismissed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ResurfaceStartedEvent value)?  started,TResult? Function( ResurfaceEditedEvent value)?  edited,TResult? Function( ResurfaceDismissedEvent value)?  dismissed,}){
final _that = this;
switch (_that) {
case ResurfaceStartedEvent() when started != null:
return started(_that);case ResurfaceEditedEvent() when edited != null:
return edited(_that);case ResurfaceDismissedEvent() when dismissed != null:
return dismissed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String noteId)?  started,TResult Function( String title,  String body,  int caret)?  edited,TResult Function( String otherId)?  dismissed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ResurfaceStartedEvent() when started != null:
return started(_that.noteId);case ResurfaceEditedEvent() when edited != null:
return edited(_that.title,_that.body,_that.caret);case ResurfaceDismissedEvent() when dismissed != null:
return dismissed(_that.otherId);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String noteId)  started,required TResult Function( String title,  String body,  int caret)  edited,required TResult Function( String otherId)  dismissed,}) {final _that = this;
switch (_that) {
case ResurfaceStartedEvent():
return started(_that.noteId);case ResurfaceEditedEvent():
return edited(_that.title,_that.body,_that.caret);case ResurfaceDismissedEvent():
return dismissed(_that.otherId);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String noteId)?  started,TResult? Function( String title,  String body,  int caret)?  edited,TResult? Function( String otherId)?  dismissed,}) {final _that = this;
switch (_that) {
case ResurfaceStartedEvent() when started != null:
return started(_that.noteId);case ResurfaceEditedEvent() when edited != null:
return edited(_that.title,_that.body,_that.caret);case ResurfaceDismissedEvent() when dismissed != null:
return dismissed(_that.otherId);case _:
  return null;

}
}

}

/// @nodoc


class ResurfaceStartedEvent implements ResurfaceEvent {
  const ResurfaceStartedEvent(this.noteId);
  

 final  String noteId;

/// Create a copy of ResurfaceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ResurfaceStartedEventCopyWith<ResurfaceStartedEvent> get copyWith => _$ResurfaceStartedEventCopyWithImpl<ResurfaceStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ResurfaceStartedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'ResurfaceEvent.started(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $ResurfaceStartedEventCopyWith<$Res> implements $ResurfaceEventCopyWith<$Res> {
  factory $ResurfaceStartedEventCopyWith(ResurfaceStartedEvent value, $Res Function(ResurfaceStartedEvent) _then) = _$ResurfaceStartedEventCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$ResurfaceStartedEventCopyWithImpl<$Res>
    implements $ResurfaceStartedEventCopyWith<$Res> {
  _$ResurfaceStartedEventCopyWithImpl(this._self, this._then);

  final ResurfaceStartedEvent _self;
  final $Res Function(ResurfaceStartedEvent) _then;

/// Create a copy of ResurfaceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(ResurfaceStartedEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ResurfaceEditedEvent implements ResurfaceEvent {
  const ResurfaceEditedEvent({required this.title, required this.body, required this.caret});
  

 final  String title;
 final  String body;
 final  int caret;

/// Create a copy of ResurfaceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ResurfaceEditedEventCopyWith<ResurfaceEditedEvent> get copyWith => _$ResurfaceEditedEventCopyWithImpl<ResurfaceEditedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ResurfaceEditedEvent&&(identical(other.title, title) || other.title == title)&&(identical(other.body, body) || other.body == body)&&(identical(other.caret, caret) || other.caret == caret));
}


@override
int get hashCode => Object.hash(runtimeType,title,body,caret);

@override
String toString() {
  return 'ResurfaceEvent.edited(title: $title, body: $body, caret: $caret)';
}


}

/// @nodoc
abstract mixin class $ResurfaceEditedEventCopyWith<$Res> implements $ResurfaceEventCopyWith<$Res> {
  factory $ResurfaceEditedEventCopyWith(ResurfaceEditedEvent value, $Res Function(ResurfaceEditedEvent) _then) = _$ResurfaceEditedEventCopyWithImpl;
@useResult
$Res call({
 String title, String body, int caret
});




}
/// @nodoc
class _$ResurfaceEditedEventCopyWithImpl<$Res>
    implements $ResurfaceEditedEventCopyWith<$Res> {
  _$ResurfaceEditedEventCopyWithImpl(this._self, this._then);

  final ResurfaceEditedEvent _self;
  final $Res Function(ResurfaceEditedEvent) _then;

/// Create a copy of ResurfaceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? title = null,Object? body = null,Object? caret = null,}) {
  return _then(ResurfaceEditedEvent(
title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,body: null == body ? _self.body : body // ignore: cast_nullable_to_non_nullable
as String,caret: null == caret ? _self.caret : caret // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class ResurfaceDismissedEvent implements ResurfaceEvent {
  const ResurfaceDismissedEvent(this.otherId);
  

 final  String otherId;

/// Create a copy of ResurfaceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ResurfaceDismissedEventCopyWith<ResurfaceDismissedEvent> get copyWith => _$ResurfaceDismissedEventCopyWithImpl<ResurfaceDismissedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ResurfaceDismissedEvent&&(identical(other.otherId, otherId) || other.otherId == otherId));
}


@override
int get hashCode => Object.hash(runtimeType,otherId);

@override
String toString() {
  return 'ResurfaceEvent.dismissed(otherId: $otherId)';
}


}

/// @nodoc
abstract mixin class $ResurfaceDismissedEventCopyWith<$Res> implements $ResurfaceEventCopyWith<$Res> {
  factory $ResurfaceDismissedEventCopyWith(ResurfaceDismissedEvent value, $Res Function(ResurfaceDismissedEvent) _then) = _$ResurfaceDismissedEventCopyWithImpl;
@useResult
$Res call({
 String otherId
});




}
/// @nodoc
class _$ResurfaceDismissedEventCopyWithImpl<$Res>
    implements $ResurfaceDismissedEventCopyWith<$Res> {
  _$ResurfaceDismissedEventCopyWithImpl(this._self, this._then);

  final ResurfaceDismissedEvent _self;
  final $Res Function(ResurfaceDismissedEvent) _then;

/// Create a copy of ResurfaceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? otherId = null,}) {
  return _then(ResurfaceDismissedEvent(
null == otherId ? _self.otherId : otherId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$ResurfaceState {

 String get noteId;/// Related notes are on and the setting says to offer notes.
 bool get enabled;/// "closer", "default" or "looser".
 String get closeness;/// Past notes about the paragraph, in the order they came.
 List<ResurfaceRow> get rows;/// Notes written on a daily note's date in earlier years.
 List<OnThisDayRow> get day;
/// Create a copy of ResurfaceState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ResurfaceStateCopyWith<ResurfaceState> get copyWith => _$ResurfaceStateCopyWithImpl<ResurfaceState>(this as ResurfaceState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ResurfaceState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.enabled, enabled) || other.enabled == enabled)&&(identical(other.closeness, closeness) || other.closeness == closeness)&&const DeepCollectionEquality().equals(other.rows, rows)&&const DeepCollectionEquality().equals(other.day, day));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,enabled,closeness,const DeepCollectionEquality().hash(rows),const DeepCollectionEquality().hash(day));

@override
String toString() {
  return 'ResurfaceState(noteId: $noteId, enabled: $enabled, closeness: $closeness, rows: $rows, day: $day)';
}


}

/// @nodoc
abstract mixin class $ResurfaceStateCopyWith<$Res>  {
  factory $ResurfaceStateCopyWith(ResurfaceState value, $Res Function(ResurfaceState) _then) = _$ResurfaceStateCopyWithImpl;
@useResult
$Res call({
 String noteId, bool enabled, String closeness, List<ResurfaceRow> rows, List<OnThisDayRow> day
});




}
/// @nodoc
class _$ResurfaceStateCopyWithImpl<$Res>
    implements $ResurfaceStateCopyWith<$Res> {
  _$ResurfaceStateCopyWithImpl(this._self, this._then);

  final ResurfaceState _self;
  final $Res Function(ResurfaceState) _then;

/// Create a copy of ResurfaceState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteId = null,Object? enabled = null,Object? closeness = null,Object? rows = null,Object? day = null,}) {
  return _then(_self.copyWith(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,closeness: null == closeness ? _self.closeness : closeness // ignore: cast_nullable_to_non_nullable
as String,rows: null == rows ? _self.rows : rows // ignore: cast_nullable_to_non_nullable
as List<ResurfaceRow>,day: null == day ? _self.day : day // ignore: cast_nullable_to_non_nullable
as List<OnThisDayRow>,
  ));
}

}


/// Adds pattern-matching-related methods to [ResurfaceState].
extension ResurfaceStatePatterns on ResurfaceState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ResurfaceState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ResurfaceState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ResurfaceState value)  $default,){
final _that = this;
switch (_that) {
case _ResurfaceState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ResurfaceState value)?  $default,){
final _that = this;
switch (_that) {
case _ResurfaceState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String noteId,  bool enabled,  String closeness,  List<ResurfaceRow> rows,  List<OnThisDayRow> day)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ResurfaceState() when $default != null:
return $default(_that.noteId,_that.enabled,_that.closeness,_that.rows,_that.day);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String noteId,  bool enabled,  String closeness,  List<ResurfaceRow> rows,  List<OnThisDayRow> day)  $default,) {final _that = this;
switch (_that) {
case _ResurfaceState():
return $default(_that.noteId,_that.enabled,_that.closeness,_that.rows,_that.day);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String noteId,  bool enabled,  String closeness,  List<ResurfaceRow> rows,  List<OnThisDayRow> day)?  $default,) {final _that = this;
switch (_that) {
case _ResurfaceState() when $default != null:
return $default(_that.noteId,_that.enabled,_that.closeness,_that.rows,_that.day);case _:
  return null;

}
}

}

/// @nodoc


class _ResurfaceState implements ResurfaceState {
  const _ResurfaceState({this.noteId = '', this.enabled = false, this.closeness = 'default', final  List<ResurfaceRow> rows = const <ResurfaceRow>[], final  List<OnThisDayRow> day = const <OnThisDayRow>[]}): _rows = rows,_day = day;
  

@override@JsonKey() final  String noteId;
/// Related notes are on and the setting says to offer notes.
@override@JsonKey() final  bool enabled;
/// "closer", "default" or "looser".
@override@JsonKey() final  String closeness;
/// Past notes about the paragraph, in the order they came.
 final  List<ResurfaceRow> _rows;
/// Past notes about the paragraph, in the order they came.
@override@JsonKey() List<ResurfaceRow> get rows {
  if (_rows is EqualUnmodifiableListView) return _rows;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_rows);
}

/// Notes written on a daily note's date in earlier years.
 final  List<OnThisDayRow> _day;
/// Notes written on a daily note's date in earlier years.
@override@JsonKey() List<OnThisDayRow> get day {
  if (_day is EqualUnmodifiableListView) return _day;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_day);
}


/// Create a copy of ResurfaceState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ResurfaceStateCopyWith<_ResurfaceState> get copyWith => __$ResurfaceStateCopyWithImpl<_ResurfaceState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ResurfaceState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.enabled, enabled) || other.enabled == enabled)&&(identical(other.closeness, closeness) || other.closeness == closeness)&&const DeepCollectionEquality().equals(other._rows, _rows)&&const DeepCollectionEquality().equals(other._day, _day));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,enabled,closeness,const DeepCollectionEquality().hash(_rows),const DeepCollectionEquality().hash(_day));

@override
String toString() {
  return 'ResurfaceState(noteId: $noteId, enabled: $enabled, closeness: $closeness, rows: $rows, day: $day)';
}


}

/// @nodoc
abstract mixin class _$ResurfaceStateCopyWith<$Res> implements $ResurfaceStateCopyWith<$Res> {
  factory _$ResurfaceStateCopyWith(_ResurfaceState value, $Res Function(_ResurfaceState) _then) = __$ResurfaceStateCopyWithImpl;
@override @useResult
$Res call({
 String noteId, bool enabled, String closeness, List<ResurfaceRow> rows, List<OnThisDayRow> day
});




}
/// @nodoc
class __$ResurfaceStateCopyWithImpl<$Res>
    implements _$ResurfaceStateCopyWith<$Res> {
  __$ResurfaceStateCopyWithImpl(this._self, this._then);

  final _ResurfaceState _self;
  final $Res Function(_ResurfaceState) _then;

/// Create a copy of ResurfaceState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? enabled = null,Object? closeness = null,Object? rows = null,Object? day = null,}) {
  return _then(_ResurfaceState(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,closeness: null == closeness ? _self.closeness : closeness // ignore: cast_nullable_to_non_nullable
as String,rows: null == rows ? _self._rows : rows // ignore: cast_nullable_to_non_nullable
as List<ResurfaceRow>,day: null == day ? _self._day : day // ignore: cast_nullable_to_non_nullable
as List<OnThisDayRow>,
  ));
}


}

// dart format on
