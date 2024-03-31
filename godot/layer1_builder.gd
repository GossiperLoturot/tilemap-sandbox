class_name Layer1Builder
extends MultiMeshInstance2D


@export var mesh: Mesh
@export var size: int = 32


func _ready():
	multimesh = MultiMesh.new();
	multimesh.mesh = mesh
	multimesh.transform_format = MultiMesh.TRANSFORM_2D
	multimesh.instance_count = size
	multimesh.visible_instance_count = size
	
	var buffer: PackedFloat32Array = []
	for x in range(size):
		buffer.append_array([1.0, 0.0, 0.0, float(x), 0.0, 1.0, 0.0, 0.0])
	
	multimesh.buffer = buffer
