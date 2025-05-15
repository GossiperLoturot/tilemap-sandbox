extends Control


@export var item_deploy: Node


func on_instantiate(slot_keys: Array[SlotKey]) -> void:
	for slot_key in slot_keys:
		var instance = preload("res://scenes/inventory_item.tscn").instantiate()
		item_deploy.add_child(instance)
		instance.on_instantiate(slot_key)


func _on_gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT:
		self.get_parent().move_child(self, -1)


func _on_header_gui_input(event: InputEvent) -> void:
	if event is InputEventMouseMotion and event.button_mask == MOUSE_BUTTON_MASK_LEFT:
		if self.get_viewport_rect().has_point(self.get_global_mouse_position()):
			self.position = self.position + event.relative


func _on_close_button_pressed() -> void:
	self.queue_free()
