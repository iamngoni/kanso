// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'share_problem.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ShareProblem {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareProblem);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareProblem()';
}


}

/// @nodoc
class $ShareProblemCopyWith<$Res>  {
$ShareProblemCopyWith(ShareProblem _, $Res Function(ShareProblem) __);
}


/// Adds pattern-matching-related methods to [ShareProblem].
extension ShareProblemPatterns on ShareProblem {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ShareProblemWords value)?  words,TResult Function( ShareProblemNameMissing value)?  nameMissing,TResult Function( ShareProblemLinkMissing value)?  linkMissing,TResult Function( ShareProblemOffline value)?  offline,TResult Function( ShareProblemSignedOut value)?  signedOut,TResult Function( ShareProblemUnexpected value)?  unexpected,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ShareProblemWords() when words != null:
return words(_that);case ShareProblemNameMissing() when nameMissing != null:
return nameMissing(_that);case ShareProblemLinkMissing() when linkMissing != null:
return linkMissing(_that);case ShareProblemOffline() when offline != null:
return offline(_that);case ShareProblemSignedOut() when signedOut != null:
return signedOut(_that);case ShareProblemUnexpected() when unexpected != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ShareProblemWords value)  words,required TResult Function( ShareProblemNameMissing value)  nameMissing,required TResult Function( ShareProblemLinkMissing value)  linkMissing,required TResult Function( ShareProblemOffline value)  offline,required TResult Function( ShareProblemSignedOut value)  signedOut,required TResult Function( ShareProblemUnexpected value)  unexpected,}){
final _that = this;
switch (_that) {
case ShareProblemWords():
return words(_that);case ShareProblemNameMissing():
return nameMissing(_that);case ShareProblemLinkMissing():
return linkMissing(_that);case ShareProblemOffline():
return offline(_that);case ShareProblemSignedOut():
return signedOut(_that);case ShareProblemUnexpected():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ShareProblemWords value)?  words,TResult? Function( ShareProblemNameMissing value)?  nameMissing,TResult? Function( ShareProblemLinkMissing value)?  linkMissing,TResult? Function( ShareProblemOffline value)?  offline,TResult? Function( ShareProblemSignedOut value)?  signedOut,TResult? Function( ShareProblemUnexpected value)?  unexpected,}){
final _that = this;
switch (_that) {
case ShareProblemWords() when words != null:
return words(_that);case ShareProblemNameMissing() when nameMissing != null:
return nameMissing(_that);case ShareProblemLinkMissing() when linkMissing != null:
return linkMissing(_that);case ShareProblemOffline() when offline != null:
return offline(_that);case ShareProblemSignedOut() when signedOut != null:
return signedOut(_that);case ShareProblemUnexpected() when unexpected != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String message)?  words,TResult Function()?  nameMissing,TResult Function()?  linkMissing,TResult Function()?  offline,TResult Function()?  signedOut,TResult Function()?  unexpected,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ShareProblemWords() when words != null:
return words(_that.message);case ShareProblemNameMissing() when nameMissing != null:
return nameMissing();case ShareProblemLinkMissing() when linkMissing != null:
return linkMissing();case ShareProblemOffline() when offline != null:
return offline();case ShareProblemSignedOut() when signedOut != null:
return signedOut();case ShareProblemUnexpected() when unexpected != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String message)  words,required TResult Function()  nameMissing,required TResult Function()  linkMissing,required TResult Function()  offline,required TResult Function()  signedOut,required TResult Function()  unexpected,}) {final _that = this;
switch (_that) {
case ShareProblemWords():
return words(_that.message);case ShareProblemNameMissing():
return nameMissing();case ShareProblemLinkMissing():
return linkMissing();case ShareProblemOffline():
return offline();case ShareProblemSignedOut():
return signedOut();case ShareProblemUnexpected():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String message)?  words,TResult? Function()?  nameMissing,TResult? Function()?  linkMissing,TResult? Function()?  offline,TResult? Function()?  signedOut,TResult? Function()?  unexpected,}) {final _that = this;
switch (_that) {
case ShareProblemWords() when words != null:
return words(_that.message);case ShareProblemNameMissing() when nameMissing != null:
return nameMissing();case ShareProblemLinkMissing() when linkMissing != null:
return linkMissing();case ShareProblemOffline() when offline != null:
return offline();case ShareProblemSignedOut() when signedOut != null:
return signedOut();case ShareProblemUnexpected() when unexpected != null:
return unexpected();case _:
  return null;

}
}

}

/// @nodoc


class ShareProblemWords implements ShareProblem {
  const ShareProblemWords(this.message);
  

 final  String message;

/// Create a copy of ShareProblem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ShareProblemWordsCopyWith<ShareProblemWords> get copyWith => _$ShareProblemWordsCopyWithImpl<ShareProblemWords>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareProblemWords&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'ShareProblem.words(message: $message)';
}


}

/// @nodoc
abstract mixin class $ShareProblemWordsCopyWith<$Res> implements $ShareProblemCopyWith<$Res> {
  factory $ShareProblemWordsCopyWith(ShareProblemWords value, $Res Function(ShareProblemWords) _then) = _$ShareProblemWordsCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$ShareProblemWordsCopyWithImpl<$Res>
    implements $ShareProblemWordsCopyWith<$Res> {
  _$ShareProblemWordsCopyWithImpl(this._self, this._then);

  final ShareProblemWords _self;
  final $Res Function(ShareProblemWords) _then;

/// Create a copy of ShareProblem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(ShareProblemWords(
null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ShareProblemNameMissing implements ShareProblem {
  const ShareProblemNameMissing();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareProblemNameMissing);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareProblem.nameMissing()';
}


}




/// @nodoc


class ShareProblemLinkMissing implements ShareProblem {
  const ShareProblemLinkMissing();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareProblemLinkMissing);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareProblem.linkMissing()';
}


}




/// @nodoc


class ShareProblemOffline implements ShareProblem {
  const ShareProblemOffline();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareProblemOffline);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareProblem.offline()';
}


}




/// @nodoc


class ShareProblemSignedOut implements ShareProblem {
  const ShareProblemSignedOut();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareProblemSignedOut);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareProblem.signedOut()';
}


}




/// @nodoc


class ShareProblemUnexpected implements ShareProblem {
  const ShareProblemUnexpected();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareProblemUnexpected);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareProblem.unexpected()';
}


}




// dart format on
