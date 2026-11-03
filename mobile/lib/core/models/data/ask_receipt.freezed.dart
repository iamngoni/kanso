// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'ask_receipt.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AskReceipt {

/// Its number in the answer, from 1.
 int get n; String get noteId; String get title;/// "Pricing · lines 12–14 · 14 Oct".
 String get label;/// The first and last line cited, from 0.
 int get start; int get end;/// The cited lines as they were sent.
 String get quote;
/// Create a copy of AskReceipt
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskReceiptCopyWith<AskReceipt> get copyWith => _$AskReceiptCopyWithImpl<AskReceipt>(this as AskReceipt, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskReceipt&&(identical(other.n, n) || other.n == n)&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.title, title) || other.title == title)&&(identical(other.label, label) || other.label == label)&&(identical(other.start, start) || other.start == start)&&(identical(other.end, end) || other.end == end)&&(identical(other.quote, quote) || other.quote == quote));
}


@override
int get hashCode => Object.hash(runtimeType,n,noteId,title,label,start,end,quote);

@override
String toString() {
  return 'AskReceipt(n: $n, noteId: $noteId, title: $title, label: $label, start: $start, end: $end, quote: $quote)';
}


}

/// @nodoc
abstract mixin class $AskReceiptCopyWith<$Res>  {
  factory $AskReceiptCopyWith(AskReceipt value, $Res Function(AskReceipt) _then) = _$AskReceiptCopyWithImpl;
@useResult
$Res call({
 int n, String noteId, String title, String label, int start, int end, String quote
});




}
/// @nodoc
class _$AskReceiptCopyWithImpl<$Res>
    implements $AskReceiptCopyWith<$Res> {
  _$AskReceiptCopyWithImpl(this._self, this._then);

  final AskReceipt _self;
  final $Res Function(AskReceipt) _then;

/// Create a copy of AskReceipt
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? n = null,Object? noteId = null,Object? title = null,Object? label = null,Object? start = null,Object? end = null,Object? quote = null,}) {
  return _then(_self.copyWith(
n: null == n ? _self.n : n // ignore: cast_nullable_to_non_nullable
as int,noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,label: null == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String,start: null == start ? _self.start : start // ignore: cast_nullable_to_non_nullable
as int,end: null == end ? _self.end : end // ignore: cast_nullable_to_non_nullable
as int,quote: null == quote ? _self.quote : quote // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [AskReceipt].
extension AskReceiptPatterns on AskReceipt {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AskReceipt value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AskReceipt() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AskReceipt value)  $default,){
final _that = this;
switch (_that) {
case _AskReceipt():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AskReceipt value)?  $default,){
final _that = this;
switch (_that) {
case _AskReceipt() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( int n,  String noteId,  String title,  String label,  int start,  int end,  String quote)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AskReceipt() when $default != null:
return $default(_that.n,_that.noteId,_that.title,_that.label,_that.start,_that.end,_that.quote);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( int n,  String noteId,  String title,  String label,  int start,  int end,  String quote)  $default,) {final _that = this;
switch (_that) {
case _AskReceipt():
return $default(_that.n,_that.noteId,_that.title,_that.label,_that.start,_that.end,_that.quote);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( int n,  String noteId,  String title,  String label,  int start,  int end,  String quote)?  $default,) {final _that = this;
switch (_that) {
case _AskReceipt() when $default != null:
return $default(_that.n,_that.noteId,_that.title,_that.label,_that.start,_that.end,_that.quote);case _:
  return null;

}
}

}

/// @nodoc


class _AskReceipt implements AskReceipt {
  const _AskReceipt({required this.n, required this.noteId, this.title = '', this.label = '', this.start = 0, this.end = 0, this.quote = ''});
  

/// Its number in the answer, from 1.
@override final  int n;
@override final  String noteId;
@override@JsonKey() final  String title;
/// "Pricing · lines 12–14 · 14 Oct".
@override@JsonKey() final  String label;
/// The first and last line cited, from 0.
@override@JsonKey() final  int start;
@override@JsonKey() final  int end;
/// The cited lines as they were sent.
@override@JsonKey() final  String quote;

/// Create a copy of AskReceipt
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AskReceiptCopyWith<_AskReceipt> get copyWith => __$AskReceiptCopyWithImpl<_AskReceipt>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AskReceipt&&(identical(other.n, n) || other.n == n)&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.title, title) || other.title == title)&&(identical(other.label, label) || other.label == label)&&(identical(other.start, start) || other.start == start)&&(identical(other.end, end) || other.end == end)&&(identical(other.quote, quote) || other.quote == quote));
}


@override
int get hashCode => Object.hash(runtimeType,n,noteId,title,label,start,end,quote);

@override
String toString() {
  return 'AskReceipt(n: $n, noteId: $noteId, title: $title, label: $label, start: $start, end: $end, quote: $quote)';
}


}

/// @nodoc
abstract mixin class _$AskReceiptCopyWith<$Res> implements $AskReceiptCopyWith<$Res> {
  factory _$AskReceiptCopyWith(_AskReceipt value, $Res Function(_AskReceipt) _then) = __$AskReceiptCopyWithImpl;
@override @useResult
$Res call({
 int n, String noteId, String title, String label, int start, int end, String quote
});




}
/// @nodoc
class __$AskReceiptCopyWithImpl<$Res>
    implements _$AskReceiptCopyWith<$Res> {
  __$AskReceiptCopyWithImpl(this._self, this._then);

  final _AskReceipt _self;
  final $Res Function(_AskReceipt) _then;

/// Create a copy of AskReceipt
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? n = null,Object? noteId = null,Object? title = null,Object? label = null,Object? start = null,Object? end = null,Object? quote = null,}) {
  return _then(_AskReceipt(
n: null == n ? _self.n : n // ignore: cast_nullable_to_non_nullable
as int,noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,label: null == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String,start: null == start ? _self.start : start // ignore: cast_nullable_to_non_nullable
as int,end: null == end ? _self.end : end // ignore: cast_nullable_to_non_nullable
as int,quote: null == quote ? _self.quote : quote // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
