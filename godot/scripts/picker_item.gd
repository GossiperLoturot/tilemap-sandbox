extends Control


@export var label: Label


# invoked dynamicaly
func change_pick_item(text: String) -> void:
	label.text = text
