@tool
extends EditorScript

func _run():
	var root_path := "res://images"
	var dir := DirAccess.open(root_path)
	
	var images: Array[Image] = []
	for file_name in dir.get_files():
		var file_path := root_path.path_join(file_name)
		var image: Image = ResourceLoader.load(file_path, "Image")
		
		if !image:
			continue
		
		var file_stem := file_name.split(".")[0]
		
		# print("image name: %s" % file_stem)
		images.append(image)
	
	var desc = AtlasDesc.new()
	desc.image_size = 2048
	desc.max_page_size = 8
	desc.images = images
	
	ResourceSaver.save(desc, "res://atlas.tres")
