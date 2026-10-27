// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'share_done.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ShareDone {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareDone);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareDone()';
}


}

/// @nodoc
class $ShareDoneCopyWith<$Res>  {
$ShareDoneCopyWith(ShareDone _, $Res Function(ShareDone) __);
}


/// Adds pattern-matching-related methods to [ShareDone].
extension ShareDonePatterns on ShareDone {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ShareDoneInvited value)?  invited,TResult Function( ShareDoneStopped value)?  stopped,TResult Function( ShareDoneLeft value)?  left,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ShareDoneInvited() when invited != null:
return invited(_that);case ShareDoneStopped() when stopped != null:
return stopped(_that);case ShareDoneLeft() when left != null:
return left(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ShareDoneInvited value)  invited,required TResult Function( ShareDoneStopped value)  stopped,required TResult Function( ShareDoneLeft value)  left,}){
final _that = this;
switch (_that) {
case ShareDoneInvited():
return invited(_that);case ShareDoneStopped():
return stopped(_that);case ShareDoneLeft():
return left(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ShareDoneInvited value)?  invited,TResult? Function( ShareDoneStopped value)?  stopped,TResult? Function( ShareDoneLeft value)?  left,}){
final _that = this;
switch (_that) {
case ShareDoneInvited() when invited != null:
return invited(_that);case ShareDoneStopped() when stopped != null:
return stopped(_that);case ShareDoneLeft() when left != null:
return left(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String username)?  invited,TResult Function()?  stopped,TResult Function()?  left,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ShareDoneInvited() when invited != null:
return invited(_that.username);case ShareDoneStopped() when stopped != null:
return stopped();case ShareDoneLeft() when left != null:
return left();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String username)  invited,required TResult Function()  stopped,required TResult Function()  left,}) {final _that = this;
switch (_that) {
case ShareDoneInvited():
return invited(_that.username);case ShareDoneStopped():
return stopped();case ShareDoneLeft():
return left();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String username)?  invited,TResult? Function()?  stopped,TResult? Function()?  left,}) {final _that = this;
switch (_that) {
case ShareDoneInvited() when invited != null:
return invited(_that.username);case ShareDoneStopped() when stopped != null:
return stopped();case ShareDoneLeft() when left != null:
return left();case _:
  return null;

}
}

}

/// @nodoc


class ShareDoneInvited implements ShareDone {
  const ShareDoneInvited(this.username);
  

 final  String username;

/// Create a copy of ShareDone
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ShareDoneInvitedCopyWith<ShareDoneInvited> get copyWith => _$ShareDoneInvitedCopyWithImpl<ShareDoneInvited>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareDoneInvited&&(identical(other.username, username) || other.username == username));
}


@override
int get hashCode => Object.hash(runtimeType,username);

@override
String toString() {
  return 'ShareDone.invited(username: $username)';
}


}

/// @nodoc
abstract mixin class $ShareDoneInvitedCopyWith<$Res> implements $ShareDoneCopyWith<$Res> {
  factory $ShareDoneInvitedCopyWith(ShareDoneInvited value, $Res Function(ShareDoneInvited) _then) = _$ShareDoneInvitedCopyWithImpl;
@useResult
$Res call({
 String username
});




}
/// @nodoc
class _$ShareDoneInvitedCopyWithImpl<$Res>
    implements $ShareDoneInvitedCopyWith<$Res> {
  _$ShareDoneInvitedCopyWithImpl(this._self, this._then);

  final ShareDoneInvited _self;
  final $Res Function(ShareDoneInvited) _then;

/// Create a copy of ShareDone
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? username = null,}) {
  return _then(ShareDoneInvited(
null == username ? _self.username : username // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ShareDoneStopped implements ShareDone {
  const ShareDoneStopped();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareDoneStopped);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareDone.stopped()';
}


}




/// @nodoc


class ShareDoneLeft implements ShareDone {
  const ShareDoneLeft();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareDoneLeft);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareDone.left()';
}


}




// dart format on
