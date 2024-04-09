extends MultiMeshInstance3D


@export var atlas_desc: AtlasDesc
@export var mesh: Mesh


func _ready():
	var atlas = Atlas.from_desc(atlas_desc)
	
	var textures = Texture2DArray.new()
	textures.create_from_images(atlas.images())
	
	var material: ShaderMaterial = mesh.surface_get_material(0)
	material.set_shader_parameter("textures", textures)
	
	var field = TileField.new()
	for y in range(32):
		for x in range(32):
			field.add_tile(Tile.from(12, x, y))
	
	multimesh = MultiMesh.new()
	multimesh.mesh = mesh
	multimesh.transform_format = MultiMesh.TRANSFORM_3D
	multimesh.instance_count = 32 * 32
	
	field.update_buffer(multimesh, material, atlas)
