extends Control


@export var camera: Camera3D
@export var picker: Node


func _process(_delta: float) -> void:
	if Input.is_action_just_pressed("inventory"):
		Root.open_inventory_player()


# invoked dynamically
func on_pick_item_changed(inventory_key: int, local_key: int, mouse_position: Vector2) -> void:
	picker.call("on_pick_item_changed", inventory_key, local_key, mouse_position)


func _on_gui_input(event: InputEvent) -> void:
	if event is InputEventMouseMotion:
		var mouse_position = self.get_global_mouse_position()
		var is_inside = self.get_viewport_rect().has_point(mouse_position)
		if is_inside:
			var projection = camera.project_ray_origin(mouse_position)
			var world_position = Vector2(projection.x, projection.y)
			picker.call("on_pick_changed", world_position, mouse_position)


func _on_mouse_entered() -> void:
	picker.call("on_pick_entered")


func _on_mouse_exited() -> void:
	picker.call("on_pick_existed")
