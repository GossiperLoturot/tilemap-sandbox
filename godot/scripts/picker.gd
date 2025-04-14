extends Control


@export var item_scene: PackedScene
@export var item_deploy: Node


# invoked dynamicaly
func change_pick(texts: Array[String], screen_position: Vector2) -> void:
	for child in item_deploy.get_children():
		child.queue_free()

	for text in texts:
		var item_instance = item_scene.instantiate()
		item_deploy.add_child(item_instance)
		item_instance.call("change_pick_item", text)

	self.position = screen_position
