// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'daily_template_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$DailyTemplateEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DailyTemplateEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DailyTemplateEvent()';
}


}

/// @nodoc
class $DailyTemplateEventCopyWith<$Res>  {
$DailyTemplateEventCopyWith(DailyTemplateEvent _, $Res Function(DailyTemplateEvent) __);
}


/// Adds pattern-matching-related methods to [DailyTemplateEvent].
extension DailyTemplateEventPatterns on DailyTemplateEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( DailyTemplateStartedEvent value)?  started,TResult Function( DailyTemplateChosenEvent value)?  chosen,TResult Function( DailyTemplateErrorDismissedEvent value)?  errorDismissed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case DailyTemplateStartedEvent() when started != null:
return started(_that);case DailyTemplateChosenEvent() when chosen != null:
return chosen(_that);case DailyTemplateErrorDismissedEvent() when errorDismissed != null:
return errorDismissed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( DailyTemplateStartedEvent value)  started,required TResult Function( DailyTemplateChosenEvent value)  chosen,required TResult Function( DailyTemplateErrorDismissedEvent value)  errorDismissed,}){
final _that = this;
switch (_that) {
case DailyTemplateStartedEvent():
return started(_that);case DailyTemplateChosenEvent():
return chosen(_that);case DailyTemplateErrorDismissedEvent():
return errorDismissed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( DailyTemplateStartedEvent value)?  started,TResult? Function( DailyTemplateChosenEvent value)?  chosen,TResult? Function( DailyTemplateErrorDismissedEvent value)?  errorDismissed,}){
final _that = this;
switch (_that) {
case DailyTemplateStartedEvent() when started != null:
return started(_that);case DailyTemplateChosenEvent() when chosen != null:
return chosen(_that);case DailyTemplateErrorDismissedEvent() when errorDismissed != null:
return errorDismissed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function( String? templateId)?  chosen,TResult Function()?  errorDismissed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case DailyTemplateStartedEvent() when started != null:
return started();case DailyTemplateChosenEvent() when chosen != null:
return chosen(_that.templateId);case DailyTemplateErrorDismissedEvent() when errorDismissed != null:
return errorDismissed();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function( String? templateId)  chosen,required TResult Function()  errorDismissed,}) {final _that = this;
switch (_that) {
case DailyTemplateStartedEvent():
return started();case DailyTemplateChosenEvent():
return chosen(_that.templateId);case DailyTemplateErrorDismissedEvent():
return errorDismissed();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function( String? templateId)?  chosen,TResult? Function()?  errorDismissed,}) {final _that = this;
switch (_that) {
case DailyTemplateStartedEvent() when started != null:
return started();case DailyTemplateChosenEvent() when chosen != null:
return chosen(_that.templateId);case DailyTemplateErrorDismissedEvent() when errorDismissed != null:
return errorDismissed();case _:
  return null;

}
}

}

/// @nodoc


class DailyTemplateStartedEvent implements DailyTemplateEvent {
  const DailyTemplateStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DailyTemplateStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DailyTemplateEvent.started()';
}


}




/// @nodoc


class DailyTemplateChosenEvent implements DailyTemplateEvent {
  const DailyTemplateChosenEvent(this.templateId);
  

 final  String? templateId;

/// Create a copy of DailyTemplateEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DailyTemplateChosenEventCopyWith<DailyTemplateChosenEvent> get copyWith => _$DailyTemplateChosenEventCopyWithImpl<DailyTemplateChosenEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DailyTemplateChosenEvent&&(identical(other.templateId, templateId) || other.templateId == templateId));
}


@override
int get hashCode => Object.hash(runtimeType,templateId);

@override
String toString() {
  return 'DailyTemplateEvent.chosen(templateId: $templateId)';
}


}

/// @nodoc
abstract mixin class $DailyTemplateChosenEventCopyWith<$Res> implements $DailyTemplateEventCopyWith<$Res> {
  factory $DailyTemplateChosenEventCopyWith(DailyTemplateChosenEvent value, $Res Function(DailyTemplateChosenEvent) _then) = _$DailyTemplateChosenEventCopyWithImpl;
@useResult
$Res call({
 String? templateId
});




}
/// @nodoc
class _$DailyTemplateChosenEventCopyWithImpl<$Res>
    implements $DailyTemplateChosenEventCopyWith<$Res> {
  _$DailyTemplateChosenEventCopyWithImpl(this._self, this._then);

  final DailyTemplateChosenEvent _self;
  final $Res Function(DailyTemplateChosenEvent) _then;

/// Create a copy of DailyTemplateEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? templateId = freezed,}) {
  return _then(DailyTemplateChosenEvent(
freezed == templateId ? _self.templateId : templateId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class DailyTemplateErrorDismissedEvent implements DailyTemplateEvent {
  const DailyTemplateErrorDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DailyTemplateErrorDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DailyTemplateEvent.errorDismissed()';
}


}




/// @nodoc
mixin _$DailyTemplateState {

 LoadStatus get status; List<TemplateRow> get templates;/// The id of the template chosen; null for none.
 String? get current;/// Why reading or setting failed.
 ApplicationError? get error;
/// Create a copy of DailyTemplateState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DailyTemplateStateCopyWith<DailyTemplateState> get copyWith => _$DailyTemplateStateCopyWithImpl<DailyTemplateState>(this as DailyTemplateState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DailyTemplateState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other.templates, templates)&&(identical(other.current, current) || other.current == current)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(templates),current,error);

@override
String toString() {
  return 'DailyTemplateState(status: $status, templates: $templates, current: $current, error: $error)';
}


}

/// @nodoc
abstract mixin class $DailyTemplateStateCopyWith<$Res>  {
  factory $DailyTemplateStateCopyWith(DailyTemplateState value, $Res Function(DailyTemplateState) _then) = _$DailyTemplateStateCopyWithImpl;
@useResult
$Res call({
 LoadStatus status, List<TemplateRow> templates, String? current, ApplicationError? error
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$DailyTemplateStateCopyWithImpl<$Res>
    implements $DailyTemplateStateCopyWith<$Res> {
  _$DailyTemplateStateCopyWithImpl(this._self, this._then);

  final DailyTemplateState _self;
  final $Res Function(DailyTemplateState) _then;

/// Create a copy of DailyTemplateState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? templates = null,Object? current = freezed,Object? error = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,templates: null == templates ? _self.templates : templates // ignore: cast_nullable_to_non_nullable
as List<TemplateRow>,current: freezed == current ? _self.current : current // ignore: cast_nullable_to_non_nullable
as String?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of DailyTemplateState
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


/// Adds pattern-matching-related methods to [DailyTemplateState].
extension DailyTemplateStatePatterns on DailyTemplateState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _DailyTemplateState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _DailyTemplateState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _DailyTemplateState value)  $default,){
final _that = this;
switch (_that) {
case _DailyTemplateState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _DailyTemplateState value)?  $default,){
final _that = this;
switch (_that) {
case _DailyTemplateState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LoadStatus status,  List<TemplateRow> templates,  String? current,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _DailyTemplateState() when $default != null:
return $default(_that.status,_that.templates,_that.current,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LoadStatus status,  List<TemplateRow> templates,  String? current,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _DailyTemplateState():
return $default(_that.status,_that.templates,_that.current,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LoadStatus status,  List<TemplateRow> templates,  String? current,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _DailyTemplateState() when $default != null:
return $default(_that.status,_that.templates,_that.current,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _DailyTemplateState extends DailyTemplateState {
  const _DailyTemplateState({this.status = LoadStatus.loading, final  List<TemplateRow> templates = const <TemplateRow>[], this.current, this.error}): _templates = templates,super._();
  

@override@JsonKey() final  LoadStatus status;
 final  List<TemplateRow> _templates;
@override@JsonKey() List<TemplateRow> get templates {
  if (_templates is EqualUnmodifiableListView) return _templates;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_templates);
}

/// The id of the template chosen; null for none.
@override final  String? current;
/// Why reading or setting failed.
@override final  ApplicationError? error;

/// Create a copy of DailyTemplateState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DailyTemplateStateCopyWith<_DailyTemplateState> get copyWith => __$DailyTemplateStateCopyWithImpl<_DailyTemplateState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DailyTemplateState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other._templates, _templates)&&(identical(other.current, current) || other.current == current)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(_templates),current,error);

@override
String toString() {
  return 'DailyTemplateState(status: $status, templates: $templates, current: $current, error: $error)';
}


}

/// @nodoc
abstract mixin class _$DailyTemplateStateCopyWith<$Res> implements $DailyTemplateStateCopyWith<$Res> {
  factory _$DailyTemplateStateCopyWith(_DailyTemplateState value, $Res Function(_DailyTemplateState) _then) = __$DailyTemplateStateCopyWithImpl;
@override @useResult
$Res call({
 LoadStatus status, List<TemplateRow> templates, String? current, ApplicationError? error
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$DailyTemplateStateCopyWithImpl<$Res>
    implements _$DailyTemplateStateCopyWith<$Res> {
  __$DailyTemplateStateCopyWithImpl(this._self, this._then);

  final _DailyTemplateState _self;
  final $Res Function(_DailyTemplateState) _then;

/// Create a copy of DailyTemplateState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? templates = null,Object? current = freezed,Object? error = freezed,}) {
  return _then(_DailyTemplateState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,templates: null == templates ? _self._templates : templates // ignore: cast_nullable_to_non_nullable
as List<TemplateRow>,current: freezed == current ? _self.current : current // ignore: cast_nullable_to_non_nullable
as String?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of DailyTemplateState
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
