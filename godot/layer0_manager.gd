class_name Layer0Manager
extends Node2D


@export var mesh: Mesh
@export var builder_size: int = 8
@export var instance_size: int = 32


func _ready():
	for y in range(builder_size):
		for x in range(builder_size):
			var builder = Layer0Builder.new()
			builder.transform.origin = Vector2(x * instance_size, y * instance_size)
			builder.mesh = mesh
			builder.size = instance_size
			add_child(builder)
