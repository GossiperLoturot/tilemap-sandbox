extends AudioStreamPlayer


@export var names: Array[String]
@export var streams: Array[AudioStream]

signal stream_changed(name_text: String)


func _enter_tree() -> void:
	self.connect("finished", _on_finished)

	next_stream()


func _on_finished() -> void:
	next_stream()


func next_stream() -> void:
	var i = randi_range(0, streams.size() - 1)

	stream_changed.emit(names[i])

	self.stream = streams[i]
	self.play()
