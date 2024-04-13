extends Node3D


@export var tile_field_desc: TileFieldDesc
@export var block_field_desc: BlockFieldDesc
var _tile_field: TileField
var _block_field: BlockField


func _ready():
	_tile_field = TileField.new_from(tile_field_desc, get_world_3d())
	_block_field = BlockField.new_from(block_field_desc, get_world_3d())
	
	for y in range(64):
		for x in range(64):
			_tile_field.insert(Tile.new_from(randi_range(0, 1), Vector2i(x, y)))
	
	for i in range(1024):
		var x = randi_range(0, 64)
		var y = randf_range(0, 64)
		_block_field.insert(Block.new_from(randi_range(0, 9), Vector2i(x, y)))
	
	for y in range(4):
		for x in range(4):
			_tile_field.insert_view(Vector2i(x, y))
			_block_field.insert_view(Vector2i(x, y))


func _process(_delta):
	_tile_field.update_view()
	_block_field.update_view()
