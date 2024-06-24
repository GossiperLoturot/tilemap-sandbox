extends Node3D


const MODE_ENTITY: int = 0
const MODE_BLOCK: int = 1
const MODE_TILE: int = 2

@export var field: Field
@export var camera: Camera3D

var _mouse_position: Vector2
var _mode: int = MODE_ENTITY


func _process(delta):
	if Input.is_action_just_pressed("mode_prev"):
		_mode = clampi(_mode - 1, MODE_ENTITY, MODE_TILE)
		print("mode is %s" % _mode)
	if Input.is_action_just_pressed("mode_next"):
		_mode = clampi(_mode + 1, MODE_ENTITY, MODE_TILE)
		print("mode is %s" % _mode)

	var origin = camera.project_ray_origin(_mouse_position)
	var dir = camera.project_ray_normal(_mouse_position)

	var point = origin - dir * (origin.z / dir.z)

	transform.origin = Vector3.ZERO
	transform.basis = Basis.from_scale(Vector3.ZERO)

	if _mode == MODE_ENTITY:
		var location = Vector2(point.x, point.y)
		var keys = field._entity_field.get_hint_by_point(location)
		if len(keys) > 0:
			var entity = field._entity_field.get(keys[0])
			var spec = field._entity_field_desc.entries[entity.get_id()]

			var cursor_location = entity.get_location() + spec.rendering_offset
			var cursor_scale = spec.rendering_size

			transform.origin = Vector3(cursor_location.x, cursor_location.y, spec.rendering_size.y)
			transform.basis = Basis.from_scale(Vector3(cursor_scale.x, cursor_scale.y, 1.0))

			if Input.is_action_just_pressed("primary"):
				field._world.break_entity(keys[0])

	elif _mode == MODE_BLOCK:
		var location = Vector2(point.x, point.y)
		var keys = field._block_field.get_hint_by_point(location)
		if len(keys) > 0:
			var block = field._block_field.get(keys[0])
			var spec = field._block_field_desc.entries[block.get_id()]

			var cursor_location = Vector2(block.get_location()) + spec.rendering_offset
			var cursor_scale = spec.rendering_size

			transform.origin = Vector3(cursor_location.x, cursor_location.y, spec.rendering_size.y)
			transform.basis = Basis.from_scale(Vector3(cursor_scale.x, cursor_scale.y, 1.0))

			if Input.is_action_just_pressed("primary"):
				field._world.break_block(keys[0])

	elif _mode == MODE_TILE:
		var location = Vector2i(floori(point.x), floori(point.y))
		var key = field._tile_field.get_by_point(location)
		if key:
			var tile = field._tile_field.get(key)
			var spec = field._tile_field_desc.entries[tile.get_id()]

			var cursor_location = Vector2(tile.get_location())

			transform.origin = Vector3(cursor_location.x, cursor_location.y, 0.0)
			transform.basis = Basis.from_scale(Vector3.ONE)

			if Input.is_action_just_pressed("primary"):
				field._world.break_tile(key)


func _input(event):
	if event is InputEventMouseMotion:
		_mouse_position = event.position
