extends Node3D
class_name Cursor


const MODE_ENTITY: int = 0
const MODE_BLOCK: int = 1
const MODE_TILE: int = 2

@export var world: World
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
		var keys = world._root.entity_field.get_hint_by_point(location)
		if len(keys) > 0:
			var rect = world._root.entity_field.get_hint_rect(keys[0])
			transform.origin = Vector3(rect.position.x, rect.position.y, rect.size.y)
			transform.basis = Basis.from_scale(Vector3(rect.size.x, rect.size.y, 1.0))

			if Input.is_action_just_pressed("primary"):
				Actions.break_entity(world._root, keys[0])

	elif _mode == MODE_BLOCK:
		var location = Vector2(point.x, point.y)
		var keys = world._root.block_field.get_hint_by_point(location)
		if len(keys) > 0:
			var rect = world._root.block_field.get_hint_rect(keys[0])
			transform.origin = Vector3(rect.position.x, rect.position.y, rect.size.y)
			transform.basis = Basis.from_scale(Vector3(rect.size.x, rect.size.y, 1.0))

			if Input.is_action_just_pressed("primary"):
				Actions.break_block(world._root, keys[0])

	elif _mode == MODE_TILE:
		var location = Vector2i(floori(point.x), floori(point.y))
		if world._root.tile_field.has_by_point(location):
			var key = world._root.tile_field.get_by_point(location)

			transform.origin = Vector3(location.x, location.y, 0.0)
			transform.basis = Basis.from_scale(Vector3.ONE)

			if Input.is_action_just_pressed("primary"):
				Actions.break_tile(world._root, key)


func _input(event):
	if event is InputEventMouseMotion:
		_mouse_position = event.position
