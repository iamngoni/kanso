// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'lock_target.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$LockTarget {

 String get id;
/// Create a copy of LockTarget
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LockTargetCopyWith<LockTarget> get copyWith => _$LockTargetCopyWithImpl<LockTarget>(this as LockTarget, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockTarget&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'LockTarget(id: $id)';
}


}

/// @nodoc
abstract mixin class $LockTargetCopyWith<$Res>  {
  factory $LockTargetCopyWith(LockTarget value, $Res Function(LockTarget) _then) = _$LockTargetCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$LockTargetCopyWithImpl<$Res>
    implements $LockTargetCopyWith<$Res> {
  _$LockTargetCopyWithImpl(this._self, this._then);

  final LockTarget _self;
  final $Res Function(LockTarget) _then;

/// Create a copy of LockTarget
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [LockTarget].
extension LockTargetPatterns on LockTarget {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( NoteLockTarget value)?  note,TResult Function( NotebookLockTarget value)?  notebook,required TResult orElse(),}){
final _that = this;
switch (_that) {
case NoteLockTarget() when note != null:
return note(_that);case NotebookLockTarget() when notebook != null:
return notebook(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( NoteLockTarget value)  note,required TResult Function( NotebookLockTarget value)  notebook,}){
final _that = this;
switch (_that) {
case NoteLockTarget():
return note(_that);case NotebookLockTarget():
return notebook(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( NoteLockTarget value)?  note,TResult? Function( NotebookLockTarget value)?  notebook,}){
final _that = this;
switch (_that) {
case NoteLockTarget() when note != null:
return note(_that);case NotebookLockTarget() when notebook != null:
return notebook(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String id)?  note,TResult Function( String id,  String name)?  notebook,required TResult orElse(),}) {final _that = this;
switch (_that) {
case NoteLockTarget() when note != null:
return note(_that.id);case NotebookLockTarget() when notebook != null:
return notebook(_that.id,_that.name);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String id)  note,required TResult Function( String id,  String name)  notebook,}) {final _that = this;
switch (_that) {
case NoteLockTarget():
return note(_that.id);case NotebookLockTarget():
return notebook(_that.id,_that.name);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String id)?  note,TResult? Function( String id,  String name)?  notebook,}) {final _that = this;
switch (_that) {
case NoteLockTarget() when note != null:
return note(_that.id);case NotebookLockTarget() when notebook != null:
return notebook(_that.id,_that.name);case _:
  return null;

}
}

}

/// @nodoc


class NoteLockTarget implements LockTarget {
  const NoteLockTarget(this.id);
  

@override final  String id;

/// Create a copy of LockTarget
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteLockTargetCopyWith<NoteLockTarget> get copyWith => _$NoteLockTargetCopyWithImpl<NoteLockTarget>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteLockTarget&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'LockTarget.note(id: $id)';
}


}

/// @nodoc
abstract mixin class $NoteLockTargetCopyWith<$Res> implements $LockTargetCopyWith<$Res> {
  factory $NoteLockTargetCopyWith(NoteLockTarget value, $Res Function(NoteLockTarget) _then) = _$NoteLockTargetCopyWithImpl;
@override @useResult
$Res call({
 String id
});




}
/// @nodoc
class _$NoteLockTargetCopyWithImpl<$Res>
    implements $NoteLockTargetCopyWith<$Res> {
  _$NoteLockTargetCopyWithImpl(this._self, this._then);

  final NoteLockTarget _self;
  final $Res Function(NoteLockTarget) _then;

/// Create a copy of LockTarget
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(NoteLockTarget(
null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NotebookLockTarget implements LockTarget {
  const NotebookLockTarget(this.id, this.name);
  

@override final  String id;
 final  String name;

/// Create a copy of LockTarget
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NotebookLockTargetCopyWith<NotebookLockTarget> get copyWith => _$NotebookLockTargetCopyWithImpl<NotebookLockTarget>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotebookLockTarget&&(identical(other.id, id) || other.id == id)&&(identical(other.name, name) || other.name == name));
}


@override
int get hashCode => Object.hash(runtimeType,id,name);

@override
String toString() {
  return 'LockTarget.notebook(id: $id, name: $name)';
}


}

/// @nodoc
abstract mixin class $NotebookLockTargetCopyWith<$Res> implements $LockTargetCopyWith<$Res> {
  factory $NotebookLockTargetCopyWith(NotebookLockTarget value, $Res Function(NotebookLockTarget) _then) = _$NotebookLockTargetCopyWithImpl;
@override @useResult
$Res call({
 String id, String name
});




}
/// @nodoc
class _$NotebookLockTargetCopyWithImpl<$Res>
    implements $NotebookLockTargetCopyWith<$Res> {
  _$NotebookLockTargetCopyWithImpl(this._self, this._then);

  final NotebookLockTarget _self;
  final $Res Function(NotebookLockTarget) _then;

/// Create a copy of LockTarget
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? name = null,}) {
  return _then(NotebookLockTarget(
null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
