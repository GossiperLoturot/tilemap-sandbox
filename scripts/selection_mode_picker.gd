extends Control
class_name SelectionModePicker


static var context: SelectionModePicker

const MODE_NONE: int = 0
const MODE_TILE: int = 1
const MODE_BLOCK: int = 2
const MODE_ENTITY: int = 3
var mode: int = MODE_NONE


func _enter_tree() -> void:
	context = self


func _exit_tree() -> void:
	context = null


func get_mode() -> int:
	return self.mode


func set_mode(mode: int):
	self.mode = mode
