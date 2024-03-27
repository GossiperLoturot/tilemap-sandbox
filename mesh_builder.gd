class_name MeshBuilder
extends MultiMeshInstance2D


@export var mesh: Mesh


func _ready():
	multimesh = MultiMesh.new()
	multimesh.transform_format = MultiMesh.TRANSFORM_2D
	multimesh.instance_count = 1000 * 1000
	multimesh.visible_instance_count = 1000 * 1000
	
	multimesh.mesh = mesh
	var buffer: PackedFloat32Array = []
	
	for y in range(1000):
		for x in range(1000):
			buffer.append_array([1.0, 0.0, 0.0, float(x), 0.0, 1.0, 0.0, float(y)])
	
	multimesh.buffer = buffer
