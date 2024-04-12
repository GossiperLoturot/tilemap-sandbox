extends Node3D


@export var tile_field_desc: TileFieldDesc
var _tile_field: TileField


func _ready():
	_tile_field = TileField.new_from(tile_field_desc, get_world_3d())
	
	for y in range(64):
		for x in range(64):
			_tile_field.add_tile(Tile.new_from(12, x, y))

	_tile_field.add_view(Vector2i(0, 0))
	_tile_field.add_view(Vector2i(1, 0))
	_tile_field.add_view(Vector2i(0, 1))
	_tile_field.add_view(Vector2i(1, 1))


func _process(_delta):
	_tile_field.update_view()
