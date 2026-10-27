// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'account_notice.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AccountNotice {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountNotice);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AccountNotice()';
}


}

/// @nodoc
class $AccountNoticeCopyWith<$Res>  {
$AccountNoticeCopyWith(AccountNotice _, $Res Function(AccountNotice) __);
}


/// Adds pattern-matching-related methods to [AccountNotice].
extension AccountNoticePatterns on AccountNotice {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AccountNoticeDeviceSignedOut value)?  deviceSignedOut,TResult Function( AccountNoticePasskeyAdded value)?  passkeyAdded,TResult Function( AccountNoticePasskeyRemoved value)?  passkeyRemoved,TResult Function( AccountNoticeProviderLinked value)?  providerLinked,TResult Function( AccountNoticeProviderUnlinked value)?  providerUnlinked,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AccountNoticeDeviceSignedOut() when deviceSignedOut != null:
return deviceSignedOut(_that);case AccountNoticePasskeyAdded() when passkeyAdded != null:
return passkeyAdded(_that);case AccountNoticePasskeyRemoved() when passkeyRemoved != null:
return passkeyRemoved(_that);case AccountNoticeProviderLinked() when providerLinked != null:
return providerLinked(_that);case AccountNoticeProviderUnlinked() when providerUnlinked != null:
return providerUnlinked(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AccountNoticeDeviceSignedOut value)  deviceSignedOut,required TResult Function( AccountNoticePasskeyAdded value)  passkeyAdded,required TResult Function( AccountNoticePasskeyRemoved value)  passkeyRemoved,required TResult Function( AccountNoticeProviderLinked value)  providerLinked,required TResult Function( AccountNoticeProviderUnlinked value)  providerUnlinked,}){
final _that = this;
switch (_that) {
case AccountNoticeDeviceSignedOut():
return deviceSignedOut(_that);case AccountNoticePasskeyAdded():
return passkeyAdded(_that);case AccountNoticePasskeyRemoved():
return passkeyRemoved(_that);case AccountNoticeProviderLinked():
return providerLinked(_that);case AccountNoticeProviderUnlinked():
return providerUnlinked(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AccountNoticeDeviceSignedOut value)?  deviceSignedOut,TResult? Function( AccountNoticePasskeyAdded value)?  passkeyAdded,TResult? Function( AccountNoticePasskeyRemoved value)?  passkeyRemoved,TResult? Function( AccountNoticeProviderLinked value)?  providerLinked,TResult? Function( AccountNoticeProviderUnlinked value)?  providerUnlinked,}){
final _that = this;
switch (_that) {
case AccountNoticeDeviceSignedOut() when deviceSignedOut != null:
return deviceSignedOut(_that);case AccountNoticePasskeyAdded() when passkeyAdded != null:
return passkeyAdded(_that);case AccountNoticePasskeyRemoved() when passkeyRemoved != null:
return passkeyRemoved(_that);case AccountNoticeProviderLinked() when providerLinked != null:
return providerLinked(_that);case AccountNoticeProviderUnlinked() when providerUnlinked != null:
return providerUnlinked(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String name)?  deviceSignedOut,TResult Function( bool canUnlock)?  passkeyAdded,TResult Function()?  passkeyRemoved,TResult Function( ProviderKind provider)?  providerLinked,TResult Function( ProviderKind provider)?  providerUnlinked,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AccountNoticeDeviceSignedOut() when deviceSignedOut != null:
return deviceSignedOut(_that.name);case AccountNoticePasskeyAdded() when passkeyAdded != null:
return passkeyAdded(_that.canUnlock);case AccountNoticePasskeyRemoved() when passkeyRemoved != null:
return passkeyRemoved();case AccountNoticeProviderLinked() when providerLinked != null:
return providerLinked(_that.provider);case AccountNoticeProviderUnlinked() when providerUnlinked != null:
return providerUnlinked(_that.provider);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String name)  deviceSignedOut,required TResult Function( bool canUnlock)  passkeyAdded,required TResult Function()  passkeyRemoved,required TResult Function( ProviderKind provider)  providerLinked,required TResult Function( ProviderKind provider)  providerUnlinked,}) {final _that = this;
switch (_that) {
case AccountNoticeDeviceSignedOut():
return deviceSignedOut(_that.name);case AccountNoticePasskeyAdded():
return passkeyAdded(_that.canUnlock);case AccountNoticePasskeyRemoved():
return passkeyRemoved();case AccountNoticeProviderLinked():
return providerLinked(_that.provider);case AccountNoticeProviderUnlinked():
return providerUnlinked(_that.provider);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String name)?  deviceSignedOut,TResult? Function( bool canUnlock)?  passkeyAdded,TResult? Function()?  passkeyRemoved,TResult? Function( ProviderKind provider)?  providerLinked,TResult? Function( ProviderKind provider)?  providerUnlinked,}) {final _that = this;
switch (_that) {
case AccountNoticeDeviceSignedOut() when deviceSignedOut != null:
return deviceSignedOut(_that.name);case AccountNoticePasskeyAdded() when passkeyAdded != null:
return passkeyAdded(_that.canUnlock);case AccountNoticePasskeyRemoved() when passkeyRemoved != null:
return passkeyRemoved();case AccountNoticeProviderLinked() when providerLinked != null:
return providerLinked(_that.provider);case AccountNoticeProviderUnlinked() when providerUnlinked != null:
return providerUnlinked(_that.provider);case _:
  return null;

}
}

}

/// @nodoc


class AccountNoticeDeviceSignedOut implements AccountNotice {
  const AccountNoticeDeviceSignedOut({required this.name});
  

 final  String name;

/// Create a copy of AccountNotice
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AccountNoticeDeviceSignedOutCopyWith<AccountNoticeDeviceSignedOut> get copyWith => _$AccountNoticeDeviceSignedOutCopyWithImpl<AccountNoticeDeviceSignedOut>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountNoticeDeviceSignedOut&&(identical(other.name, name) || other.name == name));
}


@override
int get hashCode => Object.hash(runtimeType,name);

@override
String toString() {
  return 'AccountNotice.deviceSignedOut(name: $name)';
}


}

/// @nodoc
abstract mixin class $AccountNoticeDeviceSignedOutCopyWith<$Res> implements $AccountNoticeCopyWith<$Res> {
  factory $AccountNoticeDeviceSignedOutCopyWith(AccountNoticeDeviceSignedOut value, $Res Function(AccountNoticeDeviceSignedOut) _then) = _$AccountNoticeDeviceSignedOutCopyWithImpl;
@useResult
$Res call({
 String name
});




}
/// @nodoc
class _$AccountNoticeDeviceSignedOutCopyWithImpl<$Res>
    implements $AccountNoticeDeviceSignedOutCopyWith<$Res> {
  _$AccountNoticeDeviceSignedOutCopyWithImpl(this._self, this._then);

  final AccountNoticeDeviceSignedOut _self;
  final $Res Function(AccountNoticeDeviceSignedOut) _then;

/// Create a copy of AccountNotice
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? name = null,}) {
  return _then(AccountNoticeDeviceSignedOut(
name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AccountNoticePasskeyAdded implements AccountNotice {
  const AccountNoticePasskeyAdded({required this.canUnlock});
  

 final  bool canUnlock;

/// Create a copy of AccountNotice
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AccountNoticePasskeyAddedCopyWith<AccountNoticePasskeyAdded> get copyWith => _$AccountNoticePasskeyAddedCopyWithImpl<AccountNoticePasskeyAdded>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountNoticePasskeyAdded&&(identical(other.canUnlock, canUnlock) || other.canUnlock == canUnlock));
}


@override
int get hashCode => Object.hash(runtimeType,canUnlock);

@override
String toString() {
  return 'AccountNotice.passkeyAdded(canUnlock: $canUnlock)';
}


}

/// @nodoc
abstract mixin class $AccountNoticePasskeyAddedCopyWith<$Res> implements $AccountNoticeCopyWith<$Res> {
  factory $AccountNoticePasskeyAddedCopyWith(AccountNoticePasskeyAdded value, $Res Function(AccountNoticePasskeyAdded) _then) = _$AccountNoticePasskeyAddedCopyWithImpl;
@useResult
$Res call({
 bool canUnlock
});




}
/// @nodoc
class _$AccountNoticePasskeyAddedCopyWithImpl<$Res>
    implements $AccountNoticePasskeyAddedCopyWith<$Res> {
  _$AccountNoticePasskeyAddedCopyWithImpl(this._self, this._then);

  final AccountNoticePasskeyAdded _self;
  final $Res Function(AccountNoticePasskeyAdded) _then;

/// Create a copy of AccountNotice
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? canUnlock = null,}) {
  return _then(AccountNoticePasskeyAdded(
canUnlock: null == canUnlock ? _self.canUnlock : canUnlock // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class AccountNoticePasskeyRemoved implements AccountNotice {
  const AccountNoticePasskeyRemoved();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountNoticePasskeyRemoved);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AccountNotice.passkeyRemoved()';
}


}




/// @nodoc


class AccountNoticeProviderLinked implements AccountNotice {
  const AccountNoticeProviderLinked({required this.provider});
  

 final  ProviderKind provider;

/// Create a copy of AccountNotice
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AccountNoticeProviderLinkedCopyWith<AccountNoticeProviderLinked> get copyWith => _$AccountNoticeProviderLinkedCopyWithImpl<AccountNoticeProviderLinked>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountNoticeProviderLinked&&(identical(other.provider, provider) || other.provider == provider));
}


@override
int get hashCode => Object.hash(runtimeType,provider);

@override
String toString() {
  return 'AccountNotice.providerLinked(provider: $provider)';
}


}

/// @nodoc
abstract mixin class $AccountNoticeProviderLinkedCopyWith<$Res> implements $AccountNoticeCopyWith<$Res> {
  factory $AccountNoticeProviderLinkedCopyWith(AccountNoticeProviderLinked value, $Res Function(AccountNoticeProviderLinked) _then) = _$AccountNoticeProviderLinkedCopyWithImpl;
@useResult
$Res call({
 ProviderKind provider
});




}
/// @nodoc
class _$AccountNoticeProviderLinkedCopyWithImpl<$Res>
    implements $AccountNoticeProviderLinkedCopyWith<$Res> {
  _$AccountNoticeProviderLinkedCopyWithImpl(this._self, this._then);

  final AccountNoticeProviderLinked _self;
  final $Res Function(AccountNoticeProviderLinked) _then;

/// Create a copy of AccountNotice
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? provider = null,}) {
  return _then(AccountNoticeProviderLinked(
provider: null == provider ? _self.provider : provider // ignore: cast_nullable_to_non_nullable
as ProviderKind,
  ));
}


}

/// @nodoc


class AccountNoticeProviderUnlinked implements AccountNotice {
  const AccountNoticeProviderUnlinked({required this.provider});
  

 final  ProviderKind provider;

/// Create a copy of AccountNotice
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AccountNoticeProviderUnlinkedCopyWith<AccountNoticeProviderUnlinked> get copyWith => _$AccountNoticeProviderUnlinkedCopyWithImpl<AccountNoticeProviderUnlinked>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountNoticeProviderUnlinked&&(identical(other.provider, provider) || other.provider == provider));
}


@override
int get hashCode => Object.hash(runtimeType,provider);

@override
String toString() {
  return 'AccountNotice.providerUnlinked(provider: $provider)';
}


}

/// @nodoc
abstract mixin class $AccountNoticeProviderUnlinkedCopyWith<$Res> implements $AccountNoticeCopyWith<$Res> {
  factory $AccountNoticeProviderUnlinkedCopyWith(AccountNoticeProviderUnlinked value, $Res Function(AccountNoticeProviderUnlinked) _then) = _$AccountNoticeProviderUnlinkedCopyWithImpl;
@useResult
$Res call({
 ProviderKind provider
});




}
/// @nodoc
class _$AccountNoticeProviderUnlinkedCopyWithImpl<$Res>
    implements $AccountNoticeProviderUnlinkedCopyWith<$Res> {
  _$AccountNoticeProviderUnlinkedCopyWithImpl(this._self, this._then);

  final AccountNoticeProviderUnlinked _self;
  final $Res Function(AccountNoticeProviderUnlinked) _then;

/// Create a copy of AccountNotice
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? provider = null,}) {
  return _then(AccountNoticeProviderUnlinked(
provider: null == provider ? _self.provider : provider // ignore: cast_nullable_to_non_nullable
as ProviderKind,
  ));
}


}

// dart format on
