// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'meeting_note_event.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$MeetingNoteEvent {

 OverviewAgendaRow get row;
/// Create a copy of MeetingNoteEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$MeetingNoteEventCopyWith<MeetingNoteEvent> get copyWith => _$MeetingNoteEventCopyWithImpl<MeetingNoteEvent>(this as MeetingNoteEvent, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MeetingNoteEvent&&(identical(other.row, row) || other.row == row));
}


@override
int get hashCode => Object.hash(runtimeType,row);

@override
String toString() {
  return 'MeetingNoteEvent(row: $row)';
}


}

/// @nodoc
abstract mixin class $MeetingNoteEventCopyWith<$Res>  {
  factory $MeetingNoteEventCopyWith(MeetingNoteEvent value, $Res Function(MeetingNoteEvent) _then) = _$MeetingNoteEventCopyWithImpl;
@useResult
$Res call({
 OverviewAgendaRow row
});




}
/// @nodoc
class _$MeetingNoteEventCopyWithImpl<$Res>
    implements $MeetingNoteEventCopyWith<$Res> {
  _$MeetingNoteEventCopyWithImpl(this._self, this._then);

  final MeetingNoteEvent _self;
  final $Res Function(MeetingNoteEvent) _then;

/// Create a copy of MeetingNoteEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? row = null,}) {
  return _then(_self.copyWith(
row: null == row ? _self.row : row // ignore: cast_nullable_to_non_nullable
as OverviewAgendaRow,
  ));
}

}


/// Adds pattern-matching-related methods to [MeetingNoteEvent].
extension MeetingNoteEventPatterns on MeetingNoteEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( MeetingNoteCreateRequested value)?  create,required TResult orElse(),}){
final _that = this;
switch (_that) {
case MeetingNoteCreateRequested() when create != null:
return create(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( MeetingNoteCreateRequested value)  create,}){
final _that = this;
switch (_that) {
case MeetingNoteCreateRequested():
return create(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( MeetingNoteCreateRequested value)?  create,}){
final _that = this;
switch (_that) {
case MeetingNoteCreateRequested() when create != null:
return create(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( OverviewAgendaRow row)?  create,required TResult orElse(),}) {final _that = this;
switch (_that) {
case MeetingNoteCreateRequested() when create != null:
return create(_that.row);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( OverviewAgendaRow row)  create,}) {final _that = this;
switch (_that) {
case MeetingNoteCreateRequested():
return create(_that.row);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( OverviewAgendaRow row)?  create,}) {final _that = this;
switch (_that) {
case MeetingNoteCreateRequested() when create != null:
return create(_that.row);case _:
  return null;

}
}

}

/// @nodoc


class MeetingNoteCreateRequested implements MeetingNoteEvent {
  const MeetingNoteCreateRequested({required this.row});
  

@override final  OverviewAgendaRow row;

/// Create a copy of MeetingNoteEvent
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$MeetingNoteCreateRequestedCopyWith<MeetingNoteCreateRequested> get copyWith => _$MeetingNoteCreateRequestedCopyWithImpl<MeetingNoteCreateRequested>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MeetingNoteCreateRequested&&(identical(other.row, row) || other.row == row));
}


@override
int get hashCode => Object.hash(runtimeType,row);

@override
String toString() {
  return 'MeetingNoteEvent.create(row: $row)';
}


}

/// @nodoc
abstract mixin class $MeetingNoteCreateRequestedCopyWith<$Res> implements $MeetingNoteEventCopyWith<$Res> {
  factory $MeetingNoteCreateRequestedCopyWith(MeetingNoteCreateRequested value, $Res Function(MeetingNoteCreateRequested) _then) = _$MeetingNoteCreateRequestedCopyWithImpl;
@override @useResult
$Res call({
 OverviewAgendaRow row
});




}
/// @nodoc
class _$MeetingNoteCreateRequestedCopyWithImpl<$Res>
    implements $MeetingNoteCreateRequestedCopyWith<$Res> {
  _$MeetingNoteCreateRequestedCopyWithImpl(this._self, this._then);

  final MeetingNoteCreateRequested _self;
  final $Res Function(MeetingNoteCreateRequested) _then;

/// Create a copy of MeetingNoteEvent
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? row = null,}) {
  return _then(MeetingNoteCreateRequested(
row: null == row ? _self.row : row // ignore: cast_nullable_to_non_nullable
as OverviewAgendaRow,
  ));
}


}

// dart format on
