extends Control


@export var placeholder_node: Control
var _ui: Control
var _inventory_key: int
var _local_key: int
var _brightness1: float
var _brightness2: float


func _process(_delta: float) -> void:
	Root.draw_item(_inventory_key, _local_key, placeholder_node)


# invoked dynamicaly
func change_inventory_item(ui: Control, inventory_key: int, local_key: int) -> void:
	_ui = ui
	_inventory_key = inventory_key
	_local_key = local_key

	ui.connect("item_picked", func(inventory_key: int, local_key: int):
		if inventory_key == _inventory_key and local_key == _local_key:
			_brightness2 = 1.5
			change_brightness()
		pass)
	ui.connect("item_unpicked", func(inventory_key: int, local_key: int):
		if inventory_key == _inventory_key and local_key == _local_key:
			_brightness2 = 1.0
			change_brightness()
		pass)


func change_brightness():
	var v = max(_brightness1, _brightness2)
	self.modulate.r = v
	self.modulate.g = v
	self.modulate.b = v


func _on_gui_input(event: InputEvent) -> void:
	if event is InputEventMouseMotion:
		var mouse_position = self.get_global_mouse_position()
		_ui.call("change_pick_item", _inventory_key, _local_key, mouse_position)
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT and event.pressed:
		_ui.call("confirm_pick_item", _inventory_key, _local_key)


func _on_mouse_entered() -> void:
	_ui.call("show_pick")

	var tween = self.create_tween()
	tween.tween_method(func(v: float):
		_brightness1 = v
		change_brightness()
		pass, 1.0, 1.5, 0.1
	)


func _on_mouse_exited() -> void:
	_ui.call("hide_pick")

	var tween = self.create_tween()
	tween.tween_method(func(v: float):
		_brightness1 = v
		change_brightness()
		pass, 1.5, 1.0, 0.1
	)
