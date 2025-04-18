extends Control


const ITEM_SIZE: int = 32;

@export var item_scene: PackedScene
@export var item_deploy: Node
var _item_nodes: Array[Control]


func _enter_tree() -> void:
	for i in ITEM_SIZE:
		var item_instance = item_scene.instantiate()
		item_deploy.add_child(item_instance)

		_item_nodes.append(item_instance)


# invoked dynamicaly
func on_inventory_changed(inventory_key: int) -> void:
	for i in ITEM_SIZE:
		_item_nodes[i].call("on_inventory_item_changed", inventory_key, i)


func _on_header_gui_input(event: InputEvent) -> void:
	if event is InputEventMouseMotion:
		var is_inside = self.get_viewport_rect().has_point(self.get_global_mouse_position())
		if is_inside and event.button_mask == MOUSE_BUTTON_MASK_LEFT:
			self.position = self.position + event.relative


func _on_close_button_pressed() -> void:
	self.queue_free()
