// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'publish_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$PublishEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PublishEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PublishEvent()';
}


}

/// @nodoc
class $PublishEventCopyWith<$Res>  {
$PublishEventCopyWith(PublishEvent _, $Res Function(PublishEvent) __);
}


/// Adds pattern-matching-related methods to [PublishEvent].
extension PublishEventPatterns on PublishEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PublishOpenNoteEvent value)?  openNote,TResult Function( PublishOpenNotebookEvent value)?  openNotebook,TResult Function( PublishSubmitEvent value)?  publish,TResult Function( PublishUnpublishEvent value)?  unpublish,TResult Function( PublishExpiryPickedEvent value)?  expiryPicked,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PublishOpenNoteEvent() when openNote != null:
return openNote(_that);case PublishOpenNotebookEvent() when openNotebook != null:
return openNotebook(_that);case PublishSubmitEvent() when publish != null:
return publish(_that);case PublishUnpublishEvent() when unpublish != null:
return unpublish(_that);case PublishExpiryPickedEvent() when expiryPicked != null:
return expiryPicked(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PublishOpenNoteEvent value)  openNote,required TResult Function( PublishOpenNotebookEvent value)  openNotebook,required TResult Function( PublishSubmitEvent value)  publish,required TResult Function( PublishUnpublishEvent value)  unpublish,required TResult Function( PublishExpiryPickedEvent value)  expiryPicked,}){
final _that = this;
switch (_that) {
case PublishOpenNoteEvent():
return openNote(_that);case PublishOpenNotebookEvent():
return openNotebook(_that);case PublishSubmitEvent():
return publish(_that);case PublishUnpublishEvent():
return unpublish(_that);case PublishExpiryPickedEvent():
return expiryPicked(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PublishOpenNoteEvent value)?  openNote,TResult? Function( PublishOpenNotebookEvent value)?  openNotebook,TResult? Function( PublishSubmitEvent value)?  publish,TResult? Function( PublishUnpublishEvent value)?  unpublish,TResult? Function( PublishExpiryPickedEvent value)?  expiryPicked,}){
final _that = this;
switch (_that) {
case PublishOpenNoteEvent() when openNote != null:
return openNote(_that);case PublishOpenNotebookEvent() when openNotebook != null:
return openNotebook(_that);case PublishSubmitEvent() when publish != null:
return publish(_that);case PublishUnpublishEvent() when unpublish != null:
return unpublish(_that);case PublishExpiryPickedEvent() when expiryPicked != null:
return expiryPicked(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String id,  PaletteDto palette,  PreviewOptions options)?  openNote,TResult Function( String id,  PaletteDto palette,  PreviewOptions options)?  openNotebook,TResult Function( PublishExpiry expiry)?  publish,TResult Function()?  unpublish,TResult Function( PublishExpiry expiry)?  expiryPicked,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PublishOpenNoteEvent() when openNote != null:
return openNote(_that.id,_that.palette,_that.options);case PublishOpenNotebookEvent() when openNotebook != null:
return openNotebook(_that.id,_that.palette,_that.options);case PublishSubmitEvent() when publish != null:
return publish(_that.expiry);case PublishUnpublishEvent() when unpublish != null:
return unpublish();case PublishExpiryPickedEvent() when expiryPicked != null:
return expiryPicked(_that.expiry);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String id,  PaletteDto palette,  PreviewOptions options)  openNote,required TResult Function( String id,  PaletteDto palette,  PreviewOptions options)  openNotebook,required TResult Function( PublishExpiry expiry)  publish,required TResult Function()  unpublish,required TResult Function( PublishExpiry expiry)  expiryPicked,}) {final _that = this;
switch (_that) {
case PublishOpenNoteEvent():
return openNote(_that.id,_that.palette,_that.options);case PublishOpenNotebookEvent():
return openNotebook(_that.id,_that.palette,_that.options);case PublishSubmitEvent():
return publish(_that.expiry);case PublishUnpublishEvent():
return unpublish();case PublishExpiryPickedEvent():
return expiryPicked(_that.expiry);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String id,  PaletteDto palette,  PreviewOptions options)?  openNote,TResult? Function( String id,  PaletteDto palette,  PreviewOptions options)?  openNotebook,TResult? Function( PublishExpiry expiry)?  publish,TResult? Function()?  unpublish,TResult? Function( PublishExpiry expiry)?  expiryPicked,}) {final _that = this;
switch (_that) {
case PublishOpenNoteEvent() when openNote != null:
return openNote(_that.id,_that.palette,_that.options);case PublishOpenNotebookEvent() when openNotebook != null:
return openNotebook(_that.id,_that.palette,_that.options);case PublishSubmitEvent() when publish != null:
return publish(_that.expiry);case PublishUnpublishEvent() when unpublish != null:
return unpublish();case PublishExpiryPickedEvent() when expiryPicked != null:
return expiryPicked(_that.expiry);case _:
  return null;

}
}

}

/// @nodoc


class PublishOpenNoteEvent implements PublishEvent {
  const PublishOpenNoteEvent({required this.id, required this.palette, required this.options});
  

 final  String id;
 final  PaletteDto palette;
 final  PreviewOptions options;

/// Create a copy of PublishEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PublishOpenNoteEventCopyWith<PublishOpenNoteEvent> get copyWith => _$PublishOpenNoteEventCopyWithImpl<PublishOpenNoteEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PublishOpenNoteEvent&&(identical(other.id, id) || other.id == id)&&(identical(other.palette, palette) || other.palette == palette)&&(identical(other.options, options) || other.options == options));
}


@override
int get hashCode => Object.hash(runtimeType,id,palette,options);

@override
String toString() {
  return 'PublishEvent.openNote(id: $id, palette: $palette, options: $options)';
}


}

/// @nodoc
abstract mixin class $PublishOpenNoteEventCopyWith<$Res> implements $PublishEventCopyWith<$Res> {
  factory $PublishOpenNoteEventCopyWith(PublishOpenNoteEvent value, $Res Function(PublishOpenNoteEvent) _then) = _$PublishOpenNoteEventCopyWithImpl;
@useResult
$Res call({
 String id, PaletteDto palette, PreviewOptions options
});




}
/// @nodoc
class _$PublishOpenNoteEventCopyWithImpl<$Res>
    implements $PublishOpenNoteEventCopyWith<$Res> {
  _$PublishOpenNoteEventCopyWithImpl(this._self, this._then);

  final PublishOpenNoteEvent _self;
  final $Res Function(PublishOpenNoteEvent) _then;

/// Create a copy of PublishEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,Object? palette = null,Object? options = null,}) {
  return _then(PublishOpenNoteEvent(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,palette: null == palette ? _self.palette : palette // ignore: cast_nullable_to_non_nullable
as PaletteDto,options: null == options ? _self.options : options // ignore: cast_nullable_to_non_nullable
as PreviewOptions,
  ));
}


}

/// @nodoc


class PublishOpenNotebookEvent implements PublishEvent {
  const PublishOpenNotebookEvent({required this.id, required this.palette, required this.options});
  

 final  String id;
 final  PaletteDto palette;
 final  PreviewOptions options;

/// Create a copy of PublishEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PublishOpenNotebookEventCopyWith<PublishOpenNotebookEvent> get copyWith => _$PublishOpenNotebookEventCopyWithImpl<PublishOpenNotebookEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PublishOpenNotebookEvent&&(identical(other.id, id) || other.id == id)&&(identical(other.palette, palette) || other.palette == palette)&&(identical(other.options, options) || other.options == options));
}


@override
int get hashCode => Object.hash(runtimeType,id,palette,options);

@override
String toString() {
  return 'PublishEvent.openNotebook(id: $id, palette: $palette, options: $options)';
}


}

/// @nodoc
abstract mixin class $PublishOpenNotebookEventCopyWith<$Res> implements $PublishEventCopyWith<$Res> {
  factory $PublishOpenNotebookEventCopyWith(PublishOpenNotebookEvent value, $Res Function(PublishOpenNotebookEvent) _then) = _$PublishOpenNotebookEventCopyWithImpl;
@useResult
$Res call({
 String id, PaletteDto palette, PreviewOptions options
});




}
/// @nodoc
class _$PublishOpenNotebookEventCopyWithImpl<$Res>
    implements $PublishOpenNotebookEventCopyWith<$Res> {
  _$PublishOpenNotebookEventCopyWithImpl(this._self, this._then);

  final PublishOpenNotebookEvent _self;
  final $Res Function(PublishOpenNotebookEvent) _then;

/// Create a copy of PublishEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,Object? palette = null,Object? options = null,}) {
  return _then(PublishOpenNotebookEvent(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,palette: null == palette ? _self.palette : palette // ignore: cast_nullable_to_non_nullable
as PaletteDto,options: null == options ? _self.options : options // ignore: cast_nullable_to_non_nullable
as PreviewOptions,
  ));
}


}

/// @nodoc


class PublishSubmitEvent implements PublishEvent {
  const PublishSubmitEvent(this.expiry);
  

 final  PublishExpiry expiry;

/// Create a copy of PublishEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PublishSubmitEventCopyWith<PublishSubmitEvent> get copyWith => _$PublishSubmitEventCopyWithImpl<PublishSubmitEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PublishSubmitEvent&&(identical(other.expiry, expiry) || other.expiry == expiry));
}


@override
int get hashCode => Object.hash(runtimeType,expiry);

@override
String toString() {
  return 'PublishEvent.publish(expiry: $expiry)';
}


}

/// @nodoc
abstract mixin class $PublishSubmitEventCopyWith<$Res> implements $PublishEventCopyWith<$Res> {
  factory $PublishSubmitEventCopyWith(PublishSubmitEvent value, $Res Function(PublishSubmitEvent) _then) = _$PublishSubmitEventCopyWithImpl;
@useResult
$Res call({
 PublishExpiry expiry
});




}
/// @nodoc
class _$PublishSubmitEventCopyWithImpl<$Res>
    implements $PublishSubmitEventCopyWith<$Res> {
  _$PublishSubmitEventCopyWithImpl(this._self, this._then);

  final PublishSubmitEvent _self;
  final $Res Function(PublishSubmitEvent) _then;

/// Create a copy of PublishEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? expiry = null,}) {
  return _then(PublishSubmitEvent(
null == expiry ? _self.expiry : expiry // ignore: cast_nullable_to_non_nullable
as PublishExpiry,
  ));
}


}

/// @nodoc


class PublishUnpublishEvent implements PublishEvent {
  const PublishUnpublishEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PublishUnpublishEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PublishEvent.unpublish()';
}


}




/// @nodoc


class PublishExpiryPickedEvent implements PublishEvent {
  const PublishExpiryPickedEvent(this.expiry);
  

 final  PublishExpiry expiry;

/// Create a copy of PublishEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PublishExpiryPickedEventCopyWith<PublishExpiryPickedEvent> get copyWith => _$PublishExpiryPickedEventCopyWithImpl<PublishExpiryPickedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PublishExpiryPickedEvent&&(identical(other.expiry, expiry) || other.expiry == expiry));
}


@override
int get hashCode => Object.hash(runtimeType,expiry);

@override
String toString() {
  return 'PublishEvent.expiryPicked(expiry: $expiry)';
}


}

/// @nodoc
abstract mixin class $PublishExpiryPickedEventCopyWith<$Res> implements $PublishEventCopyWith<$Res> {
  factory $PublishExpiryPickedEventCopyWith(PublishExpiryPickedEvent value, $Res Function(PublishExpiryPickedEvent) _then) = _$PublishExpiryPickedEventCopyWithImpl;
@useResult
$Res call({
 PublishExpiry expiry
});




}
/// @nodoc
class _$PublishExpiryPickedEventCopyWithImpl<$Res>
    implements $PublishExpiryPickedEventCopyWith<$Res> {
  _$PublishExpiryPickedEventCopyWithImpl(this._self, this._then);

  final PublishExpiryPickedEvent _self;
  final $Res Function(PublishExpiryPickedEvent) _then;

/// Create a copy of PublishEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? expiry = null,}) {
  return _then(PublishExpiryPickedEvent(
null == expiry ? _self.expiry : expiry // ignore: cast_nullable_to_non_nullable
as PublishExpiry,
  ));
}


}

/// @nodoc
mixin _$PublishState {

 PublishViewModel? get view;
/// Create a copy of PublishState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PublishStateCopyWith<PublishState> get copyWith => _$PublishStateCopyWithImpl<PublishState>(this as PublishState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PublishState&&(identical(other.view, view) || other.view == view));
}


@override
int get hashCode => Object.hash(runtimeType,view);

@override
String toString() {
  return 'PublishState(view: $view)';
}


}

/// @nodoc
abstract mixin class $PublishStateCopyWith<$Res>  {
  factory $PublishStateCopyWith(PublishState value, $Res Function(PublishState) _then) = _$PublishStateCopyWithImpl;
@useResult
$Res call({
 PublishViewModel view
});


$PublishViewModelCopyWith<$Res>? get view;

}
/// @nodoc
class _$PublishStateCopyWithImpl<$Res>
    implements $PublishStateCopyWith<$Res> {
  _$PublishStateCopyWithImpl(this._self, this._then);

  final PublishState _self;
  final $Res Function(PublishState) _then;

/// Create a copy of PublishState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? view = null,}) {
  return _then(_self.copyWith(
view: null == view ? _self.view! : view // ignore: cast_nullable_to_non_nullable
as PublishViewModel,
  ));
}
/// Create a copy of PublishState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$PublishViewModelCopyWith<$Res>? get view {
    if (_self.view == null) {
    return null;
  }

  return $PublishViewModelCopyWith<$Res>(_self.view!, (value) {
    return _then(_self.copyWith(view: value));
  });
}
}


/// Adds pattern-matching-related methods to [PublishState].
extension PublishStatePatterns on PublishState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PublishIdleState value)?  idle,TResult Function( PublishPublishingState value)?  publishing,TResult Function( PublishPublishedState value)?  published,TResult Function( PublishFailedState value)?  failed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PublishIdleState() when idle != null:
return idle(_that);case PublishPublishingState() when publishing != null:
return publishing(_that);case PublishPublishedState() when published != null:
return published(_that);case PublishFailedState() when failed != null:
return failed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PublishIdleState value)  idle,required TResult Function( PublishPublishingState value)  publishing,required TResult Function( PublishPublishedState value)  published,required TResult Function( PublishFailedState value)  failed,}){
final _that = this;
switch (_that) {
case PublishIdleState():
return idle(_that);case PublishPublishingState():
return publishing(_that);case PublishPublishedState():
return published(_that);case PublishFailedState():
return failed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PublishIdleState value)?  idle,TResult? Function( PublishPublishingState value)?  publishing,TResult? Function( PublishPublishedState value)?  published,TResult? Function( PublishFailedState value)?  failed,}){
final _that = this;
switch (_that) {
case PublishIdleState() when idle != null:
return idle(_that);case PublishPublishingState() when publishing != null:
return publishing(_that);case PublishPublishedState() when published != null:
return published(_that);case PublishFailedState() when failed != null:
return failed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( PublishViewModel? view)?  idle,TResult Function( PublishViewModel view)?  publishing,TResult Function( PublishViewModel view,  PublishLink link)?  published,TResult Function( PublishViewModel view,  String text)?  failed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PublishIdleState() when idle != null:
return idle(_that.view);case PublishPublishingState() when publishing != null:
return publishing(_that.view);case PublishPublishedState() when published != null:
return published(_that.view,_that.link);case PublishFailedState() when failed != null:
return failed(_that.view,_that.text);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( PublishViewModel? view)  idle,required TResult Function( PublishViewModel view)  publishing,required TResult Function( PublishViewModel view,  PublishLink link)  published,required TResult Function( PublishViewModel view,  String text)  failed,}) {final _that = this;
switch (_that) {
case PublishIdleState():
return idle(_that.view);case PublishPublishingState():
return publishing(_that.view);case PublishPublishedState():
return published(_that.view,_that.link);case PublishFailedState():
return failed(_that.view,_that.text);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( PublishViewModel? view)?  idle,TResult? Function( PublishViewModel view)?  publishing,TResult? Function( PublishViewModel view,  PublishLink link)?  published,TResult? Function( PublishViewModel view,  String text)?  failed,}) {final _that = this;
switch (_that) {
case PublishIdleState() when idle != null:
return idle(_that.view);case PublishPublishingState() when publishing != null:
return publishing(_that.view);case PublishPublishedState() when published != null:
return published(_that.view,_that.link);case PublishFailedState() when failed != null:
return failed(_that.view,_that.text);case _:
  return null;

}
}

}

/// @nodoc


class PublishIdleState implements PublishState {
  const PublishIdleState({this.view});
  

@override final  PublishViewModel? view;

/// Create a copy of PublishState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PublishIdleStateCopyWith<PublishIdleState> get copyWith => _$PublishIdleStateCopyWithImpl<PublishIdleState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PublishIdleState&&(identical(other.view, view) || other.view == view));
}


@override
int get hashCode => Object.hash(runtimeType,view);

@override
String toString() {
  return 'PublishState.idle(view: $view)';
}


}

/// @nodoc
abstract mixin class $PublishIdleStateCopyWith<$Res> implements $PublishStateCopyWith<$Res> {
  factory $PublishIdleStateCopyWith(PublishIdleState value, $Res Function(PublishIdleState) _then) = _$PublishIdleStateCopyWithImpl;
@override @useResult
$Res call({
 PublishViewModel? view
});


@override $PublishViewModelCopyWith<$Res>? get view;

}
/// @nodoc
class _$PublishIdleStateCopyWithImpl<$Res>
    implements $PublishIdleStateCopyWith<$Res> {
  _$PublishIdleStateCopyWithImpl(this._self, this._then);

  final PublishIdleState _self;
  final $Res Function(PublishIdleState) _then;

/// Create a copy of PublishState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? view = freezed,}) {
  return _then(PublishIdleState(
view: freezed == view ? _self.view : view // ignore: cast_nullable_to_non_nullable
as PublishViewModel?,
  ));
}

/// Create a copy of PublishState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$PublishViewModelCopyWith<$Res>? get view {
    if (_self.view == null) {
    return null;
  }

  return $PublishViewModelCopyWith<$Res>(_self.view!, (value) {
    return _then(_self.copyWith(view: value));
  });
}
}

/// @nodoc


class PublishPublishingState implements PublishState {
  const PublishPublishingState({required this.view});
  

@override final  PublishViewModel view;

/// Create a copy of PublishState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PublishPublishingStateCopyWith<PublishPublishingState> get copyWith => _$PublishPublishingStateCopyWithImpl<PublishPublishingState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PublishPublishingState&&(identical(other.view, view) || other.view == view));
}


@override
int get hashCode => Object.hash(runtimeType,view);

@override
String toString() {
  return 'PublishState.publishing(view: $view)';
}


}

/// @nodoc
abstract mixin class $PublishPublishingStateCopyWith<$Res> implements $PublishStateCopyWith<$Res> {
  factory $PublishPublishingStateCopyWith(PublishPublishingState value, $Res Function(PublishPublishingState) _then) = _$PublishPublishingStateCopyWithImpl;
@override @useResult
$Res call({
 PublishViewModel view
});


@override $PublishViewModelCopyWith<$Res> get view;

}
/// @nodoc
class _$PublishPublishingStateCopyWithImpl<$Res>
    implements $PublishPublishingStateCopyWith<$Res> {
  _$PublishPublishingStateCopyWithImpl(this._self, this._then);

  final PublishPublishingState _self;
  final $Res Function(PublishPublishingState) _then;

/// Create a copy of PublishState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? view = null,}) {
  return _then(PublishPublishingState(
view: null == view ? _self.view : view // ignore: cast_nullable_to_non_nullable
as PublishViewModel,
  ));
}

/// Create a copy of PublishState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$PublishViewModelCopyWith<$Res> get view {
  
  return $PublishViewModelCopyWith<$Res>(_self.view, (value) {
    return _then(_self.copyWith(view: value));
  });
}
}

/// @nodoc


class PublishPublishedState implements PublishState {
  const PublishPublishedState({required this.view, required this.link});
  

@override final  PublishViewModel view;
 final  PublishLink link;

/// Create a copy of PublishState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PublishPublishedStateCopyWith<PublishPublishedState> get copyWith => _$PublishPublishedStateCopyWithImpl<PublishPublishedState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PublishPublishedState&&(identical(other.view, view) || other.view == view)&&(identical(other.link, link) || other.link == link));
}


@override
int get hashCode => Object.hash(runtimeType,view,link);

@override
String toString() {
  return 'PublishState.published(view: $view, link: $link)';
}


}

/// @nodoc
abstract mixin class $PublishPublishedStateCopyWith<$Res> implements $PublishStateCopyWith<$Res> {
  factory $PublishPublishedStateCopyWith(PublishPublishedState value, $Res Function(PublishPublishedState) _then) = _$PublishPublishedStateCopyWithImpl;
@override @useResult
$Res call({
 PublishViewModel view, PublishLink link
});


@override $PublishViewModelCopyWith<$Res> get view;

}
/// @nodoc
class _$PublishPublishedStateCopyWithImpl<$Res>
    implements $PublishPublishedStateCopyWith<$Res> {
  _$PublishPublishedStateCopyWithImpl(this._self, this._then);

  final PublishPublishedState _self;
  final $Res Function(PublishPublishedState) _then;

/// Create a copy of PublishState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? view = null,Object? link = null,}) {
  return _then(PublishPublishedState(
view: null == view ? _self.view : view // ignore: cast_nullable_to_non_nullable
as PublishViewModel,link: null == link ? _self.link : link // ignore: cast_nullable_to_non_nullable
as PublishLink,
  ));
}

/// Create a copy of PublishState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$PublishViewModelCopyWith<$Res> get view {
  
  return $PublishViewModelCopyWith<$Res>(_self.view, (value) {
    return _then(_self.copyWith(view: value));
  });
}
}

/// @nodoc


class PublishFailedState implements PublishState {
  const PublishFailedState({required this.view, required this.text});
  

@override final  PublishViewModel view;
 final  String text;

/// Create a copy of PublishState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PublishFailedStateCopyWith<PublishFailedState> get copyWith => _$PublishFailedStateCopyWithImpl<PublishFailedState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PublishFailedState&&(identical(other.view, view) || other.view == view)&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,view,text);

@override
String toString() {
  return 'PublishState.failed(view: $view, text: $text)';
}


}

/// @nodoc
abstract mixin class $PublishFailedStateCopyWith<$Res> implements $PublishStateCopyWith<$Res> {
  factory $PublishFailedStateCopyWith(PublishFailedState value, $Res Function(PublishFailedState) _then) = _$PublishFailedStateCopyWithImpl;
@override @useResult
$Res call({
 PublishViewModel view, String text
});


@override $PublishViewModelCopyWith<$Res> get view;

}
/// @nodoc
class _$PublishFailedStateCopyWithImpl<$Res>
    implements $PublishFailedStateCopyWith<$Res> {
  _$PublishFailedStateCopyWithImpl(this._self, this._then);

  final PublishFailedState _self;
  final $Res Function(PublishFailedState) _then;

/// Create a copy of PublishState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? view = null,Object? text = null,}) {
  return _then(PublishFailedState(
view: null == view ? _self.view : view // ignore: cast_nullable_to_non_nullable
as PublishViewModel,text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

/// Create a copy of PublishState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$PublishViewModelCopyWith<$Res> get view {
  
  return $PublishViewModelCopyWith<$Res>(_self.view, (value) {
    return _then(_self.copyWith(view: value));
  });
}
}

// dart format on
