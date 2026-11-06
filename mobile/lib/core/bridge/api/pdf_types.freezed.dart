// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'pdf_types.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$PdfDeclined {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfDeclined);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PdfDeclined()';
}


}

/// @nodoc
class $PdfDeclinedCopyWith<$Res>  {
$PdfDeclinedCopyWith(PdfDeclined _, $Res Function(PdfDeclined) __);
}


/// Adds pattern-matching-related methods to [PdfDeclined].
extension PdfDeclinedPatterns on PdfDeclined {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PdfDeclined_NeedsPassword value)?  needsPassword,TResult Function( PdfDeclined_WrongPassword value)?  wrongPassword,TResult Function( PdfDeclined_LockedNote value)?  lockedNote,TResult Function( PdfDeclined_Unavailable value)?  unavailable,TResult Function( PdfDeclined_Refused value)?  refused,TResult Function( PdfDeclined_NotAPdf value)?  notAPdf,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PdfDeclined_NeedsPassword() when needsPassword != null:
return needsPassword(_that);case PdfDeclined_WrongPassword() when wrongPassword != null:
return wrongPassword(_that);case PdfDeclined_LockedNote() when lockedNote != null:
return lockedNote(_that);case PdfDeclined_Unavailable() when unavailable != null:
return unavailable(_that);case PdfDeclined_Refused() when refused != null:
return refused(_that);case PdfDeclined_NotAPdf() when notAPdf != null:
return notAPdf(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PdfDeclined_NeedsPassword value)  needsPassword,required TResult Function( PdfDeclined_WrongPassword value)  wrongPassword,required TResult Function( PdfDeclined_LockedNote value)  lockedNote,required TResult Function( PdfDeclined_Unavailable value)  unavailable,required TResult Function( PdfDeclined_Refused value)  refused,required TResult Function( PdfDeclined_NotAPdf value)  notAPdf,}){
final _that = this;
switch (_that) {
case PdfDeclined_NeedsPassword():
return needsPassword(_that);case PdfDeclined_WrongPassword():
return wrongPassword(_that);case PdfDeclined_LockedNote():
return lockedNote(_that);case PdfDeclined_Unavailable():
return unavailable(_that);case PdfDeclined_Refused():
return refused(_that);case PdfDeclined_NotAPdf():
return notAPdf(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PdfDeclined_NeedsPassword value)?  needsPassword,TResult? Function( PdfDeclined_WrongPassword value)?  wrongPassword,TResult? Function( PdfDeclined_LockedNote value)?  lockedNote,TResult? Function( PdfDeclined_Unavailable value)?  unavailable,TResult? Function( PdfDeclined_Refused value)?  refused,TResult? Function( PdfDeclined_NotAPdf value)?  notAPdf,}){
final _that = this;
switch (_that) {
case PdfDeclined_NeedsPassword() when needsPassword != null:
return needsPassword(_that);case PdfDeclined_WrongPassword() when wrongPassword != null:
return wrongPassword(_that);case PdfDeclined_LockedNote() when lockedNote != null:
return lockedNote(_that);case PdfDeclined_Unavailable() when unavailable != null:
return unavailable(_that);case PdfDeclined_Refused() when refused != null:
return refused(_that);case PdfDeclined_NotAPdf() when notAPdf != null:
return notAPdf(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  needsPassword,TResult Function()?  wrongPassword,TResult Function( String message)?  lockedNote,TResult Function( String message)?  unavailable,TResult Function( String message)?  refused,TResult Function()?  notAPdf,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PdfDeclined_NeedsPassword() when needsPassword != null:
return needsPassword();case PdfDeclined_WrongPassword() when wrongPassword != null:
return wrongPassword();case PdfDeclined_LockedNote() when lockedNote != null:
return lockedNote(_that.message);case PdfDeclined_Unavailable() when unavailable != null:
return unavailable(_that.message);case PdfDeclined_Refused() when refused != null:
return refused(_that.message);case PdfDeclined_NotAPdf() when notAPdf != null:
return notAPdf();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  needsPassword,required TResult Function()  wrongPassword,required TResult Function( String message)  lockedNote,required TResult Function( String message)  unavailable,required TResult Function( String message)  refused,required TResult Function()  notAPdf,}) {final _that = this;
switch (_that) {
case PdfDeclined_NeedsPassword():
return needsPassword();case PdfDeclined_WrongPassword():
return wrongPassword();case PdfDeclined_LockedNote():
return lockedNote(_that.message);case PdfDeclined_Unavailable():
return unavailable(_that.message);case PdfDeclined_Refused():
return refused(_that.message);case PdfDeclined_NotAPdf():
return notAPdf();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  needsPassword,TResult? Function()?  wrongPassword,TResult? Function( String message)?  lockedNote,TResult? Function( String message)?  unavailable,TResult? Function( String message)?  refused,TResult? Function()?  notAPdf,}) {final _that = this;
switch (_that) {
case PdfDeclined_NeedsPassword() when needsPassword != null:
return needsPassword();case PdfDeclined_WrongPassword() when wrongPassword != null:
return wrongPassword();case PdfDeclined_LockedNote() when lockedNote != null:
return lockedNote(_that.message);case PdfDeclined_Unavailable() when unavailable != null:
return unavailable(_that.message);case PdfDeclined_Refused() when refused != null:
return refused(_that.message);case PdfDeclined_NotAPdf() when notAPdf != null:
return notAPdf();case _:
  return null;

}
}

}

/// @nodoc


class PdfDeclined_NeedsPassword extends PdfDeclined {
  const PdfDeclined_NeedsPassword(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfDeclined_NeedsPassword);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PdfDeclined.needsPassword()';
}


}




/// @nodoc


class PdfDeclined_WrongPassword extends PdfDeclined {
  const PdfDeclined_WrongPassword(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfDeclined_WrongPassword);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PdfDeclined.wrongPassword()';
}


}




/// @nodoc


class PdfDeclined_LockedNote extends PdfDeclined {
  const PdfDeclined_LockedNote({required this.message}): super._();
  

 final  String message;

/// Create a copy of PdfDeclined
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PdfDeclined_LockedNoteCopyWith<PdfDeclined_LockedNote> get copyWith => _$PdfDeclined_LockedNoteCopyWithImpl<PdfDeclined_LockedNote>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfDeclined_LockedNote&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'PdfDeclined.lockedNote(message: $message)';
}


}

/// @nodoc
abstract mixin class $PdfDeclined_LockedNoteCopyWith<$Res> implements $PdfDeclinedCopyWith<$Res> {
  factory $PdfDeclined_LockedNoteCopyWith(PdfDeclined_LockedNote value, $Res Function(PdfDeclined_LockedNote) _then) = _$PdfDeclined_LockedNoteCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$PdfDeclined_LockedNoteCopyWithImpl<$Res>
    implements $PdfDeclined_LockedNoteCopyWith<$Res> {
  _$PdfDeclined_LockedNoteCopyWithImpl(this._self, this._then);

  final PdfDeclined_LockedNote _self;
  final $Res Function(PdfDeclined_LockedNote) _then;

/// Create a copy of PdfDeclined
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(PdfDeclined_LockedNote(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class PdfDeclined_Unavailable extends PdfDeclined {
  const PdfDeclined_Unavailable({required this.message}): super._();
  

 final  String message;

/// Create a copy of PdfDeclined
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PdfDeclined_UnavailableCopyWith<PdfDeclined_Unavailable> get copyWith => _$PdfDeclined_UnavailableCopyWithImpl<PdfDeclined_Unavailable>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfDeclined_Unavailable&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'PdfDeclined.unavailable(message: $message)';
}


}

/// @nodoc
abstract mixin class $PdfDeclined_UnavailableCopyWith<$Res> implements $PdfDeclinedCopyWith<$Res> {
  factory $PdfDeclined_UnavailableCopyWith(PdfDeclined_Unavailable value, $Res Function(PdfDeclined_Unavailable) _then) = _$PdfDeclined_UnavailableCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$PdfDeclined_UnavailableCopyWithImpl<$Res>
    implements $PdfDeclined_UnavailableCopyWith<$Res> {
  _$PdfDeclined_UnavailableCopyWithImpl(this._self, this._then);

  final PdfDeclined_Unavailable _self;
  final $Res Function(PdfDeclined_Unavailable) _then;

/// Create a copy of PdfDeclined
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(PdfDeclined_Unavailable(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class PdfDeclined_Refused extends PdfDeclined {
  const PdfDeclined_Refused({required this.message}): super._();
  

 final  String message;

/// Create a copy of PdfDeclined
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PdfDeclined_RefusedCopyWith<PdfDeclined_Refused> get copyWith => _$PdfDeclined_RefusedCopyWithImpl<PdfDeclined_Refused>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfDeclined_Refused&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'PdfDeclined.refused(message: $message)';
}


}

/// @nodoc
abstract mixin class $PdfDeclined_RefusedCopyWith<$Res> implements $PdfDeclinedCopyWith<$Res> {
  factory $PdfDeclined_RefusedCopyWith(PdfDeclined_Refused value, $Res Function(PdfDeclined_Refused) _then) = _$PdfDeclined_RefusedCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$PdfDeclined_RefusedCopyWithImpl<$Res>
    implements $PdfDeclined_RefusedCopyWith<$Res> {
  _$PdfDeclined_RefusedCopyWithImpl(this._self, this._then);

  final PdfDeclined_Refused _self;
  final $Res Function(PdfDeclined_Refused) _then;

/// Create a copy of PdfDeclined
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(PdfDeclined_Refused(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class PdfDeclined_NotAPdf extends PdfDeclined {
  const PdfDeclined_NotAPdf(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfDeclined_NotAPdf);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PdfDeclined.notAPdf()';
}


}




// dart format on
