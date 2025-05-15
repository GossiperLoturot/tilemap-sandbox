extends Control


@export var placeholder_node: Control
var _slot_key: SlotKey
var _is_mouse_entered: bool


func on_instantiate(slot_key: SlotKey) -> void:
	_slot_key = slot_key


func _process(_delta: float) -> void:
	Context.draw_slot(_slot_key, placeholder_node)

	if _is_mouse_entered:
		Selection.context.select_item(_slot_key)
		if Input.is_action_just_pressed("primary"):
			Selection.context.confirm_item(_slot_key)


func _on_mouse_entered() -> void:
	_is_mouse_entered = true
	Selection.context.show_selection()


func _on_mouse_exited() -> void:
	_is_mouse_entered = false
	Selection.context.hide_selection()
