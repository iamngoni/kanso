// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'palette_choice.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$PaletteChoice {

/// What the line says.
 String get label;/// What choosing it gives the command.
 String get value;/// Whether the note already has it.
 bool get checked;/// How far a notebook nests, for the lines that nest.
 int get depth;
/// Create a copy of PaletteChoice
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PaletteChoiceCopyWith<PaletteChoice> get copyWith => _$PaletteChoiceCopyWithImpl<PaletteChoice>(this as PaletteChoice, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PaletteChoice&&(identical(other.label, label) || other.label == label)&&(identical(other.value, value) || other.value == value)&&(identical(other.checked, checked) || other.checked == checked)&&(identical(other.depth, depth) || other.depth == depth));
}


@override
int get hashCode => Object.hash(runtimeType,label,value,checked,depth);

@override
String toString() {
  return 'PaletteChoice(label: $label, value: $value, checked: $checked, depth: $depth)';
}


}

/// @nodoc
abstract mixin class $PaletteChoiceCopyWith<$Res>  {
  factory $PaletteChoiceCopyWith(PaletteChoice value, $Res Function(PaletteChoice) _then) = _$PaletteChoiceCopyWithImpl;
@useResult
$Res call({
 String label, String value, bool checked, int depth
});




}
/// @nodoc
class _$PaletteChoiceCopyWithImpl<$Res>
    implements $PaletteChoiceCopyWith<$Res> {
  _$PaletteChoiceCopyWithImpl(this._self, this._then);

  final PaletteChoice _self;
  final $Res Function(PaletteChoice) _then;

/// Create a copy of PaletteChoice
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? label = null,Object? value = null,Object? checked = null,Object? depth = null,}) {
  return _then(_self.copyWith(
label: null == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String,value: null == value ? _self.value : value // ignore: cast_nullable_to_non_nullable
as String,checked: null == checked ? _self.checked : checked // ignore: cast_nullable_to_non_nullable
as bool,depth: null == depth ? _self.depth : depth // ignore: cast_nullable_to_non_nullable
as int,
  ));
}

}


/// Adds pattern-matching-related methods to [PaletteChoice].
extension PaletteChoicePatterns on PaletteChoice {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _PaletteChoice value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _PaletteChoice() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _PaletteChoice value)  $default,){
final _that = this;
switch (_that) {
case _PaletteChoice():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _PaletteChoice value)?  $default,){
final _that = this;
switch (_that) {
case _PaletteChoice() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String label,  String value,  bool checked,  int depth)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _PaletteChoice() when $default != null:
return $default(_that.label,_that.value,_that.checked,_that.depth);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String label,  String value,  bool checked,  int depth)  $default,) {final _that = this;
switch (_that) {
case _PaletteChoice():
return $default(_that.label,_that.value,_that.checked,_that.depth);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String label,  String value,  bool checked,  int depth)?  $default,) {final _that = this;
switch (_that) {
case _PaletteChoice() when $default != null:
return $default(_that.label,_that.value,_that.checked,_that.depth);case _:
  return null;

}
}

}

/// @nodoc


class _PaletteChoice implements PaletteChoice {
  const _PaletteChoice({required this.label, required this.value, this.checked = false, this.depth = 0});
  

/// What the line says.
@override final  String label;
/// What choosing it gives the command.
@override final  String value;
/// Whether the note already has it.
@override@JsonKey() final  bool checked;
/// How far a notebook nests, for the lines that nest.
@override@JsonKey() final  int depth;

/// Create a copy of PaletteChoice
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$PaletteChoiceCopyWith<_PaletteChoice> get copyWith => __$PaletteChoiceCopyWithImpl<_PaletteChoice>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _PaletteChoice&&(identical(other.label, label) || other.label == label)&&(identical(other.value, value) || other.value == value)&&(identical(other.checked, checked) || other.checked == checked)&&(identical(other.depth, depth) || other.depth == depth));
}


@override
int get hashCode => Object.hash(runtimeType,label,value,checked,depth);

@override
String toString() {
  return 'PaletteChoice(label: $label, value: $value, checked: $checked, depth: $depth)';
}


}

/// @nodoc
abstract mixin class _$PaletteChoiceCopyWith<$Res> implements $PaletteChoiceCopyWith<$Res> {
  factory _$PaletteChoiceCopyWith(_PaletteChoice value, $Res Function(_PaletteChoice) _then) = __$PaletteChoiceCopyWithImpl;
@override @useResult
$Res call({
 String label, String value, bool checked, int depth
});




}
/// @nodoc
class __$PaletteChoiceCopyWithImpl<$Res>
    implements _$PaletteChoiceCopyWith<$Res> {
  __$PaletteChoiceCopyWithImpl(this._self, this._then);

  final _PaletteChoice _self;
  final $Res Function(_PaletteChoice) _then;

/// Create a copy of PaletteChoice
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? label = null,Object? value = null,Object? checked = null,Object? depth = null,}) {
  return _then(_PaletteChoice(
label: null == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String,value: null == value ? _self.value : value // ignore: cast_nullable_to_non_nullable
as String,checked: null == checked ? _self.checked : checked // ignore: cast_nullable_to_non_nullable
as bool,depth: null == depth ? _self.depth : depth // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

// dart format on
