extends Control


@export var label: Label


func stream_changed(name: String) -> void:
	label.text = "♪ %s" % name

	var tween = label.create_tween()
	tween.tween_property(label, "modulate:a", 1.0, 1.0)
	tween.tween_interval(5.0)
	tween.tween_property(label, "modulate:a", 0.0, 1.0)
	tween.tween_callback(tween.kill)
