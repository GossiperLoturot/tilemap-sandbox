extends Control


const HOLD_NONE = 0
const HOLD_ITEM = 1

signal item_picked(inventory_key: int, local_key: int)
signal item_unpicked(inventory_key: int, local_key: int)

@export var camera: Camera3D
@export var picker: Node
var _hold: int
var _inventory_key: int
var _local_key: int
var _is_mouse_entered: bool


func _process(_delta: float) -> void:
	if _is_mouse_entered:
		var mouse_position = self.get_global_mouse_position()
		var projection = camera.project_ray_origin(mouse_position)
		var world_position = Vector2(projection.x, projection.y)
		change_pick_field(world_position, mouse_position)
	else:
		Root.clear_pick()

	if Input.is_action_just_pressed("inventory"):
		Root.open_inventory_player()


func change_pick_field(world_position: Vector2, mouse_position: Vector2) -> void:
	var texts: Array[String]
	for i in Root.get_pick_size(world_position):
		var text = Root.get_pick_name_text(world_position, i)
		texts.append(text)
	picker.call("change_pick", texts, mouse_position)

	Root.set_pick(world_position)


func confirm_pick_field(world_position: Vector2, mouse_position: Vector2) -> void:
	if _hold == HOLD_ITEM:
		print("USE ITEM TO PICKED FIELD")
		_hold = HOLD_NONE
	else:
		print("PICK FIELD")


# invoked dynamically
func change_pick_item(inventory_key: int, local_key: int, mouse_position: Vector2) -> void:
	var texts: Array[String]
	if Root.has_item(inventory_key, local_key):
		var text = Root.get_item_name_text(inventory_key, local_key)
		texts.append(text)
	picker.call("change_pick", texts, mouse_position)


# invoked dynamically
func confirm_pick_item(inventory_key: int, local_key: int) -> void:
	if _hold == HOLD_ITEM:
		Root.swap_item(_inventory_key, _local_key, inventory_key, local_key)
		item_unpicked.emit(_inventory_key, _local_key)

		_hold = HOLD_NONE
	else:
		_hold = HOLD_ITEM
		_inventory_key = inventory_key
		_local_key = local_key

		item_picked.emit(inventory_key, local_key)


# invoked dynamically
func show_pick() -> void:
	picker.show()


# invoked dynamically
func hide_pick() -> void:
	picker.hide()


func _on_gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT and event.pressed:
		var mouse_position = self.get_global_mouse_position()
		var projection = camera.project_ray_origin(mouse_position)
		var world_position = Vector2(projection.x, projection.y)
		confirm_pick_field(world_position, mouse_position)


func _on_mouse_entered() -> void:
	_is_mouse_entered = true

	show_pick()


func _on_mouse_exited() -> void:
	_is_mouse_entered = false

	hide_pick()
