extends Node
class_name Hook


func _enter_tree() -> void:
	PanicHook.open()


func _exit_tree() -> void:
	PanicHook.close()
