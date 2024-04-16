extends Node3D


@export var camera: Camera3D

var _ray_request: RayRequest


func _physics_process(delta):
	if _ray_request:
		var space_state = get_world_3d().direct_space_state
		var query = PhysicsRayQueryParameters3D.create(_ray_request.from, _ray_request.to)
		query.collide_with_bodies = true
		var result = space_state.intersect_ray(query)
		
		print(result)
		
		_ray_request = null


func _input(event):
	if event is InputEventMouseButton and event.pressed and event.button_index == 1:
		var from = camera.project_ray_origin(event.position)
		var to = from + camera.project_ray_normal(event.position) * camera.far
		_ray_request = RayRequest.new(from, to)
		
		print(_ray_request)


class RayRequest:
	var from: Vector3
	var to: Vector3
	
	func _init(from: Vector3, to: Vector3):
		self.from = from
		self.to = to
	
	func _to_string():
		return "<RayRequest from:%s to:%s>" % [from, to]
