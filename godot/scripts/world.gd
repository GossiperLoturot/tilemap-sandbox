extends Node3D


@export var min_forwarder_rect: Rect2
@export var min_gen_rect: Rect2
@export var min_view_rect: Rect2


# preload shaders for cache
func _init() -> void:
	preload("res://shaders/field.gdshader")
	preload("res://shaders/field_shadow.gdshader")


func _enter_tree() -> void:
	Root.open(get_world_3d(), self)


func _process(delta: float) -> void:
	# logic
	Root.forwarder_exec_rect(min_forwarder_rect, delta)
	Root.gen_exec_rect(min_gen_rect)
	Root.time_forward(delta)
	# rendering
	Root.update_view(min_view_rect)


func _exit_tree() -> void:
	Root.close()


func change_min_forwarder_rect(rect: Rect2) -> void:
	min_forwarder_rect = rect


func change_min_gen_rect(rect: Rect2) -> void:
	min_gen_rect = rect


func change_min_view_rect(rect: Rect2) -> void:
	min_view_rect = rect
