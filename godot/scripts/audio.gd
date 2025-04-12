extends AudioStreamPlayer


@export var names: Array[String]
@export var streams: Array[AudioStream]

signal stream_changed(name: String)


func _enter_tree() -> void:
	_next_stream()


# when finished stream
func _on_finished() -> void:
	_next_stream()


func _next_stream() -> void:
	var i = randi_range(0, streams.size() - 1)

	stream_changed.emit(names[i])

	self.stream = streams[i]
	self.play()
