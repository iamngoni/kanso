// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'assist_update.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AssistUpdate {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AssistUpdate);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AssistUpdate()';
}


}

/// @nodoc
class $AssistUpdateCopyWith<$Res>  {
$AssistUpdateCopyWith(AssistUpdate _, $Res Function(AssistUpdate) __);
}


/// Adds pattern-matching-related methods to [AssistUpdate].
extension AssistUpdatePatterns on AssistUpdate {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AssistText value)?  text,TResult Function( AssistDone value)?  done,TResult Function( AssistKeptOut value)?  keptOut,TResult Function( AssistFailed value)?  error,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AssistText() when text != null:
return text(_that);case AssistDone() when done != null:
return done(_that);case AssistKeptOut() when keptOut != null:
return keptOut(_that);case AssistFailed() when error != null:
return error(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AssistText value)  text,required TResult Function( AssistDone value)  done,required TResult Function( AssistKeptOut value)  keptOut,required TResult Function( AssistFailed value)  error,}){
final _that = this;
switch (_that) {
case AssistText():
return text(_that);case AssistDone():
return done(_that);case AssistKeptOut():
return keptOut(_that);case AssistFailed():
return error(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AssistText value)?  text,TResult? Function( AssistDone value)?  done,TResult? Function( AssistKeptOut value)?  keptOut,TResult? Function( AssistFailed value)?  error,}){
final _that = this;
switch (_that) {
case AssistText() when text != null:
return text(_that);case AssistDone() when done != null:
return done(_that);case AssistKeptOut() when keptOut != null:
return keptOut(_that);case AssistFailed() when error != null:
return error(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String text)?  text,TResult Function( String suggestion)?  done,TResult Function()?  keptOut,TResult Function( String message)?  error,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AssistText() when text != null:
return text(_that.text);case AssistDone() when done != null:
return done(_that.suggestion);case AssistKeptOut() when keptOut != null:
return keptOut();case AssistFailed() when error != null:
return error(_that.message);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String text)  text,required TResult Function( String suggestion)  done,required TResult Function()  keptOut,required TResult Function( String message)  error,}) {final _that = this;
switch (_that) {
case AssistText():
return text(_that.text);case AssistDone():
return done(_that.suggestion);case AssistKeptOut():
return keptOut();case AssistFailed():
return error(_that.message);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String text)?  text,TResult? Function( String suggestion)?  done,TResult? Function()?  keptOut,TResult? Function( String message)?  error,}) {final _that = this;
switch (_that) {
case AssistText() when text != null:
return text(_that.text);case AssistDone() when done != null:
return done(_that.suggestion);case AssistKeptOut() when keptOut != null:
return keptOut();case AssistFailed() when error != null:
return error(_that.message);case _:
  return null;

}
}

}

/// @nodoc


class AssistText implements AssistUpdate {
  const AssistText({required this.text});
  

 final  String text;

/// Create a copy of AssistUpdate
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AssistTextCopyWith<AssistText> get copyWith => _$AssistTextCopyWithImpl<AssistText>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AssistText&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,text);

@override
String toString() {
  return 'AssistUpdate.text(text: $text)';
}


}

/// @nodoc
abstract mixin class $AssistTextCopyWith<$Res> implements $AssistUpdateCopyWith<$Res> {
  factory $AssistTextCopyWith(AssistText value, $Res Function(AssistText) _then) = _$AssistTextCopyWithImpl;
@useResult
$Res call({
 String text
});




}
/// @nodoc
class _$AssistTextCopyWithImpl<$Res>
    implements $AssistTextCopyWith<$Res> {
  _$AssistTextCopyWithImpl(this._self, this._then);

  final AssistText _self;
  final $Res Function(AssistText) _then;

/// Create a copy of AssistUpdate
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? text = null,}) {
  return _then(AssistText(
text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AssistDone implements AssistUpdate {
  const AssistDone({required this.suggestion});
  

 final  String suggestion;

/// Create a copy of AssistUpdate
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AssistDoneCopyWith<AssistDone> get copyWith => _$AssistDoneCopyWithImpl<AssistDone>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AssistDone&&(identical(other.suggestion, suggestion) || other.suggestion == suggestion));
}


@override
int get hashCode => Object.hash(runtimeType,suggestion);

@override
String toString() {
  return 'AssistUpdate.done(suggestion: $suggestion)';
}


}

/// @nodoc
abstract mixin class $AssistDoneCopyWith<$Res> implements $AssistUpdateCopyWith<$Res> {
  factory $AssistDoneCopyWith(AssistDone value, $Res Function(AssistDone) _then) = _$AssistDoneCopyWithImpl;
@useResult
$Res call({
 String suggestion
});




}
/// @nodoc
class _$AssistDoneCopyWithImpl<$Res>
    implements $AssistDoneCopyWith<$Res> {
  _$AssistDoneCopyWithImpl(this._self, this._then);

  final AssistDone _self;
  final $Res Function(AssistDone) _then;

/// Create a copy of AssistUpdate
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? suggestion = null,}) {
  return _then(AssistDone(
suggestion: null == suggestion ? _self.suggestion : suggestion // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AssistKeptOut implements AssistUpdate {
  const AssistKeptOut();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AssistKeptOut);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AssistUpdate.keptOut()';
}


}




/// @nodoc


class AssistFailed implements AssistUpdate {
  const AssistFailed({required this.message});
  

 final  String message;

/// Create a copy of AssistUpdate
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AssistFailedCopyWith<AssistFailed> get copyWith => _$AssistFailedCopyWithImpl<AssistFailed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AssistFailed&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'AssistUpdate.error(message: $message)';
}


}

/// @nodoc
abstract mixin class $AssistFailedCopyWith<$Res> implements $AssistUpdateCopyWith<$Res> {
  factory $AssistFailedCopyWith(AssistFailed value, $Res Function(AssistFailed) _then) = _$AssistFailedCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$AssistFailedCopyWithImpl<$Res>
    implements $AssistFailedCopyWith<$Res> {
  _$AssistFailedCopyWithImpl(this._self, this._then);

  final AssistFailed _self;
  final $Res Function(AssistFailed) _then;

/// Create a copy of AssistUpdate
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(AssistFailed(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
