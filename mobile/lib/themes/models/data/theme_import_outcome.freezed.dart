// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'theme_import_outcome.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ThemeImportOutcome {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ThemeImportOutcome);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ThemeImportOutcome()';
}


}

/// @nodoc
class $ThemeImportOutcomeCopyWith<$Res>  {
$ThemeImportOutcomeCopyWith(ThemeImportOutcome _, $Res Function(ThemeImportOutcome) __);
}


/// Adds pattern-matching-related methods to [ThemeImportOutcome].
extension ThemeImportOutcomePatterns on ThemeImportOutcome {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ThemeImportedOutcome value)?  imported,TResult Function( ThemeImportFailedOutcome value)?  failed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ThemeImportedOutcome() when imported != null:
return imported(_that);case ThemeImportFailedOutcome() when failed != null:
return failed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ThemeImportedOutcome value)  imported,required TResult Function( ThemeImportFailedOutcome value)  failed,}){
final _that = this;
switch (_that) {
case ThemeImportedOutcome():
return imported(_that);case ThemeImportFailedOutcome():
return failed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ThemeImportedOutcome value)?  imported,TResult? Function( ThemeImportFailedOutcome value)?  failed,}){
final _that = this;
switch (_that) {
case ThemeImportedOutcome() when imported != null:
return imported(_that);case ThemeImportFailedOutcome() when failed != null:
return failed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( List<String> names,  String summary)?  imported,TResult Function( String message)?  failed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ThemeImportedOutcome() when imported != null:
return imported(_that.names,_that.summary);case ThemeImportFailedOutcome() when failed != null:
return failed(_that.message);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( List<String> names,  String summary)  imported,required TResult Function( String message)  failed,}) {final _that = this;
switch (_that) {
case ThemeImportedOutcome():
return imported(_that.names,_that.summary);case ThemeImportFailedOutcome():
return failed(_that.message);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( List<String> names,  String summary)?  imported,TResult? Function( String message)?  failed,}) {final _that = this;
switch (_that) {
case ThemeImportedOutcome() when imported != null:
return imported(_that.names,_that.summary);case ThemeImportFailedOutcome() when failed != null:
return failed(_that.message);case _:
  return null;

}
}

}

/// @nodoc


class ThemeImportedOutcome implements ThemeImportOutcome {
  const ThemeImportedOutcome({required final  List<String> names, this.summary = ''}): _names = names;
  

 final  List<String> _names;
 List<String> get names {
  if (_names is EqualUnmodifiableListView) return _names;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_names);
}

@JsonKey() final  String summary;

/// Create a copy of ThemeImportOutcome
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ThemeImportedOutcomeCopyWith<ThemeImportedOutcome> get copyWith => _$ThemeImportedOutcomeCopyWithImpl<ThemeImportedOutcome>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ThemeImportedOutcome&&const DeepCollectionEquality().equals(other._names, _names)&&(identical(other.summary, summary) || other.summary == summary));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_names),summary);

@override
String toString() {
  return 'ThemeImportOutcome.imported(names: $names, summary: $summary)';
}


}

/// @nodoc
abstract mixin class $ThemeImportedOutcomeCopyWith<$Res> implements $ThemeImportOutcomeCopyWith<$Res> {
  factory $ThemeImportedOutcomeCopyWith(ThemeImportedOutcome value, $Res Function(ThemeImportedOutcome) _then) = _$ThemeImportedOutcomeCopyWithImpl;
@useResult
$Res call({
 List<String> names, String summary
});




}
/// @nodoc
class _$ThemeImportedOutcomeCopyWithImpl<$Res>
    implements $ThemeImportedOutcomeCopyWith<$Res> {
  _$ThemeImportedOutcomeCopyWithImpl(this._self, this._then);

  final ThemeImportedOutcome _self;
  final $Res Function(ThemeImportedOutcome) _then;

/// Create a copy of ThemeImportOutcome
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? names = null,Object? summary = null,}) {
  return _then(ThemeImportedOutcome(
names: null == names ? _self._names : names // ignore: cast_nullable_to_non_nullable
as List<String>,summary: null == summary ? _self.summary : summary // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ThemeImportFailedOutcome implements ThemeImportOutcome {
  const ThemeImportFailedOutcome({required this.message});
  

 final  String message;

/// Create a copy of ThemeImportOutcome
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ThemeImportFailedOutcomeCopyWith<ThemeImportFailedOutcome> get copyWith => _$ThemeImportFailedOutcomeCopyWithImpl<ThemeImportFailedOutcome>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ThemeImportFailedOutcome&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'ThemeImportOutcome.failed(message: $message)';
}


}

/// @nodoc
abstract mixin class $ThemeImportFailedOutcomeCopyWith<$Res> implements $ThemeImportOutcomeCopyWith<$Res> {
  factory $ThemeImportFailedOutcomeCopyWith(ThemeImportFailedOutcome value, $Res Function(ThemeImportFailedOutcome) _then) = _$ThemeImportFailedOutcomeCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$ThemeImportFailedOutcomeCopyWithImpl<$Res>
    implements $ThemeImportFailedOutcomeCopyWith<$Res> {
  _$ThemeImportFailedOutcomeCopyWithImpl(this._self, this._then);

  final ThemeImportFailedOutcome _self;
  final $Res Function(ThemeImportFailedOutcome) _then;

/// Create a copy of ThemeImportOutcome
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(ThemeImportFailedOutcome(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
