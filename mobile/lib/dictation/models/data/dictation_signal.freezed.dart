// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'dictation_signal.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$DictationSignal {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DictationSignal);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DictationSignal()';
}


}

/// @nodoc
class $DictationSignalCopyWith<$Res>  {
$DictationSignalCopyWith(DictationSignal _, $Res Function(DictationSignal) __);
}


/// Adds pattern-matching-related methods to [DictationSignal].
extension DictationSignalPatterns on DictationSignal {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( DictationListeningSignal value)?  listening,TResult Function( DictationPartialSignal value)?  partial,TResult Function( DictationFinalSignal value)?  finalText,TResult Function( DictationLevelSignal value)?  level,TResult Function( DictationFailedSignal value)?  failed,TResult Function( DictationEndedSignal value)?  ended,required TResult orElse(),}){
final _that = this;
switch (_that) {
case DictationListeningSignal() when listening != null:
return listening(_that);case DictationPartialSignal() when partial != null:
return partial(_that);case DictationFinalSignal() when finalText != null:
return finalText(_that);case DictationLevelSignal() when level != null:
return level(_that);case DictationFailedSignal() when failed != null:
return failed(_that);case DictationEndedSignal() when ended != null:
return ended(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( DictationListeningSignal value)  listening,required TResult Function( DictationPartialSignal value)  partial,required TResult Function( DictationFinalSignal value)  finalText,required TResult Function( DictationLevelSignal value)  level,required TResult Function( DictationFailedSignal value)  failed,required TResult Function( DictationEndedSignal value)  ended,}){
final _that = this;
switch (_that) {
case DictationListeningSignal():
return listening(_that);case DictationPartialSignal():
return partial(_that);case DictationFinalSignal():
return finalText(_that);case DictationLevelSignal():
return level(_that);case DictationFailedSignal():
return failed(_that);case DictationEndedSignal():
return ended(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( DictationListeningSignal value)?  listening,TResult? Function( DictationPartialSignal value)?  partial,TResult? Function( DictationFinalSignal value)?  finalText,TResult? Function( DictationLevelSignal value)?  level,TResult? Function( DictationFailedSignal value)?  failed,TResult? Function( DictationEndedSignal value)?  ended,}){
final _that = this;
switch (_that) {
case DictationListeningSignal() when listening != null:
return listening(_that);case DictationPartialSignal() when partial != null:
return partial(_that);case DictationFinalSignal() when finalText != null:
return finalText(_that);case DictationLevelSignal() when level != null:
return level(_that);case DictationFailedSignal() when failed != null:
return failed(_that);case DictationEndedSignal() when ended != null:
return ended(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  listening,TResult Function( String text)?  partial,TResult Function( String text)?  finalText,TResult Function( double level)?  level,TResult Function( DictationFailure failure)?  failed,TResult Function()?  ended,required TResult orElse(),}) {final _that = this;
switch (_that) {
case DictationListeningSignal() when listening != null:
return listening();case DictationPartialSignal() when partial != null:
return partial(_that.text);case DictationFinalSignal() when finalText != null:
return finalText(_that.text);case DictationLevelSignal() when level != null:
return level(_that.level);case DictationFailedSignal() when failed != null:
return failed(_that.failure);case DictationEndedSignal() when ended != null:
return ended();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  listening,required TResult Function( String text)  partial,required TResult Function( String text)  finalText,required TResult Function( double level)  level,required TResult Function( DictationFailure failure)  failed,required TResult Function()  ended,}) {final _that = this;
switch (_that) {
case DictationListeningSignal():
return listening();case DictationPartialSignal():
return partial(_that.text);case DictationFinalSignal():
return finalText(_that.text);case DictationLevelSignal():
return level(_that.level);case DictationFailedSignal():
return failed(_that.failure);case DictationEndedSignal():
return ended();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  listening,TResult? Function( String text)?  partial,TResult? Function( String text)?  finalText,TResult? Function( double level)?  level,TResult? Function( DictationFailure failure)?  failed,TResult? Function()?  ended,}) {final _that = this;
switch (_that) {
case DictationListeningSignal() when listening != null:
return listening();case DictationPartialSignal() when partial != null:
return partial(_that.text);case DictationFinalSignal() when finalText != null:
return finalText(_that.text);case DictationLevelSignal() when level != null:
return level(_that.level);case DictationFailedSignal() when failed != null:
return failed(_that.failure);case DictationEndedSignal() when ended != null:
return ended();case _:
  return null;

}
}

}

/// @nodoc


class DictationListeningSignal implements DictationSignal {
  const DictationListeningSignal();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DictationListeningSignal);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DictationSignal.listening()';
}


}




/// @nodoc


class DictationPartialSignal implements DictationSignal {
  const DictationPartialSignal(this.text);
  

 final  String text;

/// Create a copy of DictationSignal
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DictationPartialSignalCopyWith<DictationPartialSignal> get copyWith => _$DictationPartialSignalCopyWithImpl<DictationPartialSignal>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DictationPartialSignal&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,text);

@override
String toString() {
  return 'DictationSignal.partial(text: $text)';
}


}

/// @nodoc
abstract mixin class $DictationPartialSignalCopyWith<$Res> implements $DictationSignalCopyWith<$Res> {
  factory $DictationPartialSignalCopyWith(DictationPartialSignal value, $Res Function(DictationPartialSignal) _then) = _$DictationPartialSignalCopyWithImpl;
@useResult
$Res call({
 String text
});




}
/// @nodoc
class _$DictationPartialSignalCopyWithImpl<$Res>
    implements $DictationPartialSignalCopyWith<$Res> {
  _$DictationPartialSignalCopyWithImpl(this._self, this._then);

  final DictationPartialSignal _self;
  final $Res Function(DictationPartialSignal) _then;

/// Create a copy of DictationSignal
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? text = null,}) {
  return _then(DictationPartialSignal(
null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class DictationFinalSignal implements DictationSignal {
  const DictationFinalSignal(this.text);
  

 final  String text;

/// Create a copy of DictationSignal
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DictationFinalSignalCopyWith<DictationFinalSignal> get copyWith => _$DictationFinalSignalCopyWithImpl<DictationFinalSignal>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DictationFinalSignal&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,text);

@override
String toString() {
  return 'DictationSignal.finalText(text: $text)';
}


}

/// @nodoc
abstract mixin class $DictationFinalSignalCopyWith<$Res> implements $DictationSignalCopyWith<$Res> {
  factory $DictationFinalSignalCopyWith(DictationFinalSignal value, $Res Function(DictationFinalSignal) _then) = _$DictationFinalSignalCopyWithImpl;
@useResult
$Res call({
 String text
});




}
/// @nodoc
class _$DictationFinalSignalCopyWithImpl<$Res>
    implements $DictationFinalSignalCopyWith<$Res> {
  _$DictationFinalSignalCopyWithImpl(this._self, this._then);

  final DictationFinalSignal _self;
  final $Res Function(DictationFinalSignal) _then;

/// Create a copy of DictationSignal
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? text = null,}) {
  return _then(DictationFinalSignal(
null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class DictationLevelSignal implements DictationSignal {
  const DictationLevelSignal(this.level);
  

 final  double level;

/// Create a copy of DictationSignal
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DictationLevelSignalCopyWith<DictationLevelSignal> get copyWith => _$DictationLevelSignalCopyWithImpl<DictationLevelSignal>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DictationLevelSignal&&(identical(other.level, level) || other.level == level));
}


@override
int get hashCode => Object.hash(runtimeType,level);

@override
String toString() {
  return 'DictationSignal.level(level: $level)';
}


}

/// @nodoc
abstract mixin class $DictationLevelSignalCopyWith<$Res> implements $DictationSignalCopyWith<$Res> {
  factory $DictationLevelSignalCopyWith(DictationLevelSignal value, $Res Function(DictationLevelSignal) _then) = _$DictationLevelSignalCopyWithImpl;
@useResult
$Res call({
 double level
});




}
/// @nodoc
class _$DictationLevelSignalCopyWithImpl<$Res>
    implements $DictationLevelSignalCopyWith<$Res> {
  _$DictationLevelSignalCopyWithImpl(this._self, this._then);

  final DictationLevelSignal _self;
  final $Res Function(DictationLevelSignal) _then;

/// Create a copy of DictationSignal
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? level = null,}) {
  return _then(DictationLevelSignal(
null == level ? _self.level : level // ignore: cast_nullable_to_non_nullable
as double,
  ));
}


}

/// @nodoc


class DictationFailedSignal implements DictationSignal {
  const DictationFailedSignal(this.failure);
  

 final  DictationFailure failure;

/// Create a copy of DictationSignal
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DictationFailedSignalCopyWith<DictationFailedSignal> get copyWith => _$DictationFailedSignalCopyWithImpl<DictationFailedSignal>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DictationFailedSignal&&(identical(other.failure, failure) || other.failure == failure));
}


@override
int get hashCode => Object.hash(runtimeType,failure);

@override
String toString() {
  return 'DictationSignal.failed(failure: $failure)';
}


}

/// @nodoc
abstract mixin class $DictationFailedSignalCopyWith<$Res> implements $DictationSignalCopyWith<$Res> {
  factory $DictationFailedSignalCopyWith(DictationFailedSignal value, $Res Function(DictationFailedSignal) _then) = _$DictationFailedSignalCopyWithImpl;
@useResult
$Res call({
 DictationFailure failure
});




}
/// @nodoc
class _$DictationFailedSignalCopyWithImpl<$Res>
    implements $DictationFailedSignalCopyWith<$Res> {
  _$DictationFailedSignalCopyWithImpl(this._self, this._then);

  final DictationFailedSignal _self;
  final $Res Function(DictationFailedSignal) _then;

/// Create a copy of DictationSignal
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? failure = null,}) {
  return _then(DictationFailedSignal(
null == failure ? _self.failure : failure // ignore: cast_nullable_to_non_nullable
as DictationFailure,
  ));
}


}

/// @nodoc


class DictationEndedSignal implements DictationSignal {
  const DictationEndedSignal();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DictationEndedSignal);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DictationSignal.ended()';
}


}




// dart format on
