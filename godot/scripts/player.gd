extends Node3D
class_name Player


@export var world: World
@export var camera: Camera3D
@export var speed: float
@export var view_size: float

var _entity_key: int
var _location: Vector2


func _ready():
	_entity_key = Actions.place_entity(world._root, Entity.new_from(0, Vector2(), 0))


func _process(delta):
	var input = Input.get_vector("left", "right", "down", "up")
	_location = _location + input * speed * delta

	camera.transform.origin.x = _location.x
	camera.transform.origin.y = _location.y

	world.min_view_rect = Rect2(
		_location.x - view_size,
		_location.y - view_size,
		view_size * 2,
		view_size * 2
	)

	Actions.move_entity(world._root, _entity_key, _location)
