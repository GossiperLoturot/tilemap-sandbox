extends Control


@export var item_scene: PackedScene
@export var item_deploy: Node
var _world_position: Vector2


func _process(_delta: float) -> void:
	for child in item_deploy.get_children():
		child.queue_free()

	for i in Root.get_pick_size(_world_position):
		var item_instance = item_scene.instantiate()
		item_deploy.add_child(item_instance)

		var text = Root.get_pick_name_text(_world_position, i)
		item_instance.call("on_pick_item_changed", text)


# invoked dynamicaly
func on_pick_changed(world_position: Vector2, screen_position: Vector2) -> void:
	_world_position = world_position
	self.position = screen_position


# invoked dynamicaly
func on_pick_entered() -> void:
	self.show()


# invoked dynamicaly
func on_pick_existed() -> void:
	self.hide()
