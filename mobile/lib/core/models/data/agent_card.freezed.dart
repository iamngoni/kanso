// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'agent_card.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AgentCard {

 int get index; AgentCardKind get kind; String get title; String get book; bool get ticked;/// The note changed since the agent read it.
 bool get changedSince;/// Not one of the notes the person chose.
 bool get outside; bool get published;/// `+ added`, `- removed` or plain lines.
 List<String> get lines;
/// Create a copy of AgentCard
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentCardCopyWith<AgentCard> get copyWith => _$AgentCardCopyWithImpl<AgentCard>(this as AgentCard, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentCard&&(identical(other.index, index) || other.index == index)&&(identical(other.kind, kind) || other.kind == kind)&&(identical(other.title, title) || other.title == title)&&(identical(other.book, book) || other.book == book)&&(identical(other.ticked, ticked) || other.ticked == ticked)&&(identical(other.changedSince, changedSince) || other.changedSince == changedSince)&&(identical(other.outside, outside) || other.outside == outside)&&(identical(other.published, published) || other.published == published)&&const DeepCollectionEquality().equals(other.lines, lines));
}


@override
int get hashCode => Object.hash(runtimeType,index,kind,title,book,ticked,changedSince,outside,published,const DeepCollectionEquality().hash(lines));

@override
String toString() {
  return 'AgentCard(index: $index, kind: $kind, title: $title, book: $book, ticked: $ticked, changedSince: $changedSince, outside: $outside, published: $published, lines: $lines)';
}


}

/// @nodoc
abstract mixin class $AgentCardCopyWith<$Res>  {
  factory $AgentCardCopyWith(AgentCard value, $Res Function(AgentCard) _then) = _$AgentCardCopyWithImpl;
@useResult
$Res call({
 int index, AgentCardKind kind, String title, String book, bool ticked, bool changedSince, bool outside, bool published, List<String> lines
});




}
/// @nodoc
class _$AgentCardCopyWithImpl<$Res>
    implements $AgentCardCopyWith<$Res> {
  _$AgentCardCopyWithImpl(this._self, this._then);

  final AgentCard _self;
  final $Res Function(AgentCard) _then;

/// Create a copy of AgentCard
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? index = null,Object? kind = null,Object? title = null,Object? book = null,Object? ticked = null,Object? changedSince = null,Object? outside = null,Object? published = null,Object? lines = null,}) {
  return _then(_self.copyWith(
index: null == index ? _self.index : index // ignore: cast_nullable_to_non_nullable
as int,kind: null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as AgentCardKind,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,book: null == book ? _self.book : book // ignore: cast_nullable_to_non_nullable
as String,ticked: null == ticked ? _self.ticked : ticked // ignore: cast_nullable_to_non_nullable
as bool,changedSince: null == changedSince ? _self.changedSince : changedSince // ignore: cast_nullable_to_non_nullable
as bool,outside: null == outside ? _self.outside : outside // ignore: cast_nullable_to_non_nullable
as bool,published: null == published ? _self.published : published // ignore: cast_nullable_to_non_nullable
as bool,lines: null == lines ? _self.lines : lines // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}

}


/// Adds pattern-matching-related methods to [AgentCard].
extension AgentCardPatterns on AgentCard {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AgentCard value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AgentCard() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AgentCard value)  $default,){
final _that = this;
switch (_that) {
case _AgentCard():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AgentCard value)?  $default,){
final _that = this;
switch (_that) {
case _AgentCard() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( int index,  AgentCardKind kind,  String title,  String book,  bool ticked,  bool changedSince,  bool outside,  bool published,  List<String> lines)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AgentCard() when $default != null:
return $default(_that.index,_that.kind,_that.title,_that.book,_that.ticked,_that.changedSince,_that.outside,_that.published,_that.lines);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( int index,  AgentCardKind kind,  String title,  String book,  bool ticked,  bool changedSince,  bool outside,  bool published,  List<String> lines)  $default,) {final _that = this;
switch (_that) {
case _AgentCard():
return $default(_that.index,_that.kind,_that.title,_that.book,_that.ticked,_that.changedSince,_that.outside,_that.published,_that.lines);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( int index,  AgentCardKind kind,  String title,  String book,  bool ticked,  bool changedSince,  bool outside,  bool published,  List<String> lines)?  $default,) {final _that = this;
switch (_that) {
case _AgentCard() when $default != null:
return $default(_that.index,_that.kind,_that.title,_that.book,_that.ticked,_that.changedSince,_that.outside,_that.published,_that.lines);case _:
  return null;

}
}

}

/// @nodoc


class _AgentCard implements AgentCard {
  const _AgentCard({required this.index, required this.kind, required this.title, this.book = '', this.ticked = false, this.changedSince = false, this.outside = false, this.published = false, final  List<String> lines = const <String>[]}): _lines = lines;
  

@override final  int index;
@override final  AgentCardKind kind;
@override final  String title;
@override@JsonKey() final  String book;
@override@JsonKey() final  bool ticked;
/// The note changed since the agent read it.
@override@JsonKey() final  bool changedSince;
/// Not one of the notes the person chose.
@override@JsonKey() final  bool outside;
@override@JsonKey() final  bool published;
/// `+ added`, `- removed` or plain lines.
 final  List<String> _lines;
/// `+ added`, `- removed` or plain lines.
@override@JsonKey() List<String> get lines {
  if (_lines is EqualUnmodifiableListView) return _lines;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_lines);
}


/// Create a copy of AgentCard
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AgentCardCopyWith<_AgentCard> get copyWith => __$AgentCardCopyWithImpl<_AgentCard>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AgentCard&&(identical(other.index, index) || other.index == index)&&(identical(other.kind, kind) || other.kind == kind)&&(identical(other.title, title) || other.title == title)&&(identical(other.book, book) || other.book == book)&&(identical(other.ticked, ticked) || other.ticked == ticked)&&(identical(other.changedSince, changedSince) || other.changedSince == changedSince)&&(identical(other.outside, outside) || other.outside == outside)&&(identical(other.published, published) || other.published == published)&&const DeepCollectionEquality().equals(other._lines, _lines));
}


@override
int get hashCode => Object.hash(runtimeType,index,kind,title,book,ticked,changedSince,outside,published,const DeepCollectionEquality().hash(_lines));

@override
String toString() {
  return 'AgentCard(index: $index, kind: $kind, title: $title, book: $book, ticked: $ticked, changedSince: $changedSince, outside: $outside, published: $published, lines: $lines)';
}


}

/// @nodoc
abstract mixin class _$AgentCardCopyWith<$Res> implements $AgentCardCopyWith<$Res> {
  factory _$AgentCardCopyWith(_AgentCard value, $Res Function(_AgentCard) _then) = __$AgentCardCopyWithImpl;
@override @useResult
$Res call({
 int index, AgentCardKind kind, String title, String book, bool ticked, bool changedSince, bool outside, bool published, List<String> lines
});




}
/// @nodoc
class __$AgentCardCopyWithImpl<$Res>
    implements _$AgentCardCopyWith<$Res> {
  __$AgentCardCopyWithImpl(this._self, this._then);

  final _AgentCard _self;
  final $Res Function(_AgentCard) _then;

/// Create a copy of AgentCard
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? index = null,Object? kind = null,Object? title = null,Object? book = null,Object? ticked = null,Object? changedSince = null,Object? outside = null,Object? published = null,Object? lines = null,}) {
  return _then(_AgentCard(
index: null == index ? _self.index : index // ignore: cast_nullable_to_non_nullable
as int,kind: null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as AgentCardKind,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,book: null == book ? _self.book : book // ignore: cast_nullable_to_non_nullable
as String,ticked: null == ticked ? _self.ticked : ticked // ignore: cast_nullable_to_non_nullable
as bool,changedSince: null == changedSince ? _self.changedSince : changedSince // ignore: cast_nullable_to_non_nullable
as bool,outside: null == outside ? _self.outside : outside // ignore: cast_nullable_to_non_nullable
as bool,published: null == published ? _self.published : published // ignore: cast_nullable_to_non_nullable
as bool,lines: null == lines ? _self._lines : lines // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

// dart format on
