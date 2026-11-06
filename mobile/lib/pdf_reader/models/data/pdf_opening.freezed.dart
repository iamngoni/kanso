// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'pdf_opening.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$PdfOpening {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfOpening);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PdfOpening()';
}


}

/// @nodoc
class $PdfOpeningCopyWith<$Res>  {
$PdfOpeningCopyWith(PdfOpening _, $Res Function(PdfOpening) __);
}


/// Adds pattern-matching-related methods to [PdfOpening].
extension PdfOpeningPatterns on PdfOpening {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PdfOpened value)?  opened,TResult Function( PdfDeclinedOpening value)?  declined,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PdfOpened() when opened != null:
return opened(_that);case PdfDeclinedOpening() when declined != null:
return declined(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PdfOpened value)  opened,required TResult Function( PdfDeclinedOpening value)  declined,}){
final _that = this;
switch (_that) {
case PdfOpened():
return opened(_that);case PdfDeclinedOpening():
return declined(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PdfOpened value)?  opened,TResult? Function( PdfDeclinedOpening value)?  declined,}){
final _that = this;
switch (_that) {
case PdfOpened() when opened != null:
return opened(_that);case PdfDeclinedOpening() when declined != null:
return declined(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( PdfDocumentSession session,  PdfInfo info)?  opened,TResult Function( PdfDeclined reason)?  declined,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PdfOpened() when opened != null:
return opened(_that.session,_that.info);case PdfDeclinedOpening() when declined != null:
return declined(_that.reason);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( PdfDocumentSession session,  PdfInfo info)  opened,required TResult Function( PdfDeclined reason)  declined,}) {final _that = this;
switch (_that) {
case PdfOpened():
return opened(_that.session,_that.info);case PdfDeclinedOpening():
return declined(_that.reason);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( PdfDocumentSession session,  PdfInfo info)?  opened,TResult? Function( PdfDeclined reason)?  declined,}) {final _that = this;
switch (_that) {
case PdfOpened() when opened != null:
return opened(_that.session,_that.info);case PdfDeclinedOpening() when declined != null:
return declined(_that.reason);case _:
  return null;

}
}

}

/// @nodoc


class PdfOpened implements PdfOpening {
  const PdfOpened({required this.session, required this.info});
  

 final  PdfDocumentSession session;
 final  PdfInfo info;

/// Create a copy of PdfOpening
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PdfOpenedCopyWith<PdfOpened> get copyWith => _$PdfOpenedCopyWithImpl<PdfOpened>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfOpened&&(identical(other.session, session) || other.session == session)&&(identical(other.info, info) || other.info == info));
}


@override
int get hashCode => Object.hash(runtimeType,session,info);

@override
String toString() {
  return 'PdfOpening.opened(session: $session, info: $info)';
}


}

/// @nodoc
abstract mixin class $PdfOpenedCopyWith<$Res> implements $PdfOpeningCopyWith<$Res> {
  factory $PdfOpenedCopyWith(PdfOpened value, $Res Function(PdfOpened) _then) = _$PdfOpenedCopyWithImpl;
@useResult
$Res call({
 PdfDocumentSession session, PdfInfo info
});




}
/// @nodoc
class _$PdfOpenedCopyWithImpl<$Res>
    implements $PdfOpenedCopyWith<$Res> {
  _$PdfOpenedCopyWithImpl(this._self, this._then);

  final PdfOpened _self;
  final $Res Function(PdfOpened) _then;

/// Create a copy of PdfOpening
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? session = null,Object? info = null,}) {
  return _then(PdfOpened(
session: null == session ? _self.session : session // ignore: cast_nullable_to_non_nullable
as PdfDocumentSession,info: null == info ? _self.info : info // ignore: cast_nullable_to_non_nullable
as PdfInfo,
  ));
}


}

/// @nodoc


class PdfDeclinedOpening implements PdfOpening {
  const PdfDeclinedOpening({required this.reason});
  

 final  PdfDeclined reason;

/// Create a copy of PdfOpening
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PdfDeclinedOpeningCopyWith<PdfDeclinedOpening> get copyWith => _$PdfDeclinedOpeningCopyWithImpl<PdfDeclinedOpening>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfDeclinedOpening&&(identical(other.reason, reason) || other.reason == reason));
}


@override
int get hashCode => Object.hash(runtimeType,reason);

@override
String toString() {
  return 'PdfOpening.declined(reason: $reason)';
}


}

/// @nodoc
abstract mixin class $PdfDeclinedOpeningCopyWith<$Res> implements $PdfOpeningCopyWith<$Res> {
  factory $PdfDeclinedOpeningCopyWith(PdfDeclinedOpening value, $Res Function(PdfDeclinedOpening) _then) = _$PdfDeclinedOpeningCopyWithImpl;
@useResult
$Res call({
 PdfDeclined reason
});


$PdfDeclinedCopyWith<$Res> get reason;

}
/// @nodoc
class _$PdfDeclinedOpeningCopyWithImpl<$Res>
    implements $PdfDeclinedOpeningCopyWith<$Res> {
  _$PdfDeclinedOpeningCopyWithImpl(this._self, this._then);

  final PdfDeclinedOpening _self;
  final $Res Function(PdfDeclinedOpening) _then;

/// Create a copy of PdfOpening
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? reason = null,}) {
  return _then(PdfDeclinedOpening(
reason: null == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as PdfDeclined,
  ));
}

/// Create a copy of PdfOpening
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$PdfDeclinedCopyWith<$Res> get reason {
  
  return $PdfDeclinedCopyWith<$Res>(_self.reason, (value) {
    return _then(_self.copyWith(reason: value));
  });
}
}

// dart format on
