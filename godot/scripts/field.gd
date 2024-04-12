extends Node3D


@export var tile_field_desc: TileFieldDesc
var _tile_field: TileField


func _ready():
	_tile_field = TileField.new_from(tile_field_desc, get_world_3d())
	
	for y in range(64):
		for x in range(64):
			_tile_field.add_tile(Tile.new_from(randi_range(12, 15), Vector2i(x, y)))
	
	for y in range(4):
		for x in range(4):
			_tile_field.add_view(Vector2i(x, y))


func _process(_delta):
	_tile_field.update_view()
