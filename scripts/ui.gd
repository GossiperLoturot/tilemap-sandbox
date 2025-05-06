extends Control


const HOLD_NONE = 0
const HOLD_ITEM = 1

signal item_selected(inventory_key: int, local_key: int)
signal item_unselected(inventory_key: int, local_key: int)

@export var camera: Camera3D
@export var selection: Node
var _hold: int
var _inventory_key: int
var _local_key: int
var _is_mouse_entered: bool


func _process(_delta: float) -> void:
	if _is_mouse_entered:
		var mouse_position = self.get_global_mouse_position()
		var projection = camera.project_ray_origin(mouse_position)
		var world_position = Vector2(projection.x, projection.y)
		change_select_field(world_position, mouse_position)
	else:
		Context.clear_selection()

	if Input.is_action_just_pressed("inventory"):
		Context.open_player_inventory()


func change_select_field(world_position: Vector2, mouse_position: Vector2) -> void:
	Context.set_selection(world_position)

	var texts: Array[String]
	for i in Context.get_selection_size():
		var text = Context.get_selection_display_name()
		texts.append(text)
	selection.call("change_selection", texts, mouse_position)


func confirm_select_field(world_position: Vector2, mouse_position: Vector2) -> void:
	if _hold == HOLD_ITEM:
		print("USE ITEM TO PICKED FIELD")
		_hold = HOLD_NONE
	else:
		print("PICK FIELD")


# invoked dynamically
func change_select_item(inventory_key: int, local_key: int, mouse_position: Vector2) -> void:
	var texts: Array[String]
	if Context.has_item(inventory_key, local_key):
		var text = Context.get_item_display_name(inventory_key, local_key)
		texts.append(text)
	selection.call("change_selection", texts, mouse_position)


# invoked dynamically
func confirm_select_item(inventory_key: int, local_key: int) -> void:
	if _hold == HOLD_ITEM:
		Context.swap_item(_inventory_key, _local_key, inventory_key, local_key)
		item_unselected.emit(_inventory_key, _local_key)

		_hold = HOLD_NONE
	else:
		_hold = HOLD_ITEM
		_inventory_key = inventory_key
		_local_key = local_key

		item_selected.emit(inventory_key, local_key)


# invoked dynamically
func show_selection() -> void:
	selection.show()


# invoked dynamically
func hide_selection() -> void:
	selection.hide()


func _on_gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT and event.pressed:
		var mouse_position = self.get_global_mouse_position()
		var projection = camera.project_ray_origin(mouse_position)
		var world_position = Vector2(projection.x, projection.y)
		confirm_select_field(world_position, mouse_position)


func _on_mouse_entered() -> void:
	_is_mouse_entered = true

	show_selection()


func _on_mouse_exited() -> void:
	_is_mouse_entered = false

	hide_selection()
