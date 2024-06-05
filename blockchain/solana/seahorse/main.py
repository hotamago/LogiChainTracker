# Built with Seahorse v0.2.0

from seahorse.prelude import *

# This is your program's public key and it will update
# automatically when you build the project.
declare_id('91RQeXbQroq5oct6TZuFVLZJrE93q2eR8G5ntdXE69YN')

class Coordinates:
    lat: f64 # 8 bytes
    long: f64 # 8 bytes

class Items(Account):
    owner: Pubkey # 32 bytes
    depot: Pubkey # 32 bytes
    name_u16_32_array: Array[u16, 32] # 64 bytes
    info_u16_256_array: Array[u16, 256] # 512 bytes
    coordinates_Coordinates_class: Coordinates # 16 bytes
    cur_time: i64 # 8 bytes
    time_created: i64 # 8 bytes

class Depot(Account):
    owner: Pubkey # 32 bytes
    name_u16_32_array: Array[u16, 32] # 64 bytes
    info_u16_256_array: Array[u16, 256] # 512 bytes
    coordinates_Coordinates_class: Coordinates # 16 bytes
    time_created: i64 # 8 bytes

@instruction
def init_depot(
    clock: Clock,
    payer: Signer,
    owner: Signer,
    depot: Empty[Depot],
    name_u16_32_array: Array[u16, 32],
    info_u16_256_array: Array[u16, 256],
    coordinates_Coordinates_class: Coordinates,
    seed_random: u128
):
    time: i64 = clock.unix_timestamp()
    depot = depot.init(payer = payer, seeds = [owner, "depot", seed_random])
    depot.owner = owner.key()
    depot.name_u16_32_array = name_u16_32_array
    depot.info_u16_256_array = info_u16_256_array
    depot.coordinates_Coordinates_class = coordinates_Coordinates_class
    depot.time_created = time

@instruction
def init_items(
    clock: Clock,
    payer: Signer,
    owner: Signer,
    depot_signer: Signer,
    depot: Depot,
    items: Empty[Items],
    name_u16_32_array: Array[u16, 32],
    info_u16_256_array: Array[u16, 256],
    coordinates_Coordinates_class: Coordinates,
    seed_random: u128
):
    time: i64 = clock.unix_timestamp()
    assert depot.owner == depot_signer.key(), "Depot signer is not the owner"

    items = items.init(payer = payer, seeds = [owner, depot_signer, "items", seed_random])
    items.owner = owner.key()
    items.depot = depot.key()
    items.name_u16_32_array = name_u16_32_array
    items.info_u16_256_array = info_u16_256_array
    items.coordinates_Coordinates_class = coordinates_Coordinates_class
    items.cur_time = time
    items.time_created = time

@instruction
def update_items(
    clock: Clock,
    payer: Signer,
    owner: Signer,
    items: Items,
    coordinates_Coordinates_class: Coordinates
):
    time: i64 = clock.unix_timestamp()
    assert items.owner == owner.key(), "Owner is not the owner"

    items.coordinates_Coordinates_class = coordinates_Coordinates_class
    items.cur_time = time

@instruction
def transfer_items(
    clock: Clock,
    payer: Signer,
    old_depot_owner: Signer,
    new_depot_owner: Signer,
    old_depot: Depot,
    new_depot: Depot,
    items: Items,
):
    time: i64 = clock.unix_timestamp()
    assert old_depot.owner == old_depot_owner.key(), "Old depot owner is not the owner of old depot"
    assert new_depot.owner == new_depot_owner.key(), "New depot owner is not the owner of new depot"
    assert items.depot == old_depot.key(), "Old depot is not same as items depot"
    assert old_depot.key() != new_depot.key(), "Old depot is same as new depot"

    items.depot = new_depot.key()
    items.cur_time = time