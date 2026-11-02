// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'about_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AboutEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AboutEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AboutEvent()';
}


}

/// @nodoc
class $AboutEventCopyWith<$Res>  {
$AboutEventCopyWith(AboutEvent _, $Res Function(AboutEvent) __);
}


/// Adds pattern-matching-related methods to [AboutEvent].
extension AboutEventPatterns on AboutEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AboutStartedEvent value)?  started,TResult Function( AboutWebsiteOpenedEvent value)?  websiteOpened,TResult Function( AboutReleaseNotesOpenedEvent value)?  releaseNotesOpened,TResult Function( AboutHelpOpenedEvent value)?  helpOpened,TResult Function( AboutRateOpenedEvent value)?  rateOpened,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AboutStartedEvent() when started != null:
return started(_that);case AboutWebsiteOpenedEvent() when websiteOpened != null:
return websiteOpened(_that);case AboutReleaseNotesOpenedEvent() when releaseNotesOpened != null:
return releaseNotesOpened(_that);case AboutHelpOpenedEvent() when helpOpened != null:
return helpOpened(_that);case AboutRateOpenedEvent() when rateOpened != null:
return rateOpened(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AboutStartedEvent value)  started,required TResult Function( AboutWebsiteOpenedEvent value)  websiteOpened,required TResult Function( AboutReleaseNotesOpenedEvent value)  releaseNotesOpened,required TResult Function( AboutHelpOpenedEvent value)  helpOpened,required TResult Function( AboutRateOpenedEvent value)  rateOpened,}){
final _that = this;
switch (_that) {
case AboutStartedEvent():
return started(_that);case AboutWebsiteOpenedEvent():
return websiteOpened(_that);case AboutReleaseNotesOpenedEvent():
return releaseNotesOpened(_that);case AboutHelpOpenedEvent():
return helpOpened(_that);case AboutRateOpenedEvent():
return rateOpened(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AboutStartedEvent value)?  started,TResult? Function( AboutWebsiteOpenedEvent value)?  websiteOpened,TResult? Function( AboutReleaseNotesOpenedEvent value)?  releaseNotesOpened,TResult? Function( AboutHelpOpenedEvent value)?  helpOpened,TResult? Function( AboutRateOpenedEvent value)?  rateOpened,}){
final _that = this;
switch (_that) {
case AboutStartedEvent() when started != null:
return started(_that);case AboutWebsiteOpenedEvent() when websiteOpened != null:
return websiteOpened(_that);case AboutReleaseNotesOpenedEvent() when releaseNotesOpened != null:
return releaseNotesOpened(_that);case AboutHelpOpenedEvent() when helpOpened != null:
return helpOpened(_that);case AboutRateOpenedEvent() when rateOpened != null:
return rateOpened(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function()?  websiteOpened,TResult Function()?  releaseNotesOpened,TResult Function()?  helpOpened,TResult Function()?  rateOpened,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AboutStartedEvent() when started != null:
return started();case AboutWebsiteOpenedEvent() when websiteOpened != null:
return websiteOpened();case AboutReleaseNotesOpenedEvent() when releaseNotesOpened != null:
return releaseNotesOpened();case AboutHelpOpenedEvent() when helpOpened != null:
return helpOpened();case AboutRateOpenedEvent() when rateOpened != null:
return rateOpened();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function()  websiteOpened,required TResult Function()  releaseNotesOpened,required TResult Function()  helpOpened,required TResult Function()  rateOpened,}) {final _that = this;
switch (_that) {
case AboutStartedEvent():
return started();case AboutWebsiteOpenedEvent():
return websiteOpened();case AboutReleaseNotesOpenedEvent():
return releaseNotesOpened();case AboutHelpOpenedEvent():
return helpOpened();case AboutRateOpenedEvent():
return rateOpened();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function()?  websiteOpened,TResult? Function()?  releaseNotesOpened,TResult? Function()?  helpOpened,TResult? Function()?  rateOpened,}) {final _that = this;
switch (_that) {
case AboutStartedEvent() when started != null:
return started();case AboutWebsiteOpenedEvent() when websiteOpened != null:
return websiteOpened();case AboutReleaseNotesOpenedEvent() when releaseNotesOpened != null:
return releaseNotesOpened();case AboutHelpOpenedEvent() when helpOpened != null:
return helpOpened();case AboutRateOpenedEvent() when rateOpened != null:
return rateOpened();case _:
  return null;

}
}

}

/// @nodoc


class AboutStartedEvent implements AboutEvent {
  const AboutStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AboutStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AboutEvent.started()';
}


}




/// @nodoc


class AboutWebsiteOpenedEvent implements AboutEvent {
  const AboutWebsiteOpenedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AboutWebsiteOpenedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AboutEvent.websiteOpened()';
}


}




/// @nodoc


class AboutReleaseNotesOpenedEvent implements AboutEvent {
  const AboutReleaseNotesOpenedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AboutReleaseNotesOpenedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AboutEvent.releaseNotesOpened()';
}


}




/// @nodoc


class AboutHelpOpenedEvent implements AboutEvent {
  const AboutHelpOpenedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AboutHelpOpenedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AboutEvent.helpOpened()';
}


}




/// @nodoc


class AboutRateOpenedEvent implements AboutEvent {
  const AboutRateOpenedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AboutRateOpenedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AboutEvent.rateOpened()';
}


}




/// @nodoc
mixin _$AboutState {

 LoadStatus get status;/// The version of the app; null until it is read.
 AppVersion? get version;
/// Create a copy of AboutState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AboutStateCopyWith<AboutState> get copyWith => _$AboutStateCopyWithImpl<AboutState>(this as AboutState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AboutState&&(identical(other.status, status) || other.status == status)&&(identical(other.version, version) || other.version == version));
}


@override
int get hashCode => Object.hash(runtimeType,status,version);

@override
String toString() {
  return 'AboutState(status: $status, version: $version)';
}


}

/// @nodoc
abstract mixin class $AboutStateCopyWith<$Res>  {
  factory $AboutStateCopyWith(AboutState value, $Res Function(AboutState) _then) = _$AboutStateCopyWithImpl;
@useResult
$Res call({
 LoadStatus status, AppVersion? version
});


$AppVersionCopyWith<$Res>? get version;

}
/// @nodoc
class _$AboutStateCopyWithImpl<$Res>
    implements $AboutStateCopyWith<$Res> {
  _$AboutStateCopyWithImpl(this._self, this._then);

  final AboutState _self;
  final $Res Function(AboutState) _then;

/// Create a copy of AboutState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? version = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,version: freezed == version ? _self.version : version // ignore: cast_nullable_to_non_nullable
as AppVersion?,
  ));
}
/// Create a copy of AboutState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AppVersionCopyWith<$Res>? get version {
    if (_self.version == null) {
    return null;
  }

  return $AppVersionCopyWith<$Res>(_self.version!, (value) {
    return _then(_self.copyWith(version: value));
  });
}
}


/// Adds pattern-matching-related methods to [AboutState].
extension AboutStatePatterns on AboutState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AboutState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AboutState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AboutState value)  $default,){
final _that = this;
switch (_that) {
case _AboutState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AboutState value)?  $default,){
final _that = this;
switch (_that) {
case _AboutState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LoadStatus status,  AppVersion? version)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AboutState() when $default != null:
return $default(_that.status,_that.version);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LoadStatus status,  AppVersion? version)  $default,) {final _that = this;
switch (_that) {
case _AboutState():
return $default(_that.status,_that.version);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LoadStatus status,  AppVersion? version)?  $default,) {final _that = this;
switch (_that) {
case _AboutState() when $default != null:
return $default(_that.status,_that.version);case _:
  return null;

}
}

}

/// @nodoc


class _AboutState implements AboutState {
  const _AboutState({this.status = LoadStatus.loading, this.version});
  

@override@JsonKey() final  LoadStatus status;
/// The version of the app; null until it is read.
@override final  AppVersion? version;

/// Create a copy of AboutState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AboutStateCopyWith<_AboutState> get copyWith => __$AboutStateCopyWithImpl<_AboutState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AboutState&&(identical(other.status, status) || other.status == status)&&(identical(other.version, version) || other.version == version));
}


@override
int get hashCode => Object.hash(runtimeType,status,version);

@override
String toString() {
  return 'AboutState(status: $status, version: $version)';
}


}

/// @nodoc
abstract mixin class _$AboutStateCopyWith<$Res> implements $AboutStateCopyWith<$Res> {
  factory _$AboutStateCopyWith(_AboutState value, $Res Function(_AboutState) _then) = __$AboutStateCopyWithImpl;
@override @useResult
$Res call({
 LoadStatus status, AppVersion? version
});


@override $AppVersionCopyWith<$Res>? get version;

}
/// @nodoc
class __$AboutStateCopyWithImpl<$Res>
    implements _$AboutStateCopyWith<$Res> {
  __$AboutStateCopyWithImpl(this._self, this._then);

  final _AboutState _self;
  final $Res Function(_AboutState) _then;

/// Create a copy of AboutState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? version = freezed,}) {
  return _then(_AboutState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,version: freezed == version ? _self.version : version // ignore: cast_nullable_to_non_nullable
as AppVersion?,
  ));
}

/// Create a copy of AboutState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AppVersionCopyWith<$Res>? get version {
    if (_self.version == null) {
    return null;
  }

  return $AppVersionCopyWith<$Res>(_self.version!, (value) {
    return _then(_self.copyWith(version: value));
  });
}
}

// dart format on
