extends Node


@export var field: Field
@export var speed: float = 4.0

var _location: Vector2
var _key: int


func _ready():
	var entity = Entity.new_from(0, _location)
	_key = field._entity_field.insert(entity)


func _process(delta):
	var input_dir = Input.get_vector("left", "right", "down", "up")
	var move = input_dir * delta * speed
	
	var dst = _location + move
	var dst_x = _location + move * Vector2.RIGHT
	var dst_y = _location + move * Vector2.DOWN
	
	var col = field._block_field.get_collision_by_point(dst)
	var col_x = field._block_field.get_collision_by_point(dst_x)
	var col_y = field._block_field.get_collision_by_point(dst_y)
	
	var changed = false
	if not col:
		_location = dst
		changed = true
	elif not col_x:
		_location = dst_x
		changed = true
	elif not col_y:
		_location = dst_y
		changed = true
	
	if changed:
		field._entity_field.remove(_key)
		
		var entity = Entity.new_from(0, _location)
		_key = field._entity_field.insert(entity)
