extends Node3D


@export var camera: Camera3D
@export var gen_size_over: float
@export var view_size_over: float

var _scroll: float
var _interpolated_scroll: float

signal gen_rect_changed(rect: Rect2)
signal view_rect_changed(rect: Rect2)


func _process(delta) -> void:
	var location = Vector2()

	if Input.is_action_just_released("scroll_up"):
		_scroll = clamp(_scroll - 0.25, log(16.0), log(512.0))
	if Input.is_action_just_released("scroll_down"):
		_scroll = clamp(_scroll + 0.25, log(16.0), log(512.0))
	_interpolated_scroll = lerp(_interpolated_scroll, _scroll, delta * 10.0)
	camera.size = exp(_interpolated_scroll)

	var viewport_size = self.get_viewport().size;
	if viewport_size.x > viewport_size.y:
		camera.keep_aspect = Camera3D.KEEP_WIDTH
	else:
		camera.keep_aspect = Camera3D.KEEP_HEIGHT
	camera.transform.origin.x = location.x
	camera.transform.origin.y = location.y

	var gen_size = camera.size * 0.5 + gen_size_over
	gen_rect_changed.emit(Rect2(
		location.x - gen_size,
		location.y - gen_size,
		gen_size * 2,
		gen_size * 2
	))

	var view_size = camera.size * 0.5 + view_size_over
	view_rect_changed.emit(Rect2(
		location.x - view_size,
		location.y - view_size,
		view_size * 2,
		view_size * 2
	))
