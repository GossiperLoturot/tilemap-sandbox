extends Node3D
class_name World


@export var min_forwarder_rect: Rect2
@export var min_gen_rect: Rect2
@export var min_view_rect: Rect2

var _root: Root


func _ready() -> void:
	_root = Root.create(get_world_3d())


func _process(delta_secs) -> void:
	# logic
	_root.forwarder_exec_rect(min_forwarder_rect, delta_secs)

	_root.gen_exec_rect(min_gen_rect)

	_root.time_forward(delta_secs)

	# rendering
	_root.update_view(min_view_rect)
