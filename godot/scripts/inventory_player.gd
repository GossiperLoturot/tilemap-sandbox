extends Control
class_name Inventory


@export var _inventory_key: int
@export var _slot_nodes: Array[Control]


func set_inventory_key(inventory_key: int) -> void:
	_inventory_key = inventory_key


func get_slot_nodes() -> Array[Control]:
	return _slot_nodes


func close_inventory() -> void:
	print("CLOSE")
	queue_free()
