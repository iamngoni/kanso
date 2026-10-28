// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'opened_batch.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$OpenedBatch {

/// The documents, in the order they arrived.
 List<OpenedFile> get files;/// The folder the library stages copies of them under.
 String get staging;
/// Create a copy of OpenedBatch
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$OpenedBatchCopyWith<OpenedBatch> get copyWith => _$OpenedBatchCopyWithImpl<OpenedBatch>(this as OpenedBatch, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is OpenedBatch&&const DeepCollectionEquality().equals(other.files, files)&&(identical(other.staging, staging) || other.staging == staging));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(files),staging);

@override
String toString() {
  return 'OpenedBatch(files: $files, staging: $staging)';
}


}

/// @nodoc
abstract mixin class $OpenedBatchCopyWith<$Res>  {
  factory $OpenedBatchCopyWith(OpenedBatch value, $Res Function(OpenedBatch) _then) = _$OpenedBatchCopyWithImpl;
@useResult
$Res call({
 List<OpenedFile> files, String staging
});




}
/// @nodoc
class _$OpenedBatchCopyWithImpl<$Res>
    implements $OpenedBatchCopyWith<$Res> {
  _$OpenedBatchCopyWithImpl(this._self, this._then);

  final OpenedBatch _self;
  final $Res Function(OpenedBatch) _then;

/// Create a copy of OpenedBatch
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? files = null,Object? staging = null,}) {
  return _then(_self.copyWith(
files: null == files ? _self.files : files // ignore: cast_nullable_to_non_nullable
as List<OpenedFile>,staging: null == staging ? _self.staging : staging // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [OpenedBatch].
extension OpenedBatchPatterns on OpenedBatch {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _OpenedBatch value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _OpenedBatch() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _OpenedBatch value)  $default,){
final _that = this;
switch (_that) {
case _OpenedBatch():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _OpenedBatch value)?  $default,){
final _that = this;
switch (_that) {
case _OpenedBatch() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( List<OpenedFile> files,  String staging)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _OpenedBatch() when $default != null:
return $default(_that.files,_that.staging);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( List<OpenedFile> files,  String staging)  $default,) {final _that = this;
switch (_that) {
case _OpenedBatch():
return $default(_that.files,_that.staging);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( List<OpenedFile> files,  String staging)?  $default,) {final _that = this;
switch (_that) {
case _OpenedBatch() when $default != null:
return $default(_that.files,_that.staging);case _:
  return null;

}
}

}

/// @nodoc


class _OpenedBatch implements OpenedBatch {
  const _OpenedBatch({required final  List<OpenedFile> files, required this.staging}): _files = files;
  

/// The documents, in the order they arrived.
 final  List<OpenedFile> _files;
/// The documents, in the order they arrived.
@override List<OpenedFile> get files {
  if (_files is EqualUnmodifiableListView) return _files;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_files);
}

/// The folder the library stages copies of them under.
@override final  String staging;

/// Create a copy of OpenedBatch
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$OpenedBatchCopyWith<_OpenedBatch> get copyWith => __$OpenedBatchCopyWithImpl<_OpenedBatch>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _OpenedBatch&&const DeepCollectionEquality().equals(other._files, _files)&&(identical(other.staging, staging) || other.staging == staging));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_files),staging);

@override
String toString() {
  return 'OpenedBatch(files: $files, staging: $staging)';
}


}

/// @nodoc
abstract mixin class _$OpenedBatchCopyWith<$Res> implements $OpenedBatchCopyWith<$Res> {
  factory _$OpenedBatchCopyWith(_OpenedBatch value, $Res Function(_OpenedBatch) _then) = __$OpenedBatchCopyWithImpl;
@override @useResult
$Res call({
 List<OpenedFile> files, String staging
});




}
/// @nodoc
class __$OpenedBatchCopyWithImpl<$Res>
    implements _$OpenedBatchCopyWith<$Res> {
  __$OpenedBatchCopyWithImpl(this._self, this._then);

  final _OpenedBatch _self;
  final $Res Function(_OpenedBatch) _then;

/// Create a copy of OpenedBatch
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? files = null,Object? staging = null,}) {
  return _then(_OpenedBatch(
files: null == files ? _self._files : files // ignore: cast_nullable_to_non_nullable
as List<OpenedFile>,staging: null == staging ? _self.staging : staging // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
