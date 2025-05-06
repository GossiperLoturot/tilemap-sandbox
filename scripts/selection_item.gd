extends Control


@export var label: Label


# invoked dynamicaly
func change_selection_item(text: String) -> void:
	label.text = text
