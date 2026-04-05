extends Node3D


var _gen_rect: Rect2
var _view_rect: Rect2


func _enter_tree() -> void:
	_warmup()

	Context.open(self.get_viewport(), _callback)
	# spawn player
	Context.spawn_player()
	# spawn 65,536 animal for load-test
	Context.spawn_bulk_animal()


func _process(delta: float) -> void:
	_logic_process(delta)
	_draw_process()


func _logic_process(delta: float) -> void:
	Context.process(delta)
	Context.generate_field(_gen_rect)


func _draw_process() -> void:
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


func _callback():
	pass
