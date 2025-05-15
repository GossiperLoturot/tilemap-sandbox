extends Control


@export var label: Label


func _on_stream_changed(name_text: String) -> void:
	label.text = "♪ %s" % name_text

	var tween = label.create_tween()
	tween.tween_property(self, "visible", true, 0.0)
	tween.tween_property(self, "modulate:a", 1.0, 1.0)
	tween.tween_interval(5.0)
	tween.tween_property(self, "modulate:a", 0.0, 1.0)
	tween.tween_property(self, "visible", false, 0.0)
