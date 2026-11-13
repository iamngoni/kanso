// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'template_picker_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$TemplatePickerEvent {

 String? get notebookId;
/// Create a copy of TemplatePickerEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TemplatePickerEventCopyWith<TemplatePickerEvent> get copyWith => _$TemplatePickerEventCopyWithImpl<TemplatePickerEvent>(this as TemplatePickerEvent, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TemplatePickerEvent&&(identical(other.notebookId, notebookId) || other.notebookId == notebookId));
}


@override
int get hashCode => Object.hash(runtimeType,notebookId);

@override
String toString() {
  return 'TemplatePickerEvent(notebookId: $notebookId)';
}


}

/// @nodoc
abstract mixin class $TemplatePickerEventCopyWith<$Res>  {
  factory $TemplatePickerEventCopyWith(TemplatePickerEvent value, $Res Function(TemplatePickerEvent) _then) = _$TemplatePickerEventCopyWithImpl;
@useResult
$Res call({
 String? notebookId
});




}
/// @nodoc
class _$TemplatePickerEventCopyWithImpl<$Res>
    implements $TemplatePickerEventCopyWith<$Res> {
  _$TemplatePickerEventCopyWithImpl(this._self, this._then);

  final TemplatePickerEvent _self;
  final $Res Function(TemplatePickerEvent) _then;

/// Create a copy of TemplatePickerEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? notebookId = freezed,}) {
  return _then(_self.copyWith(
notebookId: freezed == notebookId ? _self.notebookId : notebookId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

}


/// Adds pattern-matching-related methods to [TemplatePickerEvent].
extension TemplatePickerEventPatterns on TemplatePickerEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( TemplatePickerStartedEvent value)?  started,required TResult orElse(),}){
final _that = this;
switch (_that) {
case TemplatePickerStartedEvent() when started != null:
return started(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( TemplatePickerStartedEvent value)  started,}){
final _that = this;
switch (_that) {
case TemplatePickerStartedEvent():
return started(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( TemplatePickerStartedEvent value)?  started,}){
final _that = this;
switch (_that) {
case TemplatePickerStartedEvent() when started != null:
return started(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String? notebookId)?  started,required TResult orElse(),}) {final _that = this;
switch (_that) {
case TemplatePickerStartedEvent() when started != null:
return started(_that.notebookId);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String? notebookId)  started,}) {final _that = this;
switch (_that) {
case TemplatePickerStartedEvent():
return started(_that.notebookId);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String? notebookId)?  started,}) {final _that = this;
switch (_that) {
case TemplatePickerStartedEvent() when started != null:
return started(_that.notebookId);case _:
  return null;

}
}

}

/// @nodoc


class TemplatePickerStartedEvent implements TemplatePickerEvent {
  const TemplatePickerStartedEvent({this.notebookId});
  

@override final  String? notebookId;

/// Create a copy of TemplatePickerEvent
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TemplatePickerStartedEventCopyWith<TemplatePickerStartedEvent> get copyWith => _$TemplatePickerStartedEventCopyWithImpl<TemplatePickerStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TemplatePickerStartedEvent&&(identical(other.notebookId, notebookId) || other.notebookId == notebookId));
}


@override
int get hashCode => Object.hash(runtimeType,notebookId);

@override
String toString() {
  return 'TemplatePickerEvent.started(notebookId: $notebookId)';
}


}

/// @nodoc
abstract mixin class $TemplatePickerStartedEventCopyWith<$Res> implements $TemplatePickerEventCopyWith<$Res> {
  factory $TemplatePickerStartedEventCopyWith(TemplatePickerStartedEvent value, $Res Function(TemplatePickerStartedEvent) _then) = _$TemplatePickerStartedEventCopyWithImpl;
@override @useResult
$Res call({
 String? notebookId
});




}
/// @nodoc
class _$TemplatePickerStartedEventCopyWithImpl<$Res>
    implements $TemplatePickerStartedEventCopyWith<$Res> {
  _$TemplatePickerStartedEventCopyWithImpl(this._self, this._then);

  final TemplatePickerStartedEvent _self;
  final $Res Function(TemplatePickerStartedEvent) _then;

/// Create a copy of TemplatePickerEvent
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? notebookId = freezed,}) {
  return _then(TemplatePickerStartedEvent(
notebookId: freezed == notebookId ? _self.notebookId : notebookId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc
mixin _$TemplatePickerState {

 LoadStatus get status; List<TemplateRow> get templates; ApplicationError? get error;
/// Create a copy of TemplatePickerState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TemplatePickerStateCopyWith<TemplatePickerState> get copyWith => _$TemplatePickerStateCopyWithImpl<TemplatePickerState>(this as TemplatePickerState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TemplatePickerState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other.templates, templates)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(templates),error);

@override
String toString() {
  return 'TemplatePickerState(status: $status, templates: $templates, error: $error)';
}


}

/// @nodoc
abstract mixin class $TemplatePickerStateCopyWith<$Res>  {
  factory $TemplatePickerStateCopyWith(TemplatePickerState value, $Res Function(TemplatePickerState) _then) = _$TemplatePickerStateCopyWithImpl;
@useResult
$Res call({
 LoadStatus status, List<TemplateRow> templates, ApplicationError? error
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$TemplatePickerStateCopyWithImpl<$Res>
    implements $TemplatePickerStateCopyWith<$Res> {
  _$TemplatePickerStateCopyWithImpl(this._self, this._then);

  final TemplatePickerState _self;
  final $Res Function(TemplatePickerState) _then;

/// Create a copy of TemplatePickerState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? templates = null,Object? error = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,templates: null == templates ? _self.templates : templates // ignore: cast_nullable_to_non_nullable
as List<TemplateRow>,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of TemplatePickerState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get error {
    if (_self.error == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.error!, (value) {
    return _then(_self.copyWith(error: value));
  });
}
}


/// Adds pattern-matching-related methods to [TemplatePickerState].
extension TemplatePickerStatePatterns on TemplatePickerState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _TemplatePickerState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _TemplatePickerState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _TemplatePickerState value)  $default,){
final _that = this;
switch (_that) {
case _TemplatePickerState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _TemplatePickerState value)?  $default,){
final _that = this;
switch (_that) {
case _TemplatePickerState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LoadStatus status,  List<TemplateRow> templates,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _TemplatePickerState() when $default != null:
return $default(_that.status,_that.templates,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LoadStatus status,  List<TemplateRow> templates,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _TemplatePickerState():
return $default(_that.status,_that.templates,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LoadStatus status,  List<TemplateRow> templates,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _TemplatePickerState() when $default != null:
return $default(_that.status,_that.templates,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _TemplatePickerState implements TemplatePickerState {
  const _TemplatePickerState({this.status = LoadStatus.loading, final  List<TemplateRow> templates = const <TemplateRow>[], this.error}): _templates = templates;
  

@override@JsonKey() final  LoadStatus status;
 final  List<TemplateRow> _templates;
@override@JsonKey() List<TemplateRow> get templates {
  if (_templates is EqualUnmodifiableListView) return _templates;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_templates);
}

@override final  ApplicationError? error;

/// Create a copy of TemplatePickerState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$TemplatePickerStateCopyWith<_TemplatePickerState> get copyWith => __$TemplatePickerStateCopyWithImpl<_TemplatePickerState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _TemplatePickerState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other._templates, _templates)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(_templates),error);

@override
String toString() {
  return 'TemplatePickerState(status: $status, templates: $templates, error: $error)';
}


}

/// @nodoc
abstract mixin class _$TemplatePickerStateCopyWith<$Res> implements $TemplatePickerStateCopyWith<$Res> {
  factory _$TemplatePickerStateCopyWith(_TemplatePickerState value, $Res Function(_TemplatePickerState) _then) = __$TemplatePickerStateCopyWithImpl;
@override @useResult
$Res call({
 LoadStatus status, List<TemplateRow> templates, ApplicationError? error
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$TemplatePickerStateCopyWithImpl<$Res>
    implements _$TemplatePickerStateCopyWith<$Res> {
  __$TemplatePickerStateCopyWithImpl(this._self, this._then);

  final _TemplatePickerState _self;
  final $Res Function(_TemplatePickerState) _then;

/// Create a copy of TemplatePickerState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? templates = null,Object? error = freezed,}) {
  return _then(_TemplatePickerState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,templates: null == templates ? _self._templates : templates // ignore: cast_nullable_to_non_nullable
as List<TemplateRow>,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of TemplatePickerState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get error {
    if (_self.error == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.error!, (value) {
    return _then(_self.copyWith(error: value));
  });
}
}

// dart format on
