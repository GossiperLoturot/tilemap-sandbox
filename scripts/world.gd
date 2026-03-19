extends Node3D


var _gen_rect: Rect2
var _view_rect: Rect2


func _enter_tree() -> void:
	_warmup()

	Context.open(self.get_viewport())
	# spawn player
	Context.spawn_player()
	# spawn 65,536 animal for load-test
	for y in range(-128, 127):
		for x in range(-128, 127):
			Context.spawn_animal(Vector2(x, y))


func _process(delta: float) -> void:
	# logic
	Context.forward_time(delta)
	Context.generate_field(_gen_rect)
	# rendering
	Context.draw_field(_view_rect)


func _exit_tree() -> void:
	Context.close()


func _on_gen_rect_changed(rect: Rect2) -> void:
	_gen_rect = rect


func _on_view_rect_changed(rect: Rect2) -> void:
	_view_rect = rect


func _warmup():
	preload("res://shaders/field.gdshader")
	preload("res://shaders/field_shadow.gdshader")
