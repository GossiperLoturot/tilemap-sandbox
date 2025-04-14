extends Control


@export var placeholder_node: Control
var _ui: Control
var _inventory_key: int
var _local_key: int


func _process(_delta: float) -> void:
	Root.draw_item(_inventory_key, _local_key, placeholder_node)


# invoked dynamicaly
func on_inventory_item_changed(ui: Control, inventory_key: int, local_key: int) -> void:
	_ui = ui
	_inventory_key = inventory_key
	_local_key = local_key


func _on_gui_input(event: InputEvent) -> void:
	if event is InputEventMouseMotion:
		var mouse_position = self.get_global_mouse_position()
		_ui.call("on_pick_item_changed", _inventory_key, _local_key, mouse_position)


func _on_mouse_entered() -> void:
	_ui.call("_on_mouse_entered")

	var tween = self.create_tween()
	tween.tween_property(self, "modulate", Color(1.5, 1.5, 1.5), 0.1)
	tween.tween_callback(tween.kill)


func _on_mouse_exited() -> void:
	_ui.call("_on_mouse_exited")

	var tween = self.create_tween()
	tween.tween_property(self, "modulate", Color(1.0, 1.0, 1.0), 0.1)
	tween.tween_callback(tween.kill)
