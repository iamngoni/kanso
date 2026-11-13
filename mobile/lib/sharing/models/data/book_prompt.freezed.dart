// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'book_prompt.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$BookPrompt {

 BookRole get role;
/// Create a copy of BookPrompt
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BookPromptCopyWith<BookPrompt> get copyWith => _$BookPromptCopyWithImpl<BookPrompt>(this as BookPrompt, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BookPrompt&&(identical(other.role, role) || other.role == role));
}


@override
int get hashCode => Object.hash(runtimeType,role);

@override
String toString() {
  return 'BookPrompt(role: $role)';
}


}

/// @nodoc
abstract mixin class $BookPromptCopyWith<$Res>  {
  factory $BookPromptCopyWith(BookPrompt value, $Res Function(BookPrompt) _then) = _$BookPromptCopyWithImpl;
@useResult
$Res call({
 BookRole role
});




}
/// @nodoc
class _$BookPromptCopyWithImpl<$Res>
    implements $BookPromptCopyWith<$Res> {
  _$BookPromptCopyWithImpl(this._self, this._then);

  final BookPrompt _self;
  final $Res Function(BookPrompt) _then;

/// Create a copy of BookPrompt
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? role = null,}) {
  return _then(_self.copyWith(
role: null == role ? _self.role : role // ignore: cast_nullable_to_non_nullable
as BookRole,
  ));
}

}


/// Adds pattern-matching-related methods to [BookPrompt].
extension BookPromptPatterns on BookPrompt {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( BookPromptVerify value)?  verify,TResult Function( BookPromptKeyChanged value)?  keyChanged,required TResult orElse(),}){
final _that = this;
switch (_that) {
case BookPromptVerify() when verify != null:
return verify(_that);case BookPromptKeyChanged() when keyChanged != null:
return keyChanged(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( BookPromptVerify value)  verify,required TResult Function( BookPromptKeyChanged value)  keyChanged,}){
final _that = this;
switch (_that) {
case BookPromptVerify():
return verify(_that);case BookPromptKeyChanged():
return keyChanged(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( BookPromptVerify value)?  verify,TResult? Function( BookPromptKeyChanged value)?  keyChanged,}){
final _that = this;
switch (_that) {
case BookPromptVerify() when verify != null:
return verify(_that);case BookPromptKeyChanged() when keyChanged != null:
return keyChanged(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( PersonRow person,  BookRole role)?  verify,TResult Function( PersonRow offered,  BookRole role,  PersonRow? pinned)?  keyChanged,required TResult orElse(),}) {final _that = this;
switch (_that) {
case BookPromptVerify() when verify != null:
return verify(_that.person,_that.role);case BookPromptKeyChanged() when keyChanged != null:
return keyChanged(_that.offered,_that.role,_that.pinned);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( PersonRow person,  BookRole role)  verify,required TResult Function( PersonRow offered,  BookRole role,  PersonRow? pinned)  keyChanged,}) {final _that = this;
switch (_that) {
case BookPromptVerify():
return verify(_that.person,_that.role);case BookPromptKeyChanged():
return keyChanged(_that.offered,_that.role,_that.pinned);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( PersonRow person,  BookRole role)?  verify,TResult? Function( PersonRow offered,  BookRole role,  PersonRow? pinned)?  keyChanged,}) {final _that = this;
switch (_that) {
case BookPromptVerify() when verify != null:
return verify(_that.person,_that.role);case BookPromptKeyChanged() when keyChanged != null:
return keyChanged(_that.offered,_that.role,_that.pinned);case _:
  return null;

}
}

}

/// @nodoc


class BookPromptVerify implements BookPrompt {
  const BookPromptVerify({required this.person, required this.role});
  

 final  PersonRow person;
@override final  BookRole role;

/// Create a copy of BookPrompt
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BookPromptVerifyCopyWith<BookPromptVerify> get copyWith => _$BookPromptVerifyCopyWithImpl<BookPromptVerify>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BookPromptVerify&&(identical(other.person, person) || other.person == person)&&(identical(other.role, role) || other.role == role));
}


@override
int get hashCode => Object.hash(runtimeType,person,role);

@override
String toString() {
  return 'BookPrompt.verify(person: $person, role: $role)';
}


}

/// @nodoc
abstract mixin class $BookPromptVerifyCopyWith<$Res> implements $BookPromptCopyWith<$Res> {
  factory $BookPromptVerifyCopyWith(BookPromptVerify value, $Res Function(BookPromptVerify) _then) = _$BookPromptVerifyCopyWithImpl;
@override @useResult
$Res call({
 PersonRow person, BookRole role
});




}
/// @nodoc
class _$BookPromptVerifyCopyWithImpl<$Res>
    implements $BookPromptVerifyCopyWith<$Res> {
  _$BookPromptVerifyCopyWithImpl(this._self, this._then);

  final BookPromptVerify _self;
  final $Res Function(BookPromptVerify) _then;

/// Create a copy of BookPrompt
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? person = null,Object? role = null,}) {
  return _then(BookPromptVerify(
person: null == person ? _self.person : person // ignore: cast_nullable_to_non_nullable
as PersonRow,role: null == role ? _self.role : role // ignore: cast_nullable_to_non_nullable
as BookRole,
  ));
}


}

/// @nodoc


class BookPromptKeyChanged implements BookPrompt {
  const BookPromptKeyChanged({required this.offered, required this.role, this.pinned});
  

 final  PersonRow offered;
@override final  BookRole role;
 final  PersonRow? pinned;

/// Create a copy of BookPrompt
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BookPromptKeyChangedCopyWith<BookPromptKeyChanged> get copyWith => _$BookPromptKeyChangedCopyWithImpl<BookPromptKeyChanged>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BookPromptKeyChanged&&(identical(other.offered, offered) || other.offered == offered)&&(identical(other.role, role) || other.role == role)&&(identical(other.pinned, pinned) || other.pinned == pinned));
}


@override
int get hashCode => Object.hash(runtimeType,offered,role,pinned);

@override
String toString() {
  return 'BookPrompt.keyChanged(offered: $offered, role: $role, pinned: $pinned)';
}


}

/// @nodoc
abstract mixin class $BookPromptKeyChangedCopyWith<$Res> implements $BookPromptCopyWith<$Res> {
  factory $BookPromptKeyChangedCopyWith(BookPromptKeyChanged value, $Res Function(BookPromptKeyChanged) _then) = _$BookPromptKeyChangedCopyWithImpl;
@override @useResult
$Res call({
 PersonRow offered, BookRole role, PersonRow? pinned
});




}
/// @nodoc
class _$BookPromptKeyChangedCopyWithImpl<$Res>
    implements $BookPromptKeyChangedCopyWith<$Res> {
  _$BookPromptKeyChangedCopyWithImpl(this._self, this._then);

  final BookPromptKeyChanged _self;
  final $Res Function(BookPromptKeyChanged) _then;

/// Create a copy of BookPrompt
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? offered = null,Object? role = null,Object? pinned = freezed,}) {
  return _then(BookPromptKeyChanged(
offered: null == offered ? _self.offered : offered // ignore: cast_nullable_to_non_nullable
as PersonRow,role: null == role ? _self.role : role // ignore: cast_nullable_to_non_nullable
as BookRole,pinned: freezed == pinned ? _self.pinned : pinned // ignore: cast_nullable_to_non_nullable
as PersonRow?,
  ));
}


}

/// @nodoc
mixin _$BookDone {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BookDone);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'BookDone()';
}


}

/// @nodoc
class $BookDoneCopyWith<$Res>  {
$BookDoneCopyWith(BookDone _, $Res Function(BookDone) __);
}


/// Adds pattern-matching-related methods to [BookDone].
extension BookDonePatterns on BookDone {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( BookDoneShared value)?  shared,TResult Function( BookDoneInvited value)?  invited,TResult Function( BookDoneRemoved value)?  removed,TResult Function( BookDoneEnded value)?  ended,required TResult orElse(),}){
final _that = this;
switch (_that) {
case BookDoneShared() when shared != null:
return shared(_that);case BookDoneInvited() when invited != null:
return invited(_that);case BookDoneRemoved() when removed != null:
return removed(_that);case BookDoneEnded() when ended != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( BookDoneShared value)  shared,required TResult Function( BookDoneInvited value)  invited,required TResult Function( BookDoneRemoved value)  removed,required TResult Function( BookDoneEnded value)  ended,}){
final _that = this;
switch (_that) {
case BookDoneShared():
return shared(_that);case BookDoneInvited():
return invited(_that);case BookDoneRemoved():
return removed(_that);case BookDoneEnded():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( BookDoneShared value)?  shared,TResult? Function( BookDoneInvited value)?  invited,TResult? Function( BookDoneRemoved value)?  removed,TResult? Function( BookDoneEnded value)?  ended,}){
final _that = this;
switch (_that) {
case BookDoneShared() when shared != null:
return shared(_that);case BookDoneInvited() when invited != null:
return invited(_that);case BookDoneRemoved() when removed != null:
return removed(_that);case BookDoneEnded() when ended != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String username)?  shared,TResult Function( String username)?  invited,TResult Function( String username)?  removed,TResult Function( BookEndedRow ended)?  ended,required TResult orElse(),}) {final _that = this;
switch (_that) {
case BookDoneShared() when shared != null:
return shared(_that.username);case BookDoneInvited() when invited != null:
return invited(_that.username);case BookDoneRemoved() when removed != null:
return removed(_that.username);case BookDoneEnded() when ended != null:
return ended(_that.ended);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String username)  shared,required TResult Function( String username)  invited,required TResult Function( String username)  removed,required TResult Function( BookEndedRow ended)  ended,}) {final _that = this;
switch (_that) {
case BookDoneShared():
return shared(_that.username);case BookDoneInvited():
return invited(_that.username);case BookDoneRemoved():
return removed(_that.username);case BookDoneEnded():
return ended(_that.ended);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String username)?  shared,TResult? Function( String username)?  invited,TResult? Function( String username)?  removed,TResult? Function( BookEndedRow ended)?  ended,}) {final _that = this;
switch (_that) {
case BookDoneShared() when shared != null:
return shared(_that.username);case BookDoneInvited() when invited != null:
return invited(_that.username);case BookDoneRemoved() when removed != null:
return removed(_that.username);case BookDoneEnded() when ended != null:
return ended(_that.ended);case _:
  return null;

}
}

}

/// @nodoc


class BookDoneShared implements BookDone {
  const BookDoneShared(this.username);
  

 final  String username;

/// Create a copy of BookDone
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BookDoneSharedCopyWith<BookDoneShared> get copyWith => _$BookDoneSharedCopyWithImpl<BookDoneShared>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BookDoneShared&&(identical(other.username, username) || other.username == username));
}


@override
int get hashCode => Object.hash(runtimeType,username);

@override
String toString() {
  return 'BookDone.shared(username: $username)';
}


}

/// @nodoc
abstract mixin class $BookDoneSharedCopyWith<$Res> implements $BookDoneCopyWith<$Res> {
  factory $BookDoneSharedCopyWith(BookDoneShared value, $Res Function(BookDoneShared) _then) = _$BookDoneSharedCopyWithImpl;
@useResult
$Res call({
 String username
});




}
/// @nodoc
class _$BookDoneSharedCopyWithImpl<$Res>
    implements $BookDoneSharedCopyWith<$Res> {
  _$BookDoneSharedCopyWithImpl(this._self, this._then);

  final BookDoneShared _self;
  final $Res Function(BookDoneShared) _then;

/// Create a copy of BookDone
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? username = null,}) {
  return _then(BookDoneShared(
null == username ? _self.username : username // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class BookDoneInvited implements BookDone {
  const BookDoneInvited(this.username);
  

 final  String username;

/// Create a copy of BookDone
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BookDoneInvitedCopyWith<BookDoneInvited> get copyWith => _$BookDoneInvitedCopyWithImpl<BookDoneInvited>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BookDoneInvited&&(identical(other.username, username) || other.username == username));
}


@override
int get hashCode => Object.hash(runtimeType,username);

@override
String toString() {
  return 'BookDone.invited(username: $username)';
}


}

/// @nodoc
abstract mixin class $BookDoneInvitedCopyWith<$Res> implements $BookDoneCopyWith<$Res> {
  factory $BookDoneInvitedCopyWith(BookDoneInvited value, $Res Function(BookDoneInvited) _then) = _$BookDoneInvitedCopyWithImpl;
@useResult
$Res call({
 String username
});




}
/// @nodoc
class _$BookDoneInvitedCopyWithImpl<$Res>
    implements $BookDoneInvitedCopyWith<$Res> {
  _$BookDoneInvitedCopyWithImpl(this._self, this._then);

  final BookDoneInvited _self;
  final $Res Function(BookDoneInvited) _then;

/// Create a copy of BookDone
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? username = null,}) {
  return _then(BookDoneInvited(
null == username ? _self.username : username // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class BookDoneRemoved implements BookDone {
  const BookDoneRemoved(this.username);
  

 final  String username;

/// Create a copy of BookDone
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BookDoneRemovedCopyWith<BookDoneRemoved> get copyWith => _$BookDoneRemovedCopyWithImpl<BookDoneRemoved>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BookDoneRemoved&&(identical(other.username, username) || other.username == username));
}


@override
int get hashCode => Object.hash(runtimeType,username);

@override
String toString() {
  return 'BookDone.removed(username: $username)';
}


}

/// @nodoc
abstract mixin class $BookDoneRemovedCopyWith<$Res> implements $BookDoneCopyWith<$Res> {
  factory $BookDoneRemovedCopyWith(BookDoneRemoved value, $Res Function(BookDoneRemoved) _then) = _$BookDoneRemovedCopyWithImpl;
@useResult
$Res call({
 String username
});




}
/// @nodoc
class _$BookDoneRemovedCopyWithImpl<$Res>
    implements $BookDoneRemovedCopyWith<$Res> {
  _$BookDoneRemovedCopyWithImpl(this._self, this._then);

  final BookDoneRemoved _self;
  final $Res Function(BookDoneRemoved) _then;

/// Create a copy of BookDone
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? username = null,}) {
  return _then(BookDoneRemoved(
null == username ? _self.username : username // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class BookDoneEnded implements BookDone {
  const BookDoneEnded(this.ended);
  

 final  BookEndedRow ended;

/// Create a copy of BookDone
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BookDoneEndedCopyWith<BookDoneEnded> get copyWith => _$BookDoneEndedCopyWithImpl<BookDoneEnded>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BookDoneEnded&&(identical(other.ended, ended) || other.ended == ended));
}


@override
int get hashCode => Object.hash(runtimeType,ended);

@override
String toString() {
  return 'BookDone.ended(ended: $ended)';
}


}

/// @nodoc
abstract mixin class $BookDoneEndedCopyWith<$Res> implements $BookDoneCopyWith<$Res> {
  factory $BookDoneEndedCopyWith(BookDoneEnded value, $Res Function(BookDoneEnded) _then) = _$BookDoneEndedCopyWithImpl;
@useResult
$Res call({
 BookEndedRow ended
});




}
/// @nodoc
class _$BookDoneEndedCopyWithImpl<$Res>
    implements $BookDoneEndedCopyWith<$Res> {
  _$BookDoneEndedCopyWithImpl(this._self, this._then);

  final BookDoneEnded _self;
  final $Res Function(BookDoneEnded) _then;

/// Create a copy of BookDone
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? ended = null,}) {
  return _then(BookDoneEnded(
null == ended ? _self.ended : ended // ignore: cast_nullable_to_non_nullable
as BookEndedRow,
  ));
}


}

// dart format on
