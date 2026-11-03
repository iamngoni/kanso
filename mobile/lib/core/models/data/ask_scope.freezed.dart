// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'ask_scope.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AskScope {

/// `all`, `book` or `tag`.
 String get kind;/// The notebook's ID or the tag's name; empty for all notes.
 String get value;/// "All notes", the notebook's path (`Work/Plans`) or `#tag`.
 String get label;
/// Create a copy of AskScope
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskScopeCopyWith<AskScope> get copyWith => _$AskScopeCopyWithImpl<AskScope>(this as AskScope, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskScope&&(identical(other.kind, kind) || other.kind == kind)&&(identical(other.value, value) || other.value == value)&&(identical(other.label, label) || other.label == label));
}


@override
int get hashCode => Object.hash(runtimeType,kind,value,label);

@override
String toString() {
  return 'AskScope(kind: $kind, value: $value, label: $label)';
}


}

/// @nodoc
abstract mixin class $AskScopeCopyWith<$Res>  {
  factory $AskScopeCopyWith(AskScope value, $Res Function(AskScope) _then) = _$AskScopeCopyWithImpl;
@useResult
$Res call({
 String kind, String value, String label
});




}
/// @nodoc
class _$AskScopeCopyWithImpl<$Res>
    implements $AskScopeCopyWith<$Res> {
  _$AskScopeCopyWithImpl(this._self, this._then);

  final AskScope _self;
  final $Res Function(AskScope) _then;

/// Create a copy of AskScope
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? kind = null,Object? value = null,Object? label = null,}) {
  return _then(_self.copyWith(
kind: null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as String,value: null == value ? _self.value : value // ignore: cast_nullable_to_non_nullable
as String,label: null == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [AskScope].
extension AskScopePatterns on AskScope {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AskScope value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AskScope() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AskScope value)  $default,){
final _that = this;
switch (_that) {
case _AskScope():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AskScope value)?  $default,){
final _that = this;
switch (_that) {
case _AskScope() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String kind,  String value,  String label)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AskScope() when $default != null:
return $default(_that.kind,_that.value,_that.label);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String kind,  String value,  String label)  $default,) {final _that = this;
switch (_that) {
case _AskScope():
return $default(_that.kind,_that.value,_that.label);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String kind,  String value,  String label)?  $default,) {final _that = this;
switch (_that) {
case _AskScope() when $default != null:
return $default(_that.kind,_that.value,_that.label);case _:
  return null;

}
}

}

/// @nodoc


class _AskScope extends AskScope {
  const _AskScope({this.kind = AskScope.allKind, this.value = '', this.label = ''}): super._();
  

/// `all`, `book` or `tag`.
@override@JsonKey() final  String kind;
/// The notebook's ID or the tag's name; empty for all notes.
@override@JsonKey() final  String value;
/// "All notes", the notebook's path (`Work/Plans`) or `#tag`.
@override@JsonKey() final  String label;

/// Create a copy of AskScope
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AskScopeCopyWith<_AskScope> get copyWith => __$AskScopeCopyWithImpl<_AskScope>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AskScope&&(identical(other.kind, kind) || other.kind == kind)&&(identical(other.value, value) || other.value == value)&&(identical(other.label, label) || other.label == label));
}


@override
int get hashCode => Object.hash(runtimeType,kind,value,label);

@override
String toString() {
  return 'AskScope(kind: $kind, value: $value, label: $label)';
}


}

/// @nodoc
abstract mixin class _$AskScopeCopyWith<$Res> implements $AskScopeCopyWith<$Res> {
  factory _$AskScopeCopyWith(_AskScope value, $Res Function(_AskScope) _then) = __$AskScopeCopyWithImpl;
@override @useResult
$Res call({
 String kind, String value, String label
});




}
/// @nodoc
class __$AskScopeCopyWithImpl<$Res>
    implements _$AskScopeCopyWith<$Res> {
  __$AskScopeCopyWithImpl(this._self, this._then);

  final _AskScope _self;
  final $Res Function(_AskScope) _then;

/// Create a copy of AskScope
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? kind = null,Object? value = null,Object? label = null,}) {
  return _then(_AskScope(
kind: null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as String,value: null == value ? _self.value : value // ignore: cast_nullable_to_non_nullable
as String,label: null == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
