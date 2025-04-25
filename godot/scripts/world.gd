extends Node3D


@export var ui: Control
var _forwarder_rect: Rect2
var _gen_rect: Rect2
var _view_rect: Rect2


# preload shaders for cache
func _init() -> void:
	preload("res://shaders/field.gdshader")
	preload("res://shaders/field_shadow.gdshader")
	preload("res://shaders/pick.gdshader")


func _enter_tree() -> void:
	Root.open(self.get_world_3d(), ui)


func _process(delta: float) -> void:
	# logic
	Root.forwarder_exec_rect(_forwarder_rect, delta)
	Root.gen_exec_rect(_gen_rect)
	Root.time_forward(delta)
	# rendering
	Root.update_view(_view_rect)


func _exit_tree() -> void:
	Root.close()


func _on_forwarder_rect_changed(rect: Rect2) -> void:
	_forwarder_rect = rect


func _on_gen_rect_changed(rect: Rect2) -> void:
	_gen_rect = rect


func _on_view_rect_changed(rect: Rect2) -> void:
	_view_rect = rect
