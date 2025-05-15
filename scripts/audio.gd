extends AudioStreamPlayer


signal stream_changed(name_text: String)

@export var names: Array[String]
@export var streams: Array[AudioStream]


func next_stream() -> void:
	var i = randi_range(0, streams.size() - 1)
	self.stream = streams[i]
	self.play()

	stream_changed.emit(names[i])
