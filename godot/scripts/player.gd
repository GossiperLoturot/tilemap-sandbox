extends Node3D


signal change_min_forwarder_rect(rect: Rect2)
signal change_min_gen_rect(rect: Rect2)
signal change_min_view_rect(rect: Rect2)

@export var camera: Camera3D
@export var forward_size: float
@export var view_size_over: float

var _scroll: float
var _interpolated_scroll: float


# spawn player
func _enter_tree() -> void:
	Root.player_spawn(Vector2())


func _process(delta) -> void:
	var move = Input.get_vector("left", "right", "down", "up")
	Root.player_push_input(move)

	var location = Root.player_get_location()

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

	change_min_forwarder_rect.emit(Rect2(
		location.x - forward_size,
		location.y - forward_size,
		forward_size * 2,
		forward_size * 2
	))
	change_min_gen_rect.emit(Rect2(
		location.x - view_size,
		location.y - view_size,
		view_size * 2,
		view_size * 2
	))
	change_min_view_rect.emit(Rect2(
		location.x - view_size,
		location.y - view_size,
		view_size * 2,
		view_size * 2
	))


# kill player
func _exit_tree() -> void:
	pass #Root.player_kill()
