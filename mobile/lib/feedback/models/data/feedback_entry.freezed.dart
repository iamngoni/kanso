// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'feedback_entry.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$FeedbackEntry {

 String get id; int get createdMs; FeedbackKind get kind;/// The first line of the text.
 String get preview; FeedbackChip get chip;/// The issue number for [FeedbackChip.filedAs] and [FeedbackChip.sameAs].
 int? get number;/// `0.9.2` for [FeedbackChip.releasedIn].
 String? get release;/// The pull request that fixes it, once there is one.
 int? get pr;/// Where the issue is on GitHub, once it is filed.
 String? get issueUrl;/// A delete is waiting for the server.
 bool get deleting;/// Released, and not yet looked at.
 bool get fresh;/// Whether the server has it (not still waiting on this phone).
 bool get onServer;
/// Create a copy of FeedbackEntry
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FeedbackEntryCopyWith<FeedbackEntry> get copyWith => _$FeedbackEntryCopyWithImpl<FeedbackEntry>(this as FeedbackEntry, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FeedbackEntry&&(identical(other.id, id) || other.id == id)&&(identical(other.createdMs, createdMs) || other.createdMs == createdMs)&&(identical(other.kind, kind) || other.kind == kind)&&(identical(other.preview, preview) || other.preview == preview)&&(identical(other.chip, chip) || other.chip == chip)&&(identical(other.number, number) || other.number == number)&&(identical(other.release, release) || other.release == release)&&(identical(other.pr, pr) || other.pr == pr)&&(identical(other.issueUrl, issueUrl) || other.issueUrl == issueUrl)&&(identical(other.deleting, deleting) || other.deleting == deleting)&&(identical(other.fresh, fresh) || other.fresh == fresh)&&(identical(other.onServer, onServer) || other.onServer == onServer));
}


@override
int get hashCode => Object.hash(runtimeType,id,createdMs,kind,preview,chip,number,release,pr,issueUrl,deleting,fresh,onServer);

@override
String toString() {
  return 'FeedbackEntry(id: $id, createdMs: $createdMs, kind: $kind, preview: $preview, chip: $chip, number: $number, release: $release, pr: $pr, issueUrl: $issueUrl, deleting: $deleting, fresh: $fresh, onServer: $onServer)';
}


}

/// @nodoc
abstract mixin class $FeedbackEntryCopyWith<$Res>  {
  factory $FeedbackEntryCopyWith(FeedbackEntry value, $Res Function(FeedbackEntry) _then) = _$FeedbackEntryCopyWithImpl;
@useResult
$Res call({
 String id, int createdMs, FeedbackKind kind, String preview, FeedbackChip chip, int? number, String? release, int? pr, String? issueUrl, bool deleting, bool fresh, bool onServer
});




}
/// @nodoc
class _$FeedbackEntryCopyWithImpl<$Res>
    implements $FeedbackEntryCopyWith<$Res> {
  _$FeedbackEntryCopyWithImpl(this._self, this._then);

  final FeedbackEntry _self;
  final $Res Function(FeedbackEntry) _then;

/// Create a copy of FeedbackEntry
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? createdMs = null,Object? kind = null,Object? preview = null,Object? chip = null,Object? number = freezed,Object? release = freezed,Object? pr = freezed,Object? issueUrl = freezed,Object? deleting = null,Object? fresh = null,Object? onServer = null,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,createdMs: null == createdMs ? _self.createdMs : createdMs // ignore: cast_nullable_to_non_nullable
as int,kind: null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as FeedbackKind,preview: null == preview ? _self.preview : preview // ignore: cast_nullable_to_non_nullable
as String,chip: null == chip ? _self.chip : chip // ignore: cast_nullable_to_non_nullable
as FeedbackChip,number: freezed == number ? _self.number : number // ignore: cast_nullable_to_non_nullable
as int?,release: freezed == release ? _self.release : release // ignore: cast_nullable_to_non_nullable
as String?,pr: freezed == pr ? _self.pr : pr // ignore: cast_nullable_to_non_nullable
as int?,issueUrl: freezed == issueUrl ? _self.issueUrl : issueUrl // ignore: cast_nullable_to_non_nullable
as String?,deleting: null == deleting ? _self.deleting : deleting // ignore: cast_nullable_to_non_nullable
as bool,fresh: null == fresh ? _self.fresh : fresh // ignore: cast_nullable_to_non_nullable
as bool,onServer: null == onServer ? _self.onServer : onServer // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [FeedbackEntry].
extension FeedbackEntryPatterns on FeedbackEntry {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _FeedbackEntry value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _FeedbackEntry() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _FeedbackEntry value)  $default,){
final _that = this;
switch (_that) {
case _FeedbackEntry():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _FeedbackEntry value)?  $default,){
final _that = this;
switch (_that) {
case _FeedbackEntry() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String id,  int createdMs,  FeedbackKind kind,  String preview,  FeedbackChip chip,  int? number,  String? release,  int? pr,  String? issueUrl,  bool deleting,  bool fresh,  bool onServer)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _FeedbackEntry() when $default != null:
return $default(_that.id,_that.createdMs,_that.kind,_that.preview,_that.chip,_that.number,_that.release,_that.pr,_that.issueUrl,_that.deleting,_that.fresh,_that.onServer);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String id,  int createdMs,  FeedbackKind kind,  String preview,  FeedbackChip chip,  int? number,  String? release,  int? pr,  String? issueUrl,  bool deleting,  bool fresh,  bool onServer)  $default,) {final _that = this;
switch (_that) {
case _FeedbackEntry():
return $default(_that.id,_that.createdMs,_that.kind,_that.preview,_that.chip,_that.number,_that.release,_that.pr,_that.issueUrl,_that.deleting,_that.fresh,_that.onServer);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String id,  int createdMs,  FeedbackKind kind,  String preview,  FeedbackChip chip,  int? number,  String? release,  int? pr,  String? issueUrl,  bool deleting,  bool fresh,  bool onServer)?  $default,) {final _that = this;
switch (_that) {
case _FeedbackEntry() when $default != null:
return $default(_that.id,_that.createdMs,_that.kind,_that.preview,_that.chip,_that.number,_that.release,_that.pr,_that.issueUrl,_that.deleting,_that.fresh,_that.onServer);case _:
  return null;

}
}

}

/// @nodoc


class _FeedbackEntry implements FeedbackEntry {
  const _FeedbackEntry({required this.id, required this.createdMs, required this.kind, required this.preview, required this.chip, this.number, this.release, this.pr, this.issueUrl, this.deleting = false, this.fresh = false, this.onServer = true});
  

@override final  String id;
@override final  int createdMs;
@override final  FeedbackKind kind;
/// The first line of the text.
@override final  String preview;
@override final  FeedbackChip chip;
/// The issue number for [FeedbackChip.filedAs] and [FeedbackChip.sameAs].
@override final  int? number;
/// `0.9.2` for [FeedbackChip.releasedIn].
@override final  String? release;
/// The pull request that fixes it, once there is one.
@override final  int? pr;
/// Where the issue is on GitHub, once it is filed.
@override final  String? issueUrl;
/// A delete is waiting for the server.
@override@JsonKey() final  bool deleting;
/// Released, and not yet looked at.
@override@JsonKey() final  bool fresh;
/// Whether the server has it (not still waiting on this phone).
@override@JsonKey() final  bool onServer;

/// Create a copy of FeedbackEntry
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$FeedbackEntryCopyWith<_FeedbackEntry> get copyWith => __$FeedbackEntryCopyWithImpl<_FeedbackEntry>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _FeedbackEntry&&(identical(other.id, id) || other.id == id)&&(identical(other.createdMs, createdMs) || other.createdMs == createdMs)&&(identical(other.kind, kind) || other.kind == kind)&&(identical(other.preview, preview) || other.preview == preview)&&(identical(other.chip, chip) || other.chip == chip)&&(identical(other.number, number) || other.number == number)&&(identical(other.release, release) || other.release == release)&&(identical(other.pr, pr) || other.pr == pr)&&(identical(other.issueUrl, issueUrl) || other.issueUrl == issueUrl)&&(identical(other.deleting, deleting) || other.deleting == deleting)&&(identical(other.fresh, fresh) || other.fresh == fresh)&&(identical(other.onServer, onServer) || other.onServer == onServer));
}


@override
int get hashCode => Object.hash(runtimeType,id,createdMs,kind,preview,chip,number,release,pr,issueUrl,deleting,fresh,onServer);

@override
String toString() {
  return 'FeedbackEntry(id: $id, createdMs: $createdMs, kind: $kind, preview: $preview, chip: $chip, number: $number, release: $release, pr: $pr, issueUrl: $issueUrl, deleting: $deleting, fresh: $fresh, onServer: $onServer)';
}


}

/// @nodoc
abstract mixin class _$FeedbackEntryCopyWith<$Res> implements $FeedbackEntryCopyWith<$Res> {
  factory _$FeedbackEntryCopyWith(_FeedbackEntry value, $Res Function(_FeedbackEntry) _then) = __$FeedbackEntryCopyWithImpl;
@override @useResult
$Res call({
 String id, int createdMs, FeedbackKind kind, String preview, FeedbackChip chip, int? number, String? release, int? pr, String? issueUrl, bool deleting, bool fresh, bool onServer
});




}
/// @nodoc
class __$FeedbackEntryCopyWithImpl<$Res>
    implements _$FeedbackEntryCopyWith<$Res> {
  __$FeedbackEntryCopyWithImpl(this._self, this._then);

  final _FeedbackEntry _self;
  final $Res Function(_FeedbackEntry) _then;

/// Create a copy of FeedbackEntry
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? createdMs = null,Object? kind = null,Object? preview = null,Object? chip = null,Object? number = freezed,Object? release = freezed,Object? pr = freezed,Object? issueUrl = freezed,Object? deleting = null,Object? fresh = null,Object? onServer = null,}) {
  return _then(_FeedbackEntry(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,createdMs: null == createdMs ? _self.createdMs : createdMs // ignore: cast_nullable_to_non_nullable
as int,kind: null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as FeedbackKind,preview: null == preview ? _self.preview : preview // ignore: cast_nullable_to_non_nullable
as String,chip: null == chip ? _self.chip : chip // ignore: cast_nullable_to_non_nullable
as FeedbackChip,number: freezed == number ? _self.number : number // ignore: cast_nullable_to_non_nullable
as int?,release: freezed == release ? _self.release : release // ignore: cast_nullable_to_non_nullable
as String?,pr: freezed == pr ? _self.pr : pr // ignore: cast_nullable_to_non_nullable
as int?,issueUrl: freezed == issueUrl ? _self.issueUrl : issueUrl // ignore: cast_nullable_to_non_nullable
as String?,deleting: null == deleting ? _self.deleting : deleting // ignore: cast_nullable_to_non_nullable
as bool,fresh: null == fresh ? _self.fresh : fresh // ignore: cast_nullable_to_non_nullable
as bool,onServer: null == onServer ? _self.onServer : onServer // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
