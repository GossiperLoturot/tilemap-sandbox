extends Node3D
class_name Player


@export var world: World
@export var camera: Camera3D
@export var view_size: int
@export var speed: float

var _entity_key: int
var _location: Vector2
var _view_chunk_size: int
var _view_chunk_keys: Dictionary


func _ready():
	_view_chunk_size = 32
	assert(_view_chunk_size == world._tile_field.CHUNK_SIZE)
	assert(_view_chunk_size == world._block_field.CHUNK_SIZE)
	assert(_view_chunk_size == world._entity_field.CHUNK_SIZE)

	_entity_key = Action.place_entity(world._world, Entity.new_from(0, Vector2(), 0))	


func _process(delta):
	var input = Input.get_vector("left", "right", "down", "up")
	_location = _location + input * speed * delta

	Action.call_move_entity(world._world, _entity_key, _location)

	# view
	var origin_chunk_key = Vector2i(
		floor(_location.x / _view_chunk_size),
		floor(_location.y / _view_chunk_size)
	)

	var queue: Array[Vector2i]
	for chunk_key in _view_chunk_keys:
		var local_chunk_key = chunk_key - origin_chunk_key
		var out_of_range_x = local_chunk_key.x < -view_size or view_size + 1 <= local_chunk_key.x
		var out_of_range_y = local_chunk_key.y < -view_size or view_size + 1 <= local_chunk_key.y
		if  out_of_range_x or out_of_range_y:
			queue.append(chunk_key)

	for chunk_key in queue:
		world._tile_field.remove_view(chunk_key)
		world._block_field.remove_view(chunk_key)
		world._entity_field.remove_view(chunk_key)
		_view_chunk_keys.erase(chunk_key)

	for y in range(-view_size, view_size + 1):
		for x in range(-view_size, view_size + 1):
			var chunk_key = origin_chunk_key + Vector2i(x, y)
			if not _view_chunk_keys.has(chunk_key):
				world._tile_field.insert_view(chunk_key)
				world._block_field.insert_view(chunk_key)
				world._entity_field.insert_view(chunk_key)
				_view_chunk_keys[chunk_key] = null

	# generator
	var chunk_key = Vector2i(
		floor(_location.x / 32.0),
		floor(_location.y / 32.0)
	)
	Action.generate_chunk(world._world, chunk_key)

	camera.transform.origin.x = _location.x
	camera.transform.origin.y = _location.y
