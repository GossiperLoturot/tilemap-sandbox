extends Node3D
class_name World


@export var min_forwarder_rect: Rect2
@export var min_gen_rect: Rect2
@export var min_view_rect: Rect2


# preload shaders for cache
func _init() -> void:
	preload("res://shaders/field.gdshader")
	preload("res://shaders/field_shadow.gdshader")


func _enter_tree() -> void:
	Root.open(get_world_3d(), self)


func _process(delta_secs) -> void:
	# logic
	Root.forwarder_exec_rect(min_forwarder_rect, delta_secs)
	Root.gen_exec_rect(min_gen_rect)
	Root.time_forward(delta_secs)
	# rendering
	Root.update_view(min_view_rect)


func _exit_tree() -> void:
	Root.close()
