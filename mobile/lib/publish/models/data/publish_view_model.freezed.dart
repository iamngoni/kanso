// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'publish_view_model.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$PublishViewModel {

 String get id; String get title; bool get notebook; String get summary; String get contentPreview; bool get needsUpdate; PublishExpiry get expiry; List<PublishExpiryOption> get expiryOptions; String get locked; String? get refusal; PublishLink? get link;
/// Create a copy of PublishViewModel
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PublishViewModelCopyWith<PublishViewModel> get copyWith => _$PublishViewModelCopyWithImpl<PublishViewModel>(this as PublishViewModel, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PublishViewModel&&(identical(other.id, id) || other.id == id)&&(identical(other.title, title) || other.title == title)&&(identical(other.notebook, notebook) || other.notebook == notebook)&&(identical(other.summary, summary) || other.summary == summary)&&(identical(other.contentPreview, contentPreview) || other.contentPreview == contentPreview)&&(identical(other.needsUpdate, needsUpdate) || other.needsUpdate == needsUpdate)&&(identical(other.expiry, expiry) || other.expiry == expiry)&&const DeepCollectionEquality().equals(other.expiryOptions, expiryOptions)&&(identical(other.locked, locked) || other.locked == locked)&&(identical(other.refusal, refusal) || other.refusal == refusal)&&(identical(other.link, link) || other.link == link));
}


@override
int get hashCode => Object.hash(runtimeType,id,title,notebook,summary,contentPreview,needsUpdate,expiry,const DeepCollectionEquality().hash(expiryOptions),locked,refusal,link);

@override
String toString() {
  return 'PublishViewModel(id: $id, title: $title, notebook: $notebook, summary: $summary, contentPreview: $contentPreview, needsUpdate: $needsUpdate, expiry: $expiry, expiryOptions: $expiryOptions, locked: $locked, refusal: $refusal, link: $link)';
}


}

/// @nodoc
abstract mixin class $PublishViewModelCopyWith<$Res>  {
  factory $PublishViewModelCopyWith(PublishViewModel value, $Res Function(PublishViewModel) _then) = _$PublishViewModelCopyWithImpl;
@useResult
$Res call({
 String id, String title, bool notebook, String summary, String contentPreview, bool needsUpdate, PublishExpiry expiry, List<PublishExpiryOption> expiryOptions, String locked, String? refusal, PublishLink? link
});




}
/// @nodoc
class _$PublishViewModelCopyWithImpl<$Res>
    implements $PublishViewModelCopyWith<$Res> {
  _$PublishViewModelCopyWithImpl(this._self, this._then);

  final PublishViewModel _self;
  final $Res Function(PublishViewModel) _then;

/// Create a copy of PublishViewModel
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? title = null,Object? notebook = null,Object? summary = null,Object? contentPreview = null,Object? needsUpdate = null,Object? expiry = null,Object? expiryOptions = null,Object? locked = null,Object? refusal = freezed,Object? link = freezed,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,notebook: null == notebook ? _self.notebook : notebook // ignore: cast_nullable_to_non_nullable
as bool,summary: null == summary ? _self.summary : summary // ignore: cast_nullable_to_non_nullable
as String,contentPreview: null == contentPreview ? _self.contentPreview : contentPreview // ignore: cast_nullable_to_non_nullable
as String,needsUpdate: null == needsUpdate ? _self.needsUpdate : needsUpdate // ignore: cast_nullable_to_non_nullable
as bool,expiry: null == expiry ? _self.expiry : expiry // ignore: cast_nullable_to_non_nullable
as PublishExpiry,expiryOptions: null == expiryOptions ? _self.expiryOptions : expiryOptions // ignore: cast_nullable_to_non_nullable
as List<PublishExpiryOption>,locked: null == locked ? _self.locked : locked // ignore: cast_nullable_to_non_nullable
as String,refusal: freezed == refusal ? _self.refusal : refusal // ignore: cast_nullable_to_non_nullable
as String?,link: freezed == link ? _self.link : link // ignore: cast_nullable_to_non_nullable
as PublishLink?,
  ));
}

}


/// Adds pattern-matching-related methods to [PublishViewModel].
extension PublishViewModelPatterns on PublishViewModel {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _PublishViewModel value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _PublishViewModel() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _PublishViewModel value)  $default,){
final _that = this;
switch (_that) {
case _PublishViewModel():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _PublishViewModel value)?  $default,){
final _that = this;
switch (_that) {
case _PublishViewModel() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String id,  String title,  bool notebook,  String summary,  String contentPreview,  bool needsUpdate,  PublishExpiry expiry,  List<PublishExpiryOption> expiryOptions,  String locked,  String? refusal,  PublishLink? link)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _PublishViewModel() when $default != null:
return $default(_that.id,_that.title,_that.notebook,_that.summary,_that.contentPreview,_that.needsUpdate,_that.expiry,_that.expiryOptions,_that.locked,_that.refusal,_that.link);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String id,  String title,  bool notebook,  String summary,  String contentPreview,  bool needsUpdate,  PublishExpiry expiry,  List<PublishExpiryOption> expiryOptions,  String locked,  String? refusal,  PublishLink? link)  $default,) {final _that = this;
switch (_that) {
case _PublishViewModel():
return $default(_that.id,_that.title,_that.notebook,_that.summary,_that.contentPreview,_that.needsUpdate,_that.expiry,_that.expiryOptions,_that.locked,_that.refusal,_that.link);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String id,  String title,  bool notebook,  String summary,  String contentPreview,  bool needsUpdate,  PublishExpiry expiry,  List<PublishExpiryOption> expiryOptions,  String locked,  String? refusal,  PublishLink? link)?  $default,) {final _that = this;
switch (_that) {
case _PublishViewModel() when $default != null:
return $default(_that.id,_that.title,_that.notebook,_that.summary,_that.contentPreview,_that.needsUpdate,_that.expiry,_that.expiryOptions,_that.locked,_that.refusal,_that.link);case _:
  return null;

}
}

}

/// @nodoc


class _PublishViewModel implements PublishViewModel {
  const _PublishViewModel({required this.id, required this.title, required this.notebook, required this.summary, this.contentPreview = '', this.needsUpdate = false, this.expiry = PublishExpiry.never, final  List<PublishExpiryOption> expiryOptions = const <PublishExpiryOption>[], this.locked = '', this.refusal, this.link}): _expiryOptions = expiryOptions;
  

@override final  String id;
@override final  String title;
@override final  bool notebook;
@override final  String summary;
@override@JsonKey() final  String contentPreview;
@override@JsonKey() final  bool needsUpdate;
@override@JsonKey() final  PublishExpiry expiry;
 final  List<PublishExpiryOption> _expiryOptions;
@override@JsonKey() List<PublishExpiryOption> get expiryOptions {
  if (_expiryOptions is EqualUnmodifiableListView) return _expiryOptions;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_expiryOptions);
}

@override@JsonKey() final  String locked;
@override final  String? refusal;
@override final  PublishLink? link;

/// Create a copy of PublishViewModel
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$PublishViewModelCopyWith<_PublishViewModel> get copyWith => __$PublishViewModelCopyWithImpl<_PublishViewModel>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _PublishViewModel&&(identical(other.id, id) || other.id == id)&&(identical(other.title, title) || other.title == title)&&(identical(other.notebook, notebook) || other.notebook == notebook)&&(identical(other.summary, summary) || other.summary == summary)&&(identical(other.contentPreview, contentPreview) || other.contentPreview == contentPreview)&&(identical(other.needsUpdate, needsUpdate) || other.needsUpdate == needsUpdate)&&(identical(other.expiry, expiry) || other.expiry == expiry)&&const DeepCollectionEquality().equals(other._expiryOptions, _expiryOptions)&&(identical(other.locked, locked) || other.locked == locked)&&(identical(other.refusal, refusal) || other.refusal == refusal)&&(identical(other.link, link) || other.link == link));
}


@override
int get hashCode => Object.hash(runtimeType,id,title,notebook,summary,contentPreview,needsUpdate,expiry,const DeepCollectionEquality().hash(_expiryOptions),locked,refusal,link);

@override
String toString() {
  return 'PublishViewModel(id: $id, title: $title, notebook: $notebook, summary: $summary, contentPreview: $contentPreview, needsUpdate: $needsUpdate, expiry: $expiry, expiryOptions: $expiryOptions, locked: $locked, refusal: $refusal, link: $link)';
}


}

/// @nodoc
abstract mixin class _$PublishViewModelCopyWith<$Res> implements $PublishViewModelCopyWith<$Res> {
  factory _$PublishViewModelCopyWith(_PublishViewModel value, $Res Function(_PublishViewModel) _then) = __$PublishViewModelCopyWithImpl;
@override @useResult
$Res call({
 String id, String title, bool notebook, String summary, String contentPreview, bool needsUpdate, PublishExpiry expiry, List<PublishExpiryOption> expiryOptions, String locked, String? refusal, PublishLink? link
});




}
/// @nodoc
class __$PublishViewModelCopyWithImpl<$Res>
    implements _$PublishViewModelCopyWith<$Res> {
  __$PublishViewModelCopyWithImpl(this._self, this._then);

  final _PublishViewModel _self;
  final $Res Function(_PublishViewModel) _then;

/// Create a copy of PublishViewModel
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? title = null,Object? notebook = null,Object? summary = null,Object? contentPreview = null,Object? needsUpdate = null,Object? expiry = null,Object? expiryOptions = null,Object? locked = null,Object? refusal = freezed,Object? link = freezed,}) {
  return _then(_PublishViewModel(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,notebook: null == notebook ? _self.notebook : notebook // ignore: cast_nullable_to_non_nullable
as bool,summary: null == summary ? _self.summary : summary // ignore: cast_nullable_to_non_nullable
as String,contentPreview: null == contentPreview ? _self.contentPreview : contentPreview // ignore: cast_nullable_to_non_nullable
as String,needsUpdate: null == needsUpdate ? _self.needsUpdate : needsUpdate // ignore: cast_nullable_to_non_nullable
as bool,expiry: null == expiry ? _self.expiry : expiry // ignore: cast_nullable_to_non_nullable
as PublishExpiry,expiryOptions: null == expiryOptions ? _self._expiryOptions : expiryOptions // ignore: cast_nullable_to_non_nullable
as List<PublishExpiryOption>,locked: null == locked ? _self.locked : locked // ignore: cast_nullable_to_non_nullable
as String,refusal: freezed == refusal ? _self.refusal : refusal // ignore: cast_nullable_to_non_nullable
as String?,link: freezed == link ? _self.link : link // ignore: cast_nullable_to_non_nullable
as PublishLink?,
  ));
}


}

// dart format on
