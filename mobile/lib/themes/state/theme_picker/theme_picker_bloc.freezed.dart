// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'theme_picker_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ThemePickerEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ThemePickerEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ThemePickerEvent()';
}


}

/// @nodoc
class $ThemePickerEventCopyWith<$Res>  {
$ThemePickerEventCopyWith(ThemePickerEvent _, $Res Function(ThemePickerEvent) __);
}


/// Adds pattern-matching-related methods to [ThemePickerEvent].
extension ThemePickerEventPatterns on ThemePickerEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ThemePickerStartedEvent value)?  started,TResult Function( ThemePickerInstalledChangedEvent value)?  installedChanged,TResult Function( ThemePickerImportRequestedEvent value)?  importRequested,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ThemePickerStartedEvent() when started != null:
return started(_that);case ThemePickerInstalledChangedEvent() when installedChanged != null:
return installedChanged(_that);case ThemePickerImportRequestedEvent() when importRequested != null:
return importRequested(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ThemePickerStartedEvent value)  started,required TResult Function( ThemePickerInstalledChangedEvent value)  installedChanged,required TResult Function( ThemePickerImportRequestedEvent value)  importRequested,}){
final _that = this;
switch (_that) {
case ThemePickerStartedEvent():
return started(_that);case ThemePickerInstalledChangedEvent():
return installedChanged(_that);case ThemePickerImportRequestedEvent():
return importRequested(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ThemePickerStartedEvent value)?  started,TResult? Function( ThemePickerInstalledChangedEvent value)?  installedChanged,TResult? Function( ThemePickerImportRequestedEvent value)?  importRequested,}){
final _that = this;
switch (_that) {
case ThemePickerStartedEvent() when started != null:
return started(_that);case ThemePickerInstalledChangedEvent() when installedChanged != null:
return installedChanged(_that);case ThemePickerImportRequestedEvent() when importRequested != null:
return importRequested(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function()?  installedChanged,TResult Function()?  importRequested,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ThemePickerStartedEvent() when started != null:
return started();case ThemePickerInstalledChangedEvent() when installedChanged != null:
return installedChanged();case ThemePickerImportRequestedEvent() when importRequested != null:
return importRequested();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function()  installedChanged,required TResult Function()  importRequested,}) {final _that = this;
switch (_that) {
case ThemePickerStartedEvent():
return started();case ThemePickerInstalledChangedEvent():
return installedChanged();case ThemePickerImportRequestedEvent():
return importRequested();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function()?  installedChanged,TResult? Function()?  importRequested,}) {final _that = this;
switch (_that) {
case ThemePickerStartedEvent() when started != null:
return started();case ThemePickerInstalledChangedEvent() when installedChanged != null:
return installedChanged();case ThemePickerImportRequestedEvent() when importRequested != null:
return importRequested();case _:
  return null;

}
}

}

/// @nodoc


class ThemePickerStartedEvent implements ThemePickerEvent {
  const ThemePickerStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ThemePickerStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ThemePickerEvent.started()';
}


}




/// @nodoc


class ThemePickerInstalledChangedEvent implements ThemePickerEvent {
  const ThemePickerInstalledChangedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ThemePickerInstalledChangedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ThemePickerEvent.installedChanged()';
}


}




/// @nodoc


class ThemePickerImportRequestedEvent implements ThemePickerEvent {
  const ThemePickerImportRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ThemePickerImportRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ThemePickerEvent.importRequested()';
}


}




/// @nodoc
mixin _$ThemePickerState {

 LoadStatus get status; List<ThemePreview> get themes;/// The theme files that were left out, and why.
 List<ThemeFileNotice> get notices;/// Whether a file is being imported.
 bool get importing;/// How the last import ended; null before one, and while another runs.
 ThemeImportOutcome? get outcome;
/// Create a copy of ThemePickerState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ThemePickerStateCopyWith<ThemePickerState> get copyWith => _$ThemePickerStateCopyWithImpl<ThemePickerState>(this as ThemePickerState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ThemePickerState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other.themes, themes)&&const DeepCollectionEquality().equals(other.notices, notices)&&(identical(other.importing, importing) || other.importing == importing)&&(identical(other.outcome, outcome) || other.outcome == outcome));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(themes),const DeepCollectionEquality().hash(notices),importing,outcome);

@override
String toString() {
  return 'ThemePickerState(status: $status, themes: $themes, notices: $notices, importing: $importing, outcome: $outcome)';
}


}

/// @nodoc
abstract mixin class $ThemePickerStateCopyWith<$Res>  {
  factory $ThemePickerStateCopyWith(ThemePickerState value, $Res Function(ThemePickerState) _then) = _$ThemePickerStateCopyWithImpl;
@useResult
$Res call({
 LoadStatus status, List<ThemePreview> themes, List<ThemeFileNotice> notices, bool importing, ThemeImportOutcome? outcome
});


$ThemeImportOutcomeCopyWith<$Res>? get outcome;

}
/// @nodoc
class _$ThemePickerStateCopyWithImpl<$Res>
    implements $ThemePickerStateCopyWith<$Res> {
  _$ThemePickerStateCopyWithImpl(this._self, this._then);

  final ThemePickerState _self;
  final $Res Function(ThemePickerState) _then;

/// Create a copy of ThemePickerState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? themes = null,Object? notices = null,Object? importing = null,Object? outcome = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,themes: null == themes ? _self.themes : themes // ignore: cast_nullable_to_non_nullable
as List<ThemePreview>,notices: null == notices ? _self.notices : notices // ignore: cast_nullable_to_non_nullable
as List<ThemeFileNotice>,importing: null == importing ? _self.importing : importing // ignore: cast_nullable_to_non_nullable
as bool,outcome: freezed == outcome ? _self.outcome : outcome // ignore: cast_nullable_to_non_nullable
as ThemeImportOutcome?,
  ));
}
/// Create a copy of ThemePickerState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ThemeImportOutcomeCopyWith<$Res>? get outcome {
    if (_self.outcome == null) {
    return null;
  }

  return $ThemeImportOutcomeCopyWith<$Res>(_self.outcome!, (value) {
    return _then(_self.copyWith(outcome: value));
  });
}
}


/// Adds pattern-matching-related methods to [ThemePickerState].
extension ThemePickerStatePatterns on ThemePickerState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ThemePickerState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ThemePickerState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ThemePickerState value)  $default,){
final _that = this;
switch (_that) {
case _ThemePickerState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ThemePickerState value)?  $default,){
final _that = this;
switch (_that) {
case _ThemePickerState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LoadStatus status,  List<ThemePreview> themes,  List<ThemeFileNotice> notices,  bool importing,  ThemeImportOutcome? outcome)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ThemePickerState() when $default != null:
return $default(_that.status,_that.themes,_that.notices,_that.importing,_that.outcome);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LoadStatus status,  List<ThemePreview> themes,  List<ThemeFileNotice> notices,  bool importing,  ThemeImportOutcome? outcome)  $default,) {final _that = this;
switch (_that) {
case _ThemePickerState():
return $default(_that.status,_that.themes,_that.notices,_that.importing,_that.outcome);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LoadStatus status,  List<ThemePreview> themes,  List<ThemeFileNotice> notices,  bool importing,  ThemeImportOutcome? outcome)?  $default,) {final _that = this;
switch (_that) {
case _ThemePickerState() when $default != null:
return $default(_that.status,_that.themes,_that.notices,_that.importing,_that.outcome);case _:
  return null;

}
}

}

/// @nodoc


class _ThemePickerState implements ThemePickerState {
  const _ThemePickerState({this.status = LoadStatus.loading, final  List<ThemePreview> themes = const <ThemePreview>[], final  List<ThemeFileNotice> notices = const <ThemeFileNotice>[], this.importing = false, this.outcome}): _themes = themes,_notices = notices;
  

@override@JsonKey() final  LoadStatus status;
 final  List<ThemePreview> _themes;
@override@JsonKey() List<ThemePreview> get themes {
  if (_themes is EqualUnmodifiableListView) return _themes;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_themes);
}

/// The theme files that were left out, and why.
 final  List<ThemeFileNotice> _notices;
/// The theme files that were left out, and why.
@override@JsonKey() List<ThemeFileNotice> get notices {
  if (_notices is EqualUnmodifiableListView) return _notices;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_notices);
}

/// Whether a file is being imported.
@override@JsonKey() final  bool importing;
/// How the last import ended; null before one, and while another runs.
@override final  ThemeImportOutcome? outcome;

/// Create a copy of ThemePickerState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ThemePickerStateCopyWith<_ThemePickerState> get copyWith => __$ThemePickerStateCopyWithImpl<_ThemePickerState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ThemePickerState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other._themes, _themes)&&const DeepCollectionEquality().equals(other._notices, _notices)&&(identical(other.importing, importing) || other.importing == importing)&&(identical(other.outcome, outcome) || other.outcome == outcome));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(_themes),const DeepCollectionEquality().hash(_notices),importing,outcome);

@override
String toString() {
  return 'ThemePickerState(status: $status, themes: $themes, notices: $notices, importing: $importing, outcome: $outcome)';
}


}

/// @nodoc
abstract mixin class _$ThemePickerStateCopyWith<$Res> implements $ThemePickerStateCopyWith<$Res> {
  factory _$ThemePickerStateCopyWith(_ThemePickerState value, $Res Function(_ThemePickerState) _then) = __$ThemePickerStateCopyWithImpl;
@override @useResult
$Res call({
 LoadStatus status, List<ThemePreview> themes, List<ThemeFileNotice> notices, bool importing, ThemeImportOutcome? outcome
});


@override $ThemeImportOutcomeCopyWith<$Res>? get outcome;

}
/// @nodoc
class __$ThemePickerStateCopyWithImpl<$Res>
    implements _$ThemePickerStateCopyWith<$Res> {
  __$ThemePickerStateCopyWithImpl(this._self, this._then);

  final _ThemePickerState _self;
  final $Res Function(_ThemePickerState) _then;

/// Create a copy of ThemePickerState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? themes = null,Object? notices = null,Object? importing = null,Object? outcome = freezed,}) {
  return _then(_ThemePickerState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,themes: null == themes ? _self._themes : themes // ignore: cast_nullable_to_non_nullable
as List<ThemePreview>,notices: null == notices ? _self._notices : notices // ignore: cast_nullable_to_non_nullable
as List<ThemeFileNotice>,importing: null == importing ? _self.importing : importing // ignore: cast_nullable_to_non_nullable
as bool,outcome: freezed == outcome ? _self.outcome : outcome // ignore: cast_nullable_to_non_nullable
as ThemeImportOutcome?,
  ));
}

/// Create a copy of ThemePickerState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ThemeImportOutcomeCopyWith<$Res>? get outcome {
    if (_self.outcome == null) {
    return null;
  }

  return $ThemeImportOutcomeCopyWith<$Res>(_self.outcome!, (value) {
    return _then(_self.copyWith(outcome: value));
  });
}
}

// dart format on
