extends Control
class_name InventoryPlayer


var _inventory_key: int


# invoked by the instantiate function
func set_inventory_key(inventory_key: int) -> void:
	_inventory_key = inventory_key


# invoked by the signal
func close_inventory() -> void:
	self.queue_free()
