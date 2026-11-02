// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'review_prompt_record.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$ReviewPromptRecord {

/// When the app was first opened with this record (the install, or the
/// update that brought it). Null until then.
 DateTime? get firstOpenedAt;/// On how many different days the app was opened.
 int get activeDays;/// The last day the app was opened, `YYYY-MM-DD` in local time.
 String? get lastActiveDay;/// How many edit sessions: times a note was let go after it was written
/// in (the same note twice counts twice).
 int get notesEdited;/// When the app last crashed, or sync or signing in last failed.
 DateTime? get lastTroubleAt;/// When the app last asked the system for a review.
 DateTime? get lastAskedAt;/// The versions of the app that have asked, oldest first; one is never
/// asked from twice.
 List<String> get askedVersions;/// Whether a sync has completed on this phone.
 bool get firstSyncSeen;
/// Create a copy of ReviewPromptRecord
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ReviewPromptRecordCopyWith<ReviewPromptRecord> get copyWith => _$ReviewPromptRecordCopyWithImpl<ReviewPromptRecord>(this as ReviewPromptRecord, _$identity);

  /// Serializes this ReviewPromptRecord to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ReviewPromptRecord&&(identical(other.firstOpenedAt, firstOpenedAt) || other.firstOpenedAt == firstOpenedAt)&&(identical(other.activeDays, activeDays) || other.activeDays == activeDays)&&(identical(other.lastActiveDay, lastActiveDay) || other.lastActiveDay == lastActiveDay)&&(identical(other.notesEdited, notesEdited) || other.notesEdited == notesEdited)&&(identical(other.lastTroubleAt, lastTroubleAt) || other.lastTroubleAt == lastTroubleAt)&&(identical(other.lastAskedAt, lastAskedAt) || other.lastAskedAt == lastAskedAt)&&const DeepCollectionEquality().equals(other.askedVersions, askedVersions)&&(identical(other.firstSyncSeen, firstSyncSeen) || other.firstSyncSeen == firstSyncSeen));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,firstOpenedAt,activeDays,lastActiveDay,notesEdited,lastTroubleAt,lastAskedAt,const DeepCollectionEquality().hash(askedVersions),firstSyncSeen);

@override
String toString() {
  return 'ReviewPromptRecord(firstOpenedAt: $firstOpenedAt, activeDays: $activeDays, lastActiveDay: $lastActiveDay, notesEdited: $notesEdited, lastTroubleAt: $lastTroubleAt, lastAskedAt: $lastAskedAt, askedVersions: $askedVersions, firstSyncSeen: $firstSyncSeen)';
}


}

/// @nodoc
abstract mixin class $ReviewPromptRecordCopyWith<$Res>  {
  factory $ReviewPromptRecordCopyWith(ReviewPromptRecord value, $Res Function(ReviewPromptRecord) _then) = _$ReviewPromptRecordCopyWithImpl;
@useResult
$Res call({
 DateTime? firstOpenedAt, int activeDays, String? lastActiveDay, int notesEdited, DateTime? lastTroubleAt, DateTime? lastAskedAt, List<String> askedVersions, bool firstSyncSeen
});




}
/// @nodoc
class _$ReviewPromptRecordCopyWithImpl<$Res>
    implements $ReviewPromptRecordCopyWith<$Res> {
  _$ReviewPromptRecordCopyWithImpl(this._self, this._then);

  final ReviewPromptRecord _self;
  final $Res Function(ReviewPromptRecord) _then;

/// Create a copy of ReviewPromptRecord
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? firstOpenedAt = freezed,Object? activeDays = null,Object? lastActiveDay = freezed,Object? notesEdited = null,Object? lastTroubleAt = freezed,Object? lastAskedAt = freezed,Object? askedVersions = null,Object? firstSyncSeen = null,}) {
  return _then(_self.copyWith(
firstOpenedAt: freezed == firstOpenedAt ? _self.firstOpenedAt : firstOpenedAt // ignore: cast_nullable_to_non_nullable
as DateTime?,activeDays: null == activeDays ? _self.activeDays : activeDays // ignore: cast_nullable_to_non_nullable
as int,lastActiveDay: freezed == lastActiveDay ? _self.lastActiveDay : lastActiveDay // ignore: cast_nullable_to_non_nullable
as String?,notesEdited: null == notesEdited ? _self.notesEdited : notesEdited // ignore: cast_nullable_to_non_nullable
as int,lastTroubleAt: freezed == lastTroubleAt ? _self.lastTroubleAt : lastTroubleAt // ignore: cast_nullable_to_non_nullable
as DateTime?,lastAskedAt: freezed == lastAskedAt ? _self.lastAskedAt : lastAskedAt // ignore: cast_nullable_to_non_nullable
as DateTime?,askedVersions: null == askedVersions ? _self.askedVersions : askedVersions // ignore: cast_nullable_to_non_nullable
as List<String>,firstSyncSeen: null == firstSyncSeen ? _self.firstSyncSeen : firstSyncSeen // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [ReviewPromptRecord].
extension ReviewPromptRecordPatterns on ReviewPromptRecord {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ReviewPromptRecord value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ReviewPromptRecord() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ReviewPromptRecord value)  $default,){
final _that = this;
switch (_that) {
case _ReviewPromptRecord():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ReviewPromptRecord value)?  $default,){
final _that = this;
switch (_that) {
case _ReviewPromptRecord() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( DateTime? firstOpenedAt,  int activeDays,  String? lastActiveDay,  int notesEdited,  DateTime? lastTroubleAt,  DateTime? lastAskedAt,  List<String> askedVersions,  bool firstSyncSeen)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ReviewPromptRecord() when $default != null:
return $default(_that.firstOpenedAt,_that.activeDays,_that.lastActiveDay,_that.notesEdited,_that.lastTroubleAt,_that.lastAskedAt,_that.askedVersions,_that.firstSyncSeen);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( DateTime? firstOpenedAt,  int activeDays,  String? lastActiveDay,  int notesEdited,  DateTime? lastTroubleAt,  DateTime? lastAskedAt,  List<String> askedVersions,  bool firstSyncSeen)  $default,) {final _that = this;
switch (_that) {
case _ReviewPromptRecord():
return $default(_that.firstOpenedAt,_that.activeDays,_that.lastActiveDay,_that.notesEdited,_that.lastTroubleAt,_that.lastAskedAt,_that.askedVersions,_that.firstSyncSeen);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( DateTime? firstOpenedAt,  int activeDays,  String? lastActiveDay,  int notesEdited,  DateTime? lastTroubleAt,  DateTime? lastAskedAt,  List<String> askedVersions,  bool firstSyncSeen)?  $default,) {final _that = this;
switch (_that) {
case _ReviewPromptRecord() when $default != null:
return $default(_that.firstOpenedAt,_that.activeDays,_that.lastActiveDay,_that.notesEdited,_that.lastTroubleAt,_that.lastAskedAt,_that.askedVersions,_that.firstSyncSeen);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _ReviewPromptRecord implements ReviewPromptRecord {
  const _ReviewPromptRecord({this.firstOpenedAt, this.activeDays = 0, this.lastActiveDay, this.notesEdited = 0, this.lastTroubleAt, this.lastAskedAt, final  List<String> askedVersions = const <String>[], this.firstSyncSeen = false}): _askedVersions = askedVersions;
  factory _ReviewPromptRecord.fromJson(Map<String, dynamic> json) => _$ReviewPromptRecordFromJson(json);

/// When the app was first opened with this record (the install, or the
/// update that brought it). Null until then.
@override final  DateTime? firstOpenedAt;
/// On how many different days the app was opened.
@override@JsonKey() final  int activeDays;
/// The last day the app was opened, `YYYY-MM-DD` in local time.
@override final  String? lastActiveDay;
/// How many edit sessions: times a note was let go after it was written
/// in (the same note twice counts twice).
@override@JsonKey() final  int notesEdited;
/// When the app last crashed, or sync or signing in last failed.
@override final  DateTime? lastTroubleAt;
/// When the app last asked the system for a review.
@override final  DateTime? lastAskedAt;
/// The versions of the app that have asked, oldest first; one is never
/// asked from twice.
 final  List<String> _askedVersions;
/// The versions of the app that have asked, oldest first; one is never
/// asked from twice.
@override@JsonKey() List<String> get askedVersions {
  if (_askedVersions is EqualUnmodifiableListView) return _askedVersions;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_askedVersions);
}

/// Whether a sync has completed on this phone.
@override@JsonKey() final  bool firstSyncSeen;

/// Create a copy of ReviewPromptRecord
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ReviewPromptRecordCopyWith<_ReviewPromptRecord> get copyWith => __$ReviewPromptRecordCopyWithImpl<_ReviewPromptRecord>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$ReviewPromptRecordToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ReviewPromptRecord&&(identical(other.firstOpenedAt, firstOpenedAt) || other.firstOpenedAt == firstOpenedAt)&&(identical(other.activeDays, activeDays) || other.activeDays == activeDays)&&(identical(other.lastActiveDay, lastActiveDay) || other.lastActiveDay == lastActiveDay)&&(identical(other.notesEdited, notesEdited) || other.notesEdited == notesEdited)&&(identical(other.lastTroubleAt, lastTroubleAt) || other.lastTroubleAt == lastTroubleAt)&&(identical(other.lastAskedAt, lastAskedAt) || other.lastAskedAt == lastAskedAt)&&const DeepCollectionEquality().equals(other._askedVersions, _askedVersions)&&(identical(other.firstSyncSeen, firstSyncSeen) || other.firstSyncSeen == firstSyncSeen));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,firstOpenedAt,activeDays,lastActiveDay,notesEdited,lastTroubleAt,lastAskedAt,const DeepCollectionEquality().hash(_askedVersions),firstSyncSeen);

@override
String toString() {
  return 'ReviewPromptRecord(firstOpenedAt: $firstOpenedAt, activeDays: $activeDays, lastActiveDay: $lastActiveDay, notesEdited: $notesEdited, lastTroubleAt: $lastTroubleAt, lastAskedAt: $lastAskedAt, askedVersions: $askedVersions, firstSyncSeen: $firstSyncSeen)';
}


}

/// @nodoc
abstract mixin class _$ReviewPromptRecordCopyWith<$Res> implements $ReviewPromptRecordCopyWith<$Res> {
  factory _$ReviewPromptRecordCopyWith(_ReviewPromptRecord value, $Res Function(_ReviewPromptRecord) _then) = __$ReviewPromptRecordCopyWithImpl;
@override @useResult
$Res call({
 DateTime? firstOpenedAt, int activeDays, String? lastActiveDay, int notesEdited, DateTime? lastTroubleAt, DateTime? lastAskedAt, List<String> askedVersions, bool firstSyncSeen
});




}
/// @nodoc
class __$ReviewPromptRecordCopyWithImpl<$Res>
    implements _$ReviewPromptRecordCopyWith<$Res> {
  __$ReviewPromptRecordCopyWithImpl(this._self, this._then);

  final _ReviewPromptRecord _self;
  final $Res Function(_ReviewPromptRecord) _then;

/// Create a copy of ReviewPromptRecord
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? firstOpenedAt = freezed,Object? activeDays = null,Object? lastActiveDay = freezed,Object? notesEdited = null,Object? lastTroubleAt = freezed,Object? lastAskedAt = freezed,Object? askedVersions = null,Object? firstSyncSeen = null,}) {
  return _then(_ReviewPromptRecord(
firstOpenedAt: freezed == firstOpenedAt ? _self.firstOpenedAt : firstOpenedAt // ignore: cast_nullable_to_non_nullable
as DateTime?,activeDays: null == activeDays ? _self.activeDays : activeDays // ignore: cast_nullable_to_non_nullable
as int,lastActiveDay: freezed == lastActiveDay ? _self.lastActiveDay : lastActiveDay // ignore: cast_nullable_to_non_nullable
as String?,notesEdited: null == notesEdited ? _self.notesEdited : notesEdited // ignore: cast_nullable_to_non_nullable
as int,lastTroubleAt: freezed == lastTroubleAt ? _self.lastTroubleAt : lastTroubleAt // ignore: cast_nullable_to_non_nullable
as DateTime?,lastAskedAt: freezed == lastAskedAt ? _self.lastAskedAt : lastAskedAt // ignore: cast_nullable_to_non_nullable
as DateTime?,askedVersions: null == askedVersions ? _self._askedVersions : askedVersions // ignore: cast_nullable_to_non_nullable
as List<String>,firstSyncSeen: null == firstSyncSeen ? _self.firstSyncSeen : firstSyncSeen // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
