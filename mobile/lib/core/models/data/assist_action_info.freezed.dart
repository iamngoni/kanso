// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'assist_action_info.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AssistActionInfo {

 AssistAction get action;/// The menu label: `Improve Writing`.
 String get label;/// What the suggestion is called once it is made: `Improved`.
 String get noun;/// Whether accepting replaces the selection, or else the suggestion goes
/// after it.
 bool get replaces;/// Whether the action needs text selected; Continue Writing does not.
 bool get needsSelection;
/// Create a copy of AssistActionInfo
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AssistActionInfoCopyWith<AssistActionInfo> get copyWith => _$AssistActionInfoCopyWithImpl<AssistActionInfo>(this as AssistActionInfo, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AssistActionInfo&&(identical(other.action, action) || other.action == action)&&(identical(other.label, label) || other.label == label)&&(identical(other.noun, noun) || other.noun == noun)&&(identical(other.replaces, replaces) || other.replaces == replaces)&&(identical(other.needsSelection, needsSelection) || other.needsSelection == needsSelection));
}


@override
int get hashCode => Object.hash(runtimeType,action,label,noun,replaces,needsSelection);

@override
String toString() {
  return 'AssistActionInfo(action: $action, label: $label, noun: $noun, replaces: $replaces, needsSelection: $needsSelection)';
}


}

/// @nodoc
abstract mixin class $AssistActionInfoCopyWith<$Res>  {
  factory $AssistActionInfoCopyWith(AssistActionInfo value, $Res Function(AssistActionInfo) _then) = _$AssistActionInfoCopyWithImpl;
@useResult
$Res call({
 AssistAction action, String label, String noun, bool replaces, bool needsSelection
});




}
/// @nodoc
class _$AssistActionInfoCopyWithImpl<$Res>
    implements $AssistActionInfoCopyWith<$Res> {
  _$AssistActionInfoCopyWithImpl(this._self, this._then);

  final AssistActionInfo _self;
  final $Res Function(AssistActionInfo) _then;

/// Create a copy of AssistActionInfo
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? action = null,Object? label = null,Object? noun = null,Object? replaces = null,Object? needsSelection = null,}) {
  return _then(_self.copyWith(
action: null == action ? _self.action : action // ignore: cast_nullable_to_non_nullable
as AssistAction,label: null == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String,noun: null == noun ? _self.noun : noun // ignore: cast_nullable_to_non_nullable
as String,replaces: null == replaces ? _self.replaces : replaces // ignore: cast_nullable_to_non_nullable
as bool,needsSelection: null == needsSelection ? _self.needsSelection : needsSelection // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [AssistActionInfo].
extension AssistActionInfoPatterns on AssistActionInfo {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AssistActionInfo value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AssistActionInfo() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AssistActionInfo value)  $default,){
final _that = this;
switch (_that) {
case _AssistActionInfo():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AssistActionInfo value)?  $default,){
final _that = this;
switch (_that) {
case _AssistActionInfo() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( AssistAction action,  String label,  String noun,  bool replaces,  bool needsSelection)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AssistActionInfo() when $default != null:
return $default(_that.action,_that.label,_that.noun,_that.replaces,_that.needsSelection);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( AssistAction action,  String label,  String noun,  bool replaces,  bool needsSelection)  $default,) {final _that = this;
switch (_that) {
case _AssistActionInfo():
return $default(_that.action,_that.label,_that.noun,_that.replaces,_that.needsSelection);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( AssistAction action,  String label,  String noun,  bool replaces,  bool needsSelection)?  $default,) {final _that = this;
switch (_that) {
case _AssistActionInfo() when $default != null:
return $default(_that.action,_that.label,_that.noun,_that.replaces,_that.needsSelection);case _:
  return null;

}
}

}

/// @nodoc


class _AssistActionInfo implements AssistActionInfo {
  const _AssistActionInfo({required this.action, required this.label, required this.noun, required this.replaces, this.needsSelection = true});
  

@override final  AssistAction action;
/// The menu label: `Improve Writing`.
@override final  String label;
/// What the suggestion is called once it is made: `Improved`.
@override final  String noun;
/// Whether accepting replaces the selection, or else the suggestion goes
/// after it.
@override final  bool replaces;
/// Whether the action needs text selected; Continue Writing does not.
@override@JsonKey() final  bool needsSelection;

/// Create a copy of AssistActionInfo
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AssistActionInfoCopyWith<_AssistActionInfo> get copyWith => __$AssistActionInfoCopyWithImpl<_AssistActionInfo>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AssistActionInfo&&(identical(other.action, action) || other.action == action)&&(identical(other.label, label) || other.label == label)&&(identical(other.noun, noun) || other.noun == noun)&&(identical(other.replaces, replaces) || other.replaces == replaces)&&(identical(other.needsSelection, needsSelection) || other.needsSelection == needsSelection));
}


@override
int get hashCode => Object.hash(runtimeType,action,label,noun,replaces,needsSelection);

@override
String toString() {
  return 'AssistActionInfo(action: $action, label: $label, noun: $noun, replaces: $replaces, needsSelection: $needsSelection)';
}


}

/// @nodoc
abstract mixin class _$AssistActionInfoCopyWith<$Res> implements $AssistActionInfoCopyWith<$Res> {
  factory _$AssistActionInfoCopyWith(_AssistActionInfo value, $Res Function(_AssistActionInfo) _then) = __$AssistActionInfoCopyWithImpl;
@override @useResult
$Res call({
 AssistAction action, String label, String noun, bool replaces, bool needsSelection
});




}
/// @nodoc
class __$AssistActionInfoCopyWithImpl<$Res>
    implements _$AssistActionInfoCopyWith<$Res> {
  __$AssistActionInfoCopyWithImpl(this._self, this._then);

  final _AssistActionInfo _self;
  final $Res Function(_AssistActionInfo) _then;

/// Create a copy of AssistActionInfo
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? action = null,Object? label = null,Object? noun = null,Object? replaces = null,Object? needsSelection = null,}) {
  return _then(_AssistActionInfo(
action: null == action ? _self.action : action // ignore: cast_nullable_to_non_nullable
as AssistAction,label: null == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String,noun: null == noun ? _self.noun : noun // ignore: cast_nullable_to_non_nullable
as String,replaces: null == replaces ? _self.replaces : replaces // ignore: cast_nullable_to_non_nullable
as bool,needsSelection: null == needsSelection ? _self.needsSelection : needsSelection // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
