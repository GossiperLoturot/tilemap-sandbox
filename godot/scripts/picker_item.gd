extends Control


@export var label: Label


# invoked dynamicaly
func on_pick_item_changed(text: String) -> void:
	label.text = text
