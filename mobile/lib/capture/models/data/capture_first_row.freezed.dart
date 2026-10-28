// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'capture_first_row.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$CaptureFirstRow implements DiagnosticableTreeMixin {

 KansoIcons get icon; String get label;/// What choosing the row does, once the sheet has closed.
 VoidCallback get onTap;
/// Create a copy of CaptureFirstRow
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CaptureFirstRowCopyWith<CaptureFirstRow> get copyWith => _$CaptureFirstRowCopyWithImpl<CaptureFirstRow>(this as CaptureFirstRow, _$identity);


@override
void debugFillProperties(DiagnosticPropertiesBuilder properties) {
  properties
    ..add(DiagnosticsProperty('type', 'CaptureFirstRow'))
    ..add(DiagnosticsProperty('icon', icon))..add(DiagnosticsProperty('label', label))..add(DiagnosticsProperty('onTap', onTap));
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CaptureFirstRow&&(identical(other.icon, icon) || other.icon == icon)&&(identical(other.label, label) || other.label == label)&&(identical(other.onTap, onTap) || other.onTap == onTap));
}


@override
int get hashCode => Object.hash(runtimeType,icon,label,onTap);

@override
String toString({ DiagnosticLevel minLevel = DiagnosticLevel.info }) {
  return 'CaptureFirstRow(icon: $icon, label: $label, onTap: $onTap)';
}


}

/// @nodoc
abstract mixin class $CaptureFirstRowCopyWith<$Res>  {
  factory $CaptureFirstRowCopyWith(CaptureFirstRow value, $Res Function(CaptureFirstRow) _then) = _$CaptureFirstRowCopyWithImpl;
@useResult
$Res call({
 KansoIcons icon, String label, VoidCallback onTap
});




}
/// @nodoc
class _$CaptureFirstRowCopyWithImpl<$Res>
    implements $CaptureFirstRowCopyWith<$Res> {
  _$CaptureFirstRowCopyWithImpl(this._self, this._then);

  final CaptureFirstRow _self;
  final $Res Function(CaptureFirstRow) _then;

/// Create a copy of CaptureFirstRow
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? icon = null,Object? label = null,Object? onTap = null,}) {
  return _then(_self.copyWith(
icon: null == icon ? _self.icon : icon // ignore: cast_nullable_to_non_nullable
as KansoIcons,label: null == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String,onTap: null == onTap ? _self.onTap : onTap // ignore: cast_nullable_to_non_nullable
as VoidCallback,
  ));
}

}


/// Adds pattern-matching-related methods to [CaptureFirstRow].
extension CaptureFirstRowPatterns on CaptureFirstRow {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _CaptureFirstRow value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _CaptureFirstRow() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _CaptureFirstRow value)  $default,){
final _that = this;
switch (_that) {
case _CaptureFirstRow():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _CaptureFirstRow value)?  $default,){
final _that = this;
switch (_that) {
case _CaptureFirstRow() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( KansoIcons icon,  String label,  VoidCallback onTap)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _CaptureFirstRow() when $default != null:
return $default(_that.icon,_that.label,_that.onTap);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( KansoIcons icon,  String label,  VoidCallback onTap)  $default,) {final _that = this;
switch (_that) {
case _CaptureFirstRow():
return $default(_that.icon,_that.label,_that.onTap);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( KansoIcons icon,  String label,  VoidCallback onTap)?  $default,) {final _that = this;
switch (_that) {
case _CaptureFirstRow() when $default != null:
return $default(_that.icon,_that.label,_that.onTap);case _:
  return null;

}
}

}

/// @nodoc


class _CaptureFirstRow with DiagnosticableTreeMixin implements CaptureFirstRow {
  const _CaptureFirstRow({required this.icon, required this.label, required this.onTap});
  

@override final  KansoIcons icon;
@override final  String label;
/// What choosing the row does, once the sheet has closed.
@override final  VoidCallback onTap;

/// Create a copy of CaptureFirstRow
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$CaptureFirstRowCopyWith<_CaptureFirstRow> get copyWith => __$CaptureFirstRowCopyWithImpl<_CaptureFirstRow>(this, _$identity);


@override
void debugFillProperties(DiagnosticPropertiesBuilder properties) {
  properties
    ..add(DiagnosticsProperty('type', 'CaptureFirstRow'))
    ..add(DiagnosticsProperty('icon', icon))..add(DiagnosticsProperty('label', label))..add(DiagnosticsProperty('onTap', onTap));
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _CaptureFirstRow&&(identical(other.icon, icon) || other.icon == icon)&&(identical(other.label, label) || other.label == label)&&(identical(other.onTap, onTap) || other.onTap == onTap));
}


@override
int get hashCode => Object.hash(runtimeType,icon,label,onTap);

@override
String toString({ DiagnosticLevel minLevel = DiagnosticLevel.info }) {
  return 'CaptureFirstRow(icon: $icon, label: $label, onTap: $onTap)';
}


}

/// @nodoc
abstract mixin class _$CaptureFirstRowCopyWith<$Res> implements $CaptureFirstRowCopyWith<$Res> {
  factory _$CaptureFirstRowCopyWith(_CaptureFirstRow value, $Res Function(_CaptureFirstRow) _then) = __$CaptureFirstRowCopyWithImpl;
@override @useResult
$Res call({
 KansoIcons icon, String label, VoidCallback onTap
});




}
/// @nodoc
class __$CaptureFirstRowCopyWithImpl<$Res>
    implements _$CaptureFirstRowCopyWith<$Res> {
  __$CaptureFirstRowCopyWithImpl(this._self, this._then);

  final _CaptureFirstRow _self;
  final $Res Function(_CaptureFirstRow) _then;

/// Create a copy of CaptureFirstRow
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? icon = null,Object? label = null,Object? onTap = null,}) {
  return _then(_CaptureFirstRow(
icon: null == icon ? _self.icon : icon // ignore: cast_nullable_to_non_nullable
as KansoIcons,label: null == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String,onTap: null == onTap ? _self.onTap : onTap // ignore: cast_nullable_to_non_nullable
as VoidCallback,
  ));
}


}

// dart format on
