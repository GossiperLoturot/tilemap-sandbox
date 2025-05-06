extends Node


func _enter_tree() -> void:
	PanicHook.open()


func _exit_tree() -> void:
	PanicHook.close()
