// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'gallery_problem.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$GalleryProblem {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is GalleryProblem);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'GalleryProblem()';
}


}

/// @nodoc
class $GalleryProblemCopyWith<$Res>  {
$GalleryProblemCopyWith(GalleryProblem _, $Res Function(GalleryProblem) __);
}


/// Adds pattern-matching-related methods to [GalleryProblem].
extension GalleryProblemPatterns on GalleryProblem {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( GalleryProblemWords value)?  words,TResult Function( GalleryProblemOffline value)?  offline,TResult Function( GalleryProblemUnexpected value)?  unexpected,required TResult orElse(),}){
final _that = this;
switch (_that) {
case GalleryProblemWords() when words != null:
return words(_that);case GalleryProblemOffline() when offline != null:
return offline(_that);case GalleryProblemUnexpected() when unexpected != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( GalleryProblemWords value)  words,required TResult Function( GalleryProblemOffline value)  offline,required TResult Function( GalleryProblemUnexpected value)  unexpected,}){
final _that = this;
switch (_that) {
case GalleryProblemWords():
return words(_that);case GalleryProblemOffline():
return offline(_that);case GalleryProblemUnexpected():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( GalleryProblemWords value)?  words,TResult? Function( GalleryProblemOffline value)?  offline,TResult? Function( GalleryProblemUnexpected value)?  unexpected,}){
final _that = this;
switch (_that) {
case GalleryProblemWords() when words != null:
return words(_that);case GalleryProblemOffline() when offline != null:
return offline(_that);case GalleryProblemUnexpected() when unexpected != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String message)?  words,TResult Function()?  offline,TResult Function()?  unexpected,required TResult orElse(),}) {final _that = this;
switch (_that) {
case GalleryProblemWords() when words != null:
return words(_that.message);case GalleryProblemOffline() when offline != null:
return offline();case GalleryProblemUnexpected() when unexpected != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String message)  words,required TResult Function()  offline,required TResult Function()  unexpected,}) {final _that = this;
switch (_that) {
case GalleryProblemWords():
return words(_that.message);case GalleryProblemOffline():
return offline();case GalleryProblemUnexpected():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String message)?  words,TResult? Function()?  offline,TResult? Function()?  unexpected,}) {final _that = this;
switch (_that) {
case GalleryProblemWords() when words != null:
return words(_that.message);case GalleryProblemOffline() when offline != null:
return offline();case GalleryProblemUnexpected() when unexpected != null:
return unexpected();case _:
  return null;

}
}

}

/// @nodoc


class GalleryProblemWords implements GalleryProblem {
  const GalleryProblemWords(this.message);
  

 final  String message;

/// Create a copy of GalleryProblem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$GalleryProblemWordsCopyWith<GalleryProblemWords> get copyWith => _$GalleryProblemWordsCopyWithImpl<GalleryProblemWords>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is GalleryProblemWords&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'GalleryProblem.words(message: $message)';
}


}

/// @nodoc
abstract mixin class $GalleryProblemWordsCopyWith<$Res> implements $GalleryProblemCopyWith<$Res> {
  factory $GalleryProblemWordsCopyWith(GalleryProblemWords value, $Res Function(GalleryProblemWords) _then) = _$GalleryProblemWordsCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$GalleryProblemWordsCopyWithImpl<$Res>
    implements $GalleryProblemWordsCopyWith<$Res> {
  _$GalleryProblemWordsCopyWithImpl(this._self, this._then);

  final GalleryProblemWords _self;
  final $Res Function(GalleryProblemWords) _then;

/// Create a copy of GalleryProblem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(GalleryProblemWords(
null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class GalleryProblemOffline implements GalleryProblem {
  const GalleryProblemOffline();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is GalleryProblemOffline);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'GalleryProblem.offline()';
}


}




/// @nodoc


class GalleryProblemUnexpected implements GalleryProblem {
  const GalleryProblemUnexpected();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is GalleryProblemUnexpected);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'GalleryProblem.unexpected()';
}


}




// dart format on
