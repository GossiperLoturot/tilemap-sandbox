extends Node3D
class_name Player


@export var world: World
@export var camera: Camera3D
@export var speed: float
@export var forward_size: float

var _location: Vector2
var _scroll: float
var _interpolated_scroll: float


func _process(delta):
	var input = Input.get_vector("left", "right", "down", "up")
	_location = _location + input * speed * delta

	if Input.is_action_just_released("scroll_up"):
		_scroll = clamp(_scroll - 0.25, log(16.0), log(512.0))
	if Input.is_action_just_released("scroll_down"):
		_scroll = clamp(_scroll + 0.25, log(16.0), log(512.0))
	_interpolated_scroll = lerp(_interpolated_scroll, _scroll, delta * 10.0)
	camera.size = exp(_interpolated_scroll)

	var viewport_size = get_viewport().size;
	if viewport_size.x > viewport_size.y:
		camera.keep_aspect = Camera3D.KEEP_WIDTH
	else:
		camera.keep_aspect = Camera3D.KEEP_HEIGHT
	camera.transform.origin.x = _location.x
	camera.transform.origin.y = _location.y

	world.min_forward_rect = Rect2(
		_location.x - forward_size,
		_location.y - forward_size,
		forward_size * 2,
		forward_size * 2
	)
	world.min_generate_rect = Rect2(
		_location.x - camera.size * 0.5,
		_location.y - camera.size * 0.5,
		camera.size,
		camera.size
	)
	world.min_view_rect = Rect2(
		_location.x - camera.size * 0.5,
		_location.y - camera.size * 0.5,
		camera.size,
		camera.size
	)
