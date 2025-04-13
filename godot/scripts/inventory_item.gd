extends Control


@export var placeholder_node: Control
var _inventory_key: int
var _local_key: int


func _process(_delta: float) -> void:
	Root.draw_item(_inventory_key, _local_key, placeholder_node)


# invoked dynamicaly
func on_inventory_item_changed(inventory_key: int, local_key: int) -> void:
	_inventory_key = inventory_key
	_local_key = local_key
