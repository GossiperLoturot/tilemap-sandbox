extends Node3D


@export var tile_field_desc: TileFieldDesc
var _tile_field: TileField


func _ready():
	_tile_field = TileField.new_from(tile_field_desc, get_world_3d())
	
	for y in range(32):
		for x in range(32):
			_tile_field.add_tile(Tile.new_from(12, x, y))


func _process(delta):
	_tile_field.spawn()
