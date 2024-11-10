extends Node3D
class_name Player


@export var world: World
@export var camera: Camera3D
@export var forward_size: float
@export var view_size_over: float

var _scroll: float
var _interpolated_scroll: float


func _ready() -> void:
	var entity = Entity.create(world.ENTITY_PLAYER, Vector2())
	world._root.entity_insert(entity)


func _process(delta) -> void:
	var input = Input.get_vector("left", "right", "down", "up")
	world._root.player_input(input)

	var location = world._root.player_location()

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

	world.min_forward_rect = Rect2(
		location.x - forward_size,
		location.y - forward_size,
		forward_size * 2,
		forward_size * 2
	)
	world.min_generate_rect = Rect2(
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

	if Input.is_action_just_pressed("inventory"):
		open_inventory()


func open_inventory() -> void:
	var inv = world._root.player_inventory()

	var inv_size = world._root.inventory_size(inv)
	print("player inventory (size: %d)" % inv_size)
	for i in range(inv_size):
		var item = world._root.inventory_get(inv, i)
		print("├ item (id: %d, amount: %d)" % [item.id, item.amount])
