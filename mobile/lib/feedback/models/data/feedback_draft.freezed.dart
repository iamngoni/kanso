// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'feedback_draft.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$FeedbackScreenshot {

 Uint8List get bytes;/// The file's name, for the row that shows it.
 String get name;
/// Create a copy of FeedbackScreenshot
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FeedbackScreenshotCopyWith<FeedbackScreenshot> get copyWith => _$FeedbackScreenshotCopyWithImpl<FeedbackScreenshot>(this as FeedbackScreenshot, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FeedbackScreenshot&&const DeepCollectionEquality().equals(other.bytes, bytes)&&(identical(other.name, name) || other.name == name));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(bytes),name);

@override
String toString() {
  return 'FeedbackScreenshot(bytes: $bytes, name: $name)';
}


}

/// @nodoc
abstract mixin class $FeedbackScreenshotCopyWith<$Res>  {
  factory $FeedbackScreenshotCopyWith(FeedbackScreenshot value, $Res Function(FeedbackScreenshot) _then) = _$FeedbackScreenshotCopyWithImpl;
@useResult
$Res call({
 Uint8List bytes, String name
});




}
/// @nodoc
class _$FeedbackScreenshotCopyWithImpl<$Res>
    implements $FeedbackScreenshotCopyWith<$Res> {
  _$FeedbackScreenshotCopyWithImpl(this._self, this._then);

  final FeedbackScreenshot _self;
  final $Res Function(FeedbackScreenshot) _then;

/// Create a copy of FeedbackScreenshot
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? bytes = null,Object? name = null,}) {
  return _then(_self.copyWith(
bytes: null == bytes ? _self.bytes : bytes // ignore: cast_nullable_to_non_nullable
as Uint8List,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [FeedbackScreenshot].
extension FeedbackScreenshotPatterns on FeedbackScreenshot {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _FeedbackScreenshot value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _FeedbackScreenshot() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _FeedbackScreenshot value)  $default,){
final _that = this;
switch (_that) {
case _FeedbackScreenshot():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _FeedbackScreenshot value)?  $default,){
final _that = this;
switch (_that) {
case _FeedbackScreenshot() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( Uint8List bytes,  String name)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _FeedbackScreenshot() when $default != null:
return $default(_that.bytes,_that.name);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( Uint8List bytes,  String name)  $default,) {final _that = this;
switch (_that) {
case _FeedbackScreenshot():
return $default(_that.bytes,_that.name);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( Uint8List bytes,  String name)?  $default,) {final _that = this;
switch (_that) {
case _FeedbackScreenshot() when $default != null:
return $default(_that.bytes,_that.name);case _:
  return null;

}
}

}

/// @nodoc


class _FeedbackScreenshot implements FeedbackScreenshot {
  const _FeedbackScreenshot({required this.bytes, required this.name});
  

@override final  Uint8List bytes;
/// The file's name, for the row that shows it.
@override final  String name;

/// Create a copy of FeedbackScreenshot
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$FeedbackScreenshotCopyWith<_FeedbackScreenshot> get copyWith => __$FeedbackScreenshotCopyWithImpl<_FeedbackScreenshot>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _FeedbackScreenshot&&const DeepCollectionEquality().equals(other.bytes, bytes)&&(identical(other.name, name) || other.name == name));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(bytes),name);

@override
String toString() {
  return 'FeedbackScreenshot(bytes: $bytes, name: $name)';
}


}

/// @nodoc
abstract mixin class _$FeedbackScreenshotCopyWith<$Res> implements $FeedbackScreenshotCopyWith<$Res> {
  factory _$FeedbackScreenshotCopyWith(_FeedbackScreenshot value, $Res Function(_FeedbackScreenshot) _then) = __$FeedbackScreenshotCopyWithImpl;
@override @useResult
$Res call({
 Uint8List bytes, String name
});




}
/// @nodoc
class __$FeedbackScreenshotCopyWithImpl<$Res>
    implements _$FeedbackScreenshotCopyWith<$Res> {
  __$FeedbackScreenshotCopyWithImpl(this._self, this._then);

  final _FeedbackScreenshot _self;
  final $Res Function(_FeedbackScreenshot) _then;

/// Create a copy of FeedbackScreenshot
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? bytes = null,Object? name = null,}) {
  return _then(_FeedbackScreenshot(
bytes: null == bytes ? _self.bytes : bytes // ignore: cast_nullable_to_non_nullable
as Uint8List,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$FeedbackDraft {

 FeedbackKind get kind; String get text;/// Whether "Credit me by name" is on.
 bool get credit; String get creditName; bool get includeApp; FeedbackScreenshot? get screenshot;
/// Create a copy of FeedbackDraft
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FeedbackDraftCopyWith<FeedbackDraft> get copyWith => _$FeedbackDraftCopyWithImpl<FeedbackDraft>(this as FeedbackDraft, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FeedbackDraft&&(identical(other.kind, kind) || other.kind == kind)&&(identical(other.text, text) || other.text == text)&&(identical(other.credit, credit) || other.credit == credit)&&(identical(other.creditName, creditName) || other.creditName == creditName)&&(identical(other.includeApp, includeApp) || other.includeApp == includeApp)&&(identical(other.screenshot, screenshot) || other.screenshot == screenshot));
}


@override
int get hashCode => Object.hash(runtimeType,kind,text,credit,creditName,includeApp,screenshot);

@override
String toString() {
  return 'FeedbackDraft(kind: $kind, text: $text, credit: $credit, creditName: $creditName, includeApp: $includeApp, screenshot: $screenshot)';
}


}

/// @nodoc
abstract mixin class $FeedbackDraftCopyWith<$Res>  {
  factory $FeedbackDraftCopyWith(FeedbackDraft value, $Res Function(FeedbackDraft) _then) = _$FeedbackDraftCopyWithImpl;
@useResult
$Res call({
 FeedbackKind kind, String text, bool credit, String creditName, bool includeApp, FeedbackScreenshot? screenshot
});


$FeedbackScreenshotCopyWith<$Res>? get screenshot;

}
/// @nodoc
class _$FeedbackDraftCopyWithImpl<$Res>
    implements $FeedbackDraftCopyWith<$Res> {
  _$FeedbackDraftCopyWithImpl(this._self, this._then);

  final FeedbackDraft _self;
  final $Res Function(FeedbackDraft) _then;

/// Create a copy of FeedbackDraft
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? kind = null,Object? text = null,Object? credit = null,Object? creditName = null,Object? includeApp = null,Object? screenshot = freezed,}) {
  return _then(_self.copyWith(
kind: null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as FeedbackKind,text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,credit: null == credit ? _self.credit : credit // ignore: cast_nullable_to_non_nullable
as bool,creditName: null == creditName ? _self.creditName : creditName // ignore: cast_nullable_to_non_nullable
as String,includeApp: null == includeApp ? _self.includeApp : includeApp // ignore: cast_nullable_to_non_nullable
as bool,screenshot: freezed == screenshot ? _self.screenshot : screenshot // ignore: cast_nullable_to_non_nullable
as FeedbackScreenshot?,
  ));
}
/// Create a copy of FeedbackDraft
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$FeedbackScreenshotCopyWith<$Res>? get screenshot {
    if (_self.screenshot == null) {
    return null;
  }

  return $FeedbackScreenshotCopyWith<$Res>(_self.screenshot!, (value) {
    return _then(_self.copyWith(screenshot: value));
  });
}
}


/// Adds pattern-matching-related methods to [FeedbackDraft].
extension FeedbackDraftPatterns on FeedbackDraft {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _FeedbackDraft value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _FeedbackDraft() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _FeedbackDraft value)  $default,){
final _that = this;
switch (_that) {
case _FeedbackDraft():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _FeedbackDraft value)?  $default,){
final _that = this;
switch (_that) {
case _FeedbackDraft() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( FeedbackKind kind,  String text,  bool credit,  String creditName,  bool includeApp,  FeedbackScreenshot? screenshot)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _FeedbackDraft() when $default != null:
return $default(_that.kind,_that.text,_that.credit,_that.creditName,_that.includeApp,_that.screenshot);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( FeedbackKind kind,  String text,  bool credit,  String creditName,  bool includeApp,  FeedbackScreenshot? screenshot)  $default,) {final _that = this;
switch (_that) {
case _FeedbackDraft():
return $default(_that.kind,_that.text,_that.credit,_that.creditName,_that.includeApp,_that.screenshot);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( FeedbackKind kind,  String text,  bool credit,  String creditName,  bool includeApp,  FeedbackScreenshot? screenshot)?  $default,) {final _that = this;
switch (_that) {
case _FeedbackDraft() when $default != null:
return $default(_that.kind,_that.text,_that.credit,_that.creditName,_that.includeApp,_that.screenshot);case _:
  return null;

}
}

}

/// @nodoc


class _FeedbackDraft extends FeedbackDraft {
  const _FeedbackDraft({this.kind = FeedbackKind.bug, this.text = '', this.credit = false, this.creditName = '', this.includeApp = true, this.screenshot}): super._();
  

@override@JsonKey() final  FeedbackKind kind;
@override@JsonKey() final  String text;
/// Whether "Credit me by name" is on.
@override@JsonKey() final  bool credit;
@override@JsonKey() final  String creditName;
@override@JsonKey() final  bool includeApp;
@override final  FeedbackScreenshot? screenshot;

/// Create a copy of FeedbackDraft
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$FeedbackDraftCopyWith<_FeedbackDraft> get copyWith => __$FeedbackDraftCopyWithImpl<_FeedbackDraft>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _FeedbackDraft&&(identical(other.kind, kind) || other.kind == kind)&&(identical(other.text, text) || other.text == text)&&(identical(other.credit, credit) || other.credit == credit)&&(identical(other.creditName, creditName) || other.creditName == creditName)&&(identical(other.includeApp, includeApp) || other.includeApp == includeApp)&&(identical(other.screenshot, screenshot) || other.screenshot == screenshot));
}


@override
int get hashCode => Object.hash(runtimeType,kind,text,credit,creditName,includeApp,screenshot);

@override
String toString() {
  return 'FeedbackDraft(kind: $kind, text: $text, credit: $credit, creditName: $creditName, includeApp: $includeApp, screenshot: $screenshot)';
}


}

/// @nodoc
abstract mixin class _$FeedbackDraftCopyWith<$Res> implements $FeedbackDraftCopyWith<$Res> {
  factory _$FeedbackDraftCopyWith(_FeedbackDraft value, $Res Function(_FeedbackDraft) _then) = __$FeedbackDraftCopyWithImpl;
@override @useResult
$Res call({
 FeedbackKind kind, String text, bool credit, String creditName, bool includeApp, FeedbackScreenshot? screenshot
});


@override $FeedbackScreenshotCopyWith<$Res>? get screenshot;

}
/// @nodoc
class __$FeedbackDraftCopyWithImpl<$Res>
    implements _$FeedbackDraftCopyWith<$Res> {
  __$FeedbackDraftCopyWithImpl(this._self, this._then);

  final _FeedbackDraft _self;
  final $Res Function(_FeedbackDraft) _then;

/// Create a copy of FeedbackDraft
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? kind = null,Object? text = null,Object? credit = null,Object? creditName = null,Object? includeApp = null,Object? screenshot = freezed,}) {
  return _then(_FeedbackDraft(
kind: null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as FeedbackKind,text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,credit: null == credit ? _self.credit : credit // ignore: cast_nullable_to_non_nullable
as bool,creditName: null == creditName ? _self.creditName : creditName // ignore: cast_nullable_to_non_nullable
as String,includeApp: null == includeApp ? _self.includeApp : includeApp // ignore: cast_nullable_to_non_nullable
as bool,screenshot: freezed == screenshot ? _self.screenshot : screenshot // ignore: cast_nullable_to_non_nullable
as FeedbackScreenshot?,
  ));
}

/// Create a copy of FeedbackDraft
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$FeedbackScreenshotCopyWith<$Res>? get screenshot {
    if (_self.screenshot == null) {
    return null;
  }

  return $FeedbackScreenshotCopyWith<$Res>(_self.screenshot!, (value) {
    return _then(_self.copyWith(screenshot: value));
  });
}
}

// dart format on
