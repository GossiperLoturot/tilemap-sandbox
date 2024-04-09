extends MultiMeshInstance3D


@export var atlas_desc: AtlasDesc
@export var mesh: Mesh


func _ready():
	var tile_field = TileField.new_from(atlas_desc)
	
	var material = mesh.surface_get_material(0)
	material.set_shader_parameter("textures", tile_field.get_texture_array())
	
	multimesh = MultiMesh.new()
	multimesh.mesh = mesh
	multimesh.transform_format = MultiMesh.TRANSFORM_3D
	multimesh.instance_count = 32 * 32
	
	for y in range(32):
		for x in range(32):
			tile_field.add_tile(Tile.new_from(12, x, y))
	
	tile_field.update_buffer(multimesh.get_rid(), material.get_rid())
