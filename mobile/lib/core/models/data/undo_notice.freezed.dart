// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'undo_notice.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$UndoNotice {

/// The ticket the library keeps the change under (`Library::undo`).
 int get ticket;/// What the notice says, as the library words it.
 String get message;/// Which notice this is: a later one has a larger number, so a timeout
/// belongs to the notice it was started for.
 int get serial;
/// Create a copy of UndoNotice
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$UndoNoticeCopyWith<UndoNotice> get copyWith => _$UndoNoticeCopyWithImpl<UndoNotice>(this as UndoNotice, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UndoNotice&&(identical(other.ticket, ticket) || other.ticket == ticket)&&(identical(other.message, message) || other.message == message)&&(identical(other.serial, serial) || other.serial == serial));
}


@override
int get hashCode => Object.hash(runtimeType,ticket,message,serial);

@override
String toString() {
  return 'UndoNotice(ticket: $ticket, message: $message, serial: $serial)';
}


}

/// @nodoc
abstract mixin class $UndoNoticeCopyWith<$Res>  {
  factory $UndoNoticeCopyWith(UndoNotice value, $Res Function(UndoNotice) _then) = _$UndoNoticeCopyWithImpl;
@useResult
$Res call({
 int ticket, String message, int serial
});




}
/// @nodoc
class _$UndoNoticeCopyWithImpl<$Res>
    implements $UndoNoticeCopyWith<$Res> {
  _$UndoNoticeCopyWithImpl(this._self, this._then);

  final UndoNotice _self;
  final $Res Function(UndoNotice) _then;

/// Create a copy of UndoNotice
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? ticket = null,Object? message = null,Object? serial = null,}) {
  return _then(_self.copyWith(
ticket: null == ticket ? _self.ticket : ticket // ignore: cast_nullable_to_non_nullable
as int,message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,serial: null == serial ? _self.serial : serial // ignore: cast_nullable_to_non_nullable
as int,
  ));
}

}


/// Adds pattern-matching-related methods to [UndoNotice].
extension UndoNoticePatterns on UndoNotice {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _UndoNotice value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _UndoNotice() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _UndoNotice value)  $default,){
final _that = this;
switch (_that) {
case _UndoNotice():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _UndoNotice value)?  $default,){
final _that = this;
switch (_that) {
case _UndoNotice() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( int ticket,  String message,  int serial)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _UndoNotice() when $default != null:
return $default(_that.ticket,_that.message,_that.serial);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( int ticket,  String message,  int serial)  $default,) {final _that = this;
switch (_that) {
case _UndoNotice():
return $default(_that.ticket,_that.message,_that.serial);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( int ticket,  String message,  int serial)?  $default,) {final _that = this;
switch (_that) {
case _UndoNotice() when $default != null:
return $default(_that.ticket,_that.message,_that.serial);case _:
  return null;

}
}

}

/// @nodoc


class _UndoNotice implements UndoNotice {
  const _UndoNotice({required this.ticket, required this.message, required this.serial});
  

/// The ticket the library keeps the change under (`Library::undo`).
@override final  int ticket;
/// What the notice says, as the library words it.
@override final  String message;
/// Which notice this is: a later one has a larger number, so a timeout
/// belongs to the notice it was started for.
@override final  int serial;

/// Create a copy of UndoNotice
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$UndoNoticeCopyWith<_UndoNotice> get copyWith => __$UndoNoticeCopyWithImpl<_UndoNotice>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _UndoNotice&&(identical(other.ticket, ticket) || other.ticket == ticket)&&(identical(other.message, message) || other.message == message)&&(identical(other.serial, serial) || other.serial == serial));
}


@override
int get hashCode => Object.hash(runtimeType,ticket,message,serial);

@override
String toString() {
  return 'UndoNotice(ticket: $ticket, message: $message, serial: $serial)';
}


}

/// @nodoc
abstract mixin class _$UndoNoticeCopyWith<$Res> implements $UndoNoticeCopyWith<$Res> {
  factory _$UndoNoticeCopyWith(_UndoNotice value, $Res Function(_UndoNotice) _then) = __$UndoNoticeCopyWithImpl;
@override @useResult
$Res call({
 int ticket, String message, int serial
});




}
/// @nodoc
class __$UndoNoticeCopyWithImpl<$Res>
    implements _$UndoNoticeCopyWith<$Res> {
  __$UndoNoticeCopyWithImpl(this._self, this._then);

  final _UndoNotice _self;
  final $Res Function(_UndoNotice) _then;

/// Create a copy of UndoNotice
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? ticket = null,Object? message = null,Object? serial = null,}) {
  return _then(_UndoNotice(
ticket: null == ticket ? _self.ticket : ticket // ignore: cast_nullable_to_non_nullable
as int,message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,serial: null == serial ? _self.serial : serial // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

// dart format on
