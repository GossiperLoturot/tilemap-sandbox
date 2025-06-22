extends Control


@export var camera: Camera3D
var _is_mouse_entered: bool

var offset: int


func _process(_delta: float) -> void:
	if _is_mouse_entered:
		var mouse_position = self.get_global_mouse_position()
		var projection = camera.project_ray_origin(mouse_position)
		var world_position = Vector2(projection.x, projection.y)

		if SelectionModePicker.context.get_mode() == SelectionModePicker.context.MODE_NONE:
			Selection.context.hide_selection()

		if SelectionModePicker.context.get_mode() == SelectionModePicker.context.MODE_TILE:
			var tile_key = Context.find_tile(world_position)
			Selection.context.select_tile(tile_key)
			if Input.is_action_just_pressed("primary"):
				Selection.context.confirm_tile(tile_key)

		if SelectionModePicker.context.get_mode() == SelectionModePicker.context.MODE_BLOCK:
			var block_key = Context.find_block(world_position, offset)
			Selection.context.select_block(block_key)
			if Input.is_action_just_pressed("primary"):
				Selection.context.confirm_block(block_key)

		if SelectionModePicker.context.get_mode() == SelectionModePicker.context.MODE_ENTITY:
			var entity_key = Context.find_entity(world_position, offset)
			Selection.context.select_entity(entity_key)
			if Input.is_action_just_pressed("primary"):
				Selection.context.confirm_entity(entity_key)


func _on_mouse_entered() -> void:
	_is_mouse_entered = true
	Selection.context.show_selection()


func _on_mouse_exited() -> void:
	_is_mouse_entered = false
	Selection.context.hide_selection()
