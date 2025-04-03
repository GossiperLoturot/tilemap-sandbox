extends Control
class_name InventoryPlayer


@export var item_nodes: Array[Control]

var _inventory_key: int


# invoked by the instantiate function
func set_inventory_key(inventory_key: int) -> void:
	_inventory_key = inventory_key


# invoked by the signal
func close_inventory() -> void:
	self.queue_free()


func _process(_delta: float) -> void:
	for i in item_nodes.size():
		if Root.item_has_item(_inventory_key, i):
			Root.item_draw_view(_inventory_key, i, item_nodes[i])
