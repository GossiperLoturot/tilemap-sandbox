extends Node


@export var camera: Camera3D

var _picker: Node


func _process(_delta: float) -> void:
	var mouse_position = get_viewport().get_mouse_position()
	var project_position = camera.project_ray_origin(mouse_position)
	var point = Vector2(project_position.x, project_position.y)

	if Input.is_action_just_released("inventory"):
		Root.item_open_inventory_by_entity(point)

	if Input.is_action_just_pressed("secondary"):
		if _picker:
			_picker.queue_free()
		_picker = preload("res://scenes/picker.tscn").instantiate()
		_picker.call("set_point", point)
		self.add_child(_picker)
