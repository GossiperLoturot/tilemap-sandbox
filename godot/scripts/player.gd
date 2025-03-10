extends Node3D
class_name Player


@export var world: World
@export var camera: Camera3D
@export var forward_size: float
@export var view_size_over: float
@export var label: Label

var _scroll: float
var _interpolated_scroll: float


func _ready() -> void:
	world._root.player_spawn(Vector2())


func _process(delta) -> void:
	var input = Input.get_vector("left", "right", "down", "up")
	world._root.player_insert_input(input)

	var location = world._root.player_get_current_location()

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
	camera.transform.origin.x = location.x
	camera.transform.origin.y = location.y

	var view_size = camera.size * 0.5 + view_size_over

	world.min_forwarder_rect = Rect2(
		location.x - forward_size,
		location.y - forward_size,
		forward_size * 2,
		forward_size * 2
	)
	world.min_gen_rect = Rect2(
		location.x - view_size,
		location.y - view_size,
		view_size * 2,
		view_size * 2
	)
	world.min_view_rect = Rect2(
		location.x - view_size,
		location.y - view_size,
		view_size * 2,
		view_size * 2
	)

	var mouse_position = get_viewport().get_mouse_position()
	var cursor = camera.project_ray_origin(mouse_position)
	var name_text = world._root.tile_get_name_text(Vector2i(cursor.x, cursor.y))
	label.text = name_text
