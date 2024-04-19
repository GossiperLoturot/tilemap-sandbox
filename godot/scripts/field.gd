extends Node3D


@export var camera: Camera3D

@export var tile_field_desc: TileFieldDesc
@export var block_field_desc: BlockFieldDesc
@export var entity_field_desc: EntityFieldDesc

var _tile_field: TileField
var _block_field: BlockField
var _entity_field: EntityField


func _ready():
	_tile_field = TileField.new_from(tile_field_desc, get_world_3d())
	_block_field = BlockField.new_from(block_field_desc, get_world_3d())
	_entity_field = EntityField.new_from(entity_field_desc, get_world_3d())
	
	for y in range(-64, 65):
		for x in range(-64, 65):
			_tile_field.insert(Tile.new_from(randi_range(0, 1), Vector2i(x, y)))
	
	for i in range(4096):
		var x = randi_range(-64, 65)
		var y = randf_range(-64, 65)
		_block_field.insert(Block.new_from(randi_range(0, 9), Vector2i(x, y)))
	
	_entity_field.insert(Entity.new_from(0, Vector2(1.2, 3.4)))
	
	for y in range(-4, 5):
		for x in range(-4, 5):
			_tile_field.insert_view(Vector2i(x, y))
			_block_field.insert_view(Vector2i(x, y))
			_entity_field.insert_view(Vector2i(x, y))


func _process(_delta):
	_tile_field.update_view()
	_block_field.update_view()
	_entity_field.update_view()


func _input(event):
	if event is InputEventMouseButton and event.pressed and event.button_index == 1:
		var origin = camera.project_ray_origin(event.position)
		var dir = camera.project_ray_normal(event.position)
		
		var k = origin.z / dir.z
		var point = origin - dir * k
		var point2d = Vector2(point.x, point.y)
		
		print(_block_field.intersection_with_point(point2d))
