extends Node


@export var atlas_desc: AtlasDesc
var _atlas: Atlas

func _ready():
	_atlas = Atlas.from_desc(atlas_desc)
	for texcoord in _atlas.texcoords():
		var page: int = texcoord.page()
		var min_x: float = texcoord.min_x()
		var min_y: float = texcoord.max_y()
		var max_x: float = texcoord.max_x()
		var max_y: float = texcoord.max_y()
		print("page: %s, max-x: %s, min-y: %s, max-x: %s, max-y: %s" % [page, min_x, min_y, max_x, max_y])
