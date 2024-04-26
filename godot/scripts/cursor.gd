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
		var entity = field._entity_field.intersection_with_point(location)
		if entity:
			var spec = field.entity_field_desc.entries[entity.get_id()]
			
			var marker_loc = entity.get_location() + spec.render_offset
			var marker_scl = spec.render_size
			
			transform.origin = Vector3(marker_loc.x, marker_loc.y, spec.render_size.y)
			transform.basis = Basis.from_scale(Vector3(marker_scl.x, marker_scl.y, 1.0))
		
	elif _mode == MODE_BLOCK:
		var locationi = Vector2i(floori(point.x), floori(point.y))
		var block = field._block_field.get(locationi)
		if block:
			var spec = field.block_field_desc.entries[block.get_id()]
			
			var marker_loc = Vector2(block.get_location()) + spec.render_offset
			var marker_scl = spec.render_size
			
			transform.origin = Vector3(marker_loc.x, marker_loc.y, spec.render_size.y)
			transform.basis = Basis.from_scale(Vector3(marker_scl.x, marker_scl.y, 1.0))
		
	elif _mode == MODE_TILE:
		var locationi = Vector2i(floori(point.x), floori(point.y))
		var tile = field._tile_field.get(locationi)
		if tile:
			var spec = field.tile_field_desc.entries[tile.get_id()]
			
			var marker_loc = Vector2(tile.get_location())
			
			transform.origin = Vector3(marker_loc.x, marker_loc.y, 0.0)
			transform.basis = Basis.from_scale(Vector3.ONE)


func _input(event):
	if event is InputEventMouseMotion:
		_mouse_position = event.position
