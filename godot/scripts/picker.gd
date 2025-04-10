extends Control


var _deployment: Node
var _point: Vector2


func _enter_tree() -> void:
	_deployment = $Container/Body/Container


func _process(_delta: float) -> void:
	for old in _deployment.get_children():
		old.queue_free()
	for i in Root.get_pick_size(_point):
		var picker_item = preload("res://scenes/picker_item.tscn").instantiate()
		_deployment.add_child(picker_item)

		var label = picker_item.get_node("Container/Placeholder")
		label.text = Root.get_pick_name_text(_point, i)


# invoked by the instantiate function dynamically from other script
func set_point(point: Vector2) -> void:
	_point = point


# invoked by the signal
func drag_inventory(event: InputEvent) -> void:
	if event is InputEventMouseMotion:
		var is_inside = self.get_viewport_rect().has_point(self.get_global_mouse_position())
		if is_inside and event.button_mask == MOUSE_BUTTON_MASK_LEFT:
			self.position = self.position + event.relative


# invoked by the signal
func close_inventory() -> void:
	self.queue_free()
