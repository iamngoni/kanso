// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'import_problem.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ImportProblem {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ImportProblem);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ImportProblem()';
}


}

/// @nodoc
class $ImportProblemCopyWith<$Res>  {
$ImportProblemCopyWith(ImportProblem _, $Res Function(ImportProblem) __);
}


/// Adds pattern-matching-related methods to [ImportProblem].
extension ImportProblemPatterns on ImportProblem {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ImportProblemWords value)?  words,TResult Function( ImportProblemPickFailed value)?  pickFailed,TResult Function( ImportProblemNotAnExport value)?  notAnExport,TResult Function( ImportProblemUnexpected value)?  unexpected,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ImportProblemWords() when words != null:
return words(_that);case ImportProblemPickFailed() when pickFailed != null:
return pickFailed(_that);case ImportProblemNotAnExport() when notAnExport != null:
return notAnExport(_that);case ImportProblemUnexpected() when unexpected != null:
return unexpected(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ImportProblemWords value)  words,required TResult Function( ImportProblemPickFailed value)  pickFailed,required TResult Function( ImportProblemNotAnExport value)  notAnExport,required TResult Function( ImportProblemUnexpected value)  unexpected,}){
final _that = this;
switch (_that) {
case ImportProblemWords():
return words(_that);case ImportProblemPickFailed():
return pickFailed(_that);case ImportProblemNotAnExport():
return notAnExport(_that);case ImportProblemUnexpected():
return unexpected(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ImportProblemWords value)?  words,TResult? Function( ImportProblemPickFailed value)?  pickFailed,TResult? Function( ImportProblemNotAnExport value)?  notAnExport,TResult? Function( ImportProblemUnexpected value)?  unexpected,}){
final _that = this;
switch (_that) {
case ImportProblemWords() when words != null:
return words(_that);case ImportProblemPickFailed() when pickFailed != null:
return pickFailed(_that);case ImportProblemNotAnExport() when notAnExport != null:
return notAnExport(_that);case ImportProblemUnexpected() when unexpected != null:
return unexpected(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String message)?  words,TResult Function()?  pickFailed,TResult Function()?  notAnExport,TResult Function()?  unexpected,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ImportProblemWords() when words != null:
return words(_that.message);case ImportProblemPickFailed() when pickFailed != null:
return pickFailed();case ImportProblemNotAnExport() when notAnExport != null:
return notAnExport();case ImportProblemUnexpected() when unexpected != null:
return unexpected();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String message)  words,required TResult Function()  pickFailed,required TResult Function()  notAnExport,required TResult Function()  unexpected,}) {final _that = this;
switch (_that) {
case ImportProblemWords():
return words(_that.message);case ImportProblemPickFailed():
return pickFailed();case ImportProblemNotAnExport():
return notAnExport();case ImportProblemUnexpected():
return unexpected();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String message)?  words,TResult? Function()?  pickFailed,TResult? Function()?  notAnExport,TResult? Function()?  unexpected,}) {final _that = this;
switch (_that) {
case ImportProblemWords() when words != null:
return words(_that.message);case ImportProblemPickFailed() when pickFailed != null:
return pickFailed();case ImportProblemNotAnExport() when notAnExport != null:
return notAnExport();case ImportProblemUnexpected() when unexpected != null:
return unexpected();case _:
  return null;

}
}

}

/// @nodoc


class ImportProblemWords implements ImportProblem {
  const ImportProblemWords(this.message);
  

 final  String message;

/// Create a copy of ImportProblem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ImportProblemWordsCopyWith<ImportProblemWords> get copyWith => _$ImportProblemWordsCopyWithImpl<ImportProblemWords>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ImportProblemWords&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'ImportProblem.words(message: $message)';
}


}

/// @nodoc
abstract mixin class $ImportProblemWordsCopyWith<$Res> implements $ImportProblemCopyWith<$Res> {
  factory $ImportProblemWordsCopyWith(ImportProblemWords value, $Res Function(ImportProblemWords) _then) = _$ImportProblemWordsCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$ImportProblemWordsCopyWithImpl<$Res>
    implements $ImportProblemWordsCopyWith<$Res> {
  _$ImportProblemWordsCopyWithImpl(this._self, this._then);

  final ImportProblemWords _self;
  final $Res Function(ImportProblemWords) _then;

/// Create a copy of ImportProblem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(ImportProblemWords(
null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ImportProblemPickFailed implements ImportProblem {
  const ImportProblemPickFailed();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ImportProblemPickFailed);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ImportProblem.pickFailed()';
}


}




/// @nodoc


class ImportProblemNotAnExport implements ImportProblem {
  const ImportProblemNotAnExport();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ImportProblemNotAnExport);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ImportProblem.notAnExport()';
}


}




/// @nodoc


class ImportProblemUnexpected implements ImportProblem {
  const ImportProblemUnexpected();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ImportProblemUnexpected);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ImportProblem.unexpected()';
}


}




// dart format on
