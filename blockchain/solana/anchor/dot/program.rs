#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
use crate::{id, seahorse_util::*};
use anchor_lang::{prelude::*, solana_program};
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct Coordinates {
    pub lat: f64,
    pub long: f64,
}

#[account]
#[derive(Debug)]
pub struct Depot {
    pub owner: Pubkey,
    pub name_array: [u16; 32],
    pub info_array: [u16; 256],
    pub coordinates_class: Coordinates,
    pub time_created: i64,
}

impl<'info, 'entrypoint> Depot {
    pub fn load(
        account: &'entrypoint mut Box<Account<'info, Self>>,
        programs_map: &'entrypoint ProgramsMap<'info>,
    ) -> Mutable<LoadedDepot<'info, 'entrypoint>> {
        let owner = account.owner.clone();
        let name_array = Mutable::new(account.name_array.clone());
        let info_array = Mutable::new(account.info_array.clone());
        let coordinates_class =
            Mutable::new(account.coordinates_class.clone());

        let time_created = account.time_created;

        Mutable::new(LoadedDepot {
            __account__: account,
            __programs__: programs_map,
            owner,
            name_array,
            info_array,
            coordinates_class,
            time_created,
        })
    }

    pub fn store(loaded: Mutable<LoadedDepot>) {
        let mut loaded = loaded.borrow_mut();
        let owner = loaded.owner.clone();

        loaded.__account__.owner = owner;

        let name_array = loaded.name_array.borrow().clone();

        loaded.__account__.name_array = name_array;

        let info_array = loaded.info_array.borrow().clone();

        loaded.__account__.info_array = info_array;

        let coordinates_class = loaded.coordinates_class.borrow().clone();

        loaded.__account__.coordinates_class = coordinates_class;

        let time_created = loaded.time_created;

        loaded.__account__.time_created = time_created;
    }
}

#[derive(Debug)]
pub struct LoadedDepot<'info, 'entrypoint> {
    pub __account__: &'entrypoint mut Box<Account<'info, Depot>>,
    pub __programs__: &'entrypoint ProgramsMap<'info>,
    pub owner: Pubkey,
    pub name_array: Mutable<[u16; 32]>,
    pub info_array: Mutable<[u16; 256]>,
    pub coordinates_class: Mutable<Coordinates>,
    pub time_created: i64,
}

#[account]
#[derive(Debug)]
pub struct Items {
    pub owner: Pubkey,
    pub depot: Pubkey,
    pub name_array: [u16; 32],
    pub info_array: [u16; 256],
    pub coordinates_class: Coordinates,
    pub cur_time: i64,
    pub time_created: i64,
}

impl<'info, 'entrypoint> Items {
    pub fn load(
        account: &'entrypoint mut Box<Account<'info, Self>>,
        programs_map: &'entrypoint ProgramsMap<'info>,
    ) -> Mutable<LoadedItems<'info, 'entrypoint>> {
        let owner = account.owner.clone();
        let depot = account.depot.clone();
        let name_array = Mutable::new(account.name_array.clone());
        let info_array = Mutable::new(account.info_array.clone());
        let coordinates_class =
            Mutable::new(account.coordinates_class.clone());

        let cur_time = account.cur_time;
        let time_created = account.time_created;

        Mutable::new(LoadedItems {
            __account__: account,
            __programs__: programs_map,
            owner,
            depot,
            name_array,
            info_array,
            coordinates_class,
            cur_time,
            time_created,
        })
    }

    pub fn store(loaded: Mutable<LoadedItems>) {
        let mut loaded = loaded.borrow_mut();
        let owner = loaded.owner.clone();

        loaded.__account__.owner = owner;

        let depot = loaded.depot.clone();

        loaded.__account__.depot = depot;

        let name_array = loaded.name_array.borrow().clone();

        loaded.__account__.name_array = name_array;

        let info_array = loaded.info_array.borrow().clone();

        loaded.__account__.info_array = info_array;

        let coordinates_class = loaded.coordinates_class.borrow().clone();

        loaded.__account__.coordinates_class = coordinates_class;

        let cur_time = loaded.cur_time;

        loaded.__account__.cur_time = cur_time;

        let time_created = loaded.time_created;

        loaded.__account__.time_created = time_created;
    }
}

#[derive(Debug)]
pub struct LoadedItems<'info, 'entrypoint> {
    pub __account__: &'entrypoint mut Box<Account<'info, Items>>,
    pub __programs__: &'entrypoint ProgramsMap<'info>,
    pub owner: Pubkey,
    pub depot: Pubkey,
    pub name_array: Mutable<[u16; 32]>,
    pub info_array: Mutable<[u16; 256]>,
    pub coordinates_class: Mutable<Coordinates>,
    pub cur_time: i64,
    pub time_created: i64,
}

pub fn init_depot_handler<'info>(
    mut clock: Sysvar<'info, Clock>,
    mut payer: SeahorseSigner<'info, '_>,
    mut owner: SeahorseSigner<'info, '_>,
    mut depot: Empty<Mutable<LoadedDepot<'info, '_>>>,
    mut name_array: [u16; 32],
    mut info_array: [u16; 256],
    mut coordinates_class: Coordinates,
    mut seed_random: u128,
) -> () {
    let mut time = clock.unix_timestamp;
    let mut depot = depot.account.clone();

    assign!(depot.borrow_mut().owner, owner.key());

    assign!(depot.borrow_mut().name_array, Mutable::<[u16; 32]>::new(name_array));

    assign!(depot.borrow_mut().info_array, Mutable::<[u16; 256]>::new(info_array));

    assign!(depot.borrow_mut().coordinates_class, Mutable::<Coordinates>::new(coordinates_class));

    assign!(depot.borrow_mut().time_created, time);
}

pub fn init_items_handler<'info>(
    mut clock: Sysvar<'info, Clock>,
    mut payer: SeahorseSigner<'info, '_>,
    mut owner: SeahorseSigner<'info, '_>,
    mut depot_signer: SeahorseSigner<'info, '_>,
    mut depot: Mutable<LoadedDepot<'info, '_>>,
    mut items: Empty<Mutable<LoadedItems<'info, '_>>>,
    mut name_array: [u16; 32],
    mut info_array: [u16; 256],
    mut coordinates_class: Coordinates,
    mut seed_random: u128,
) -> () {
    let mut time = clock.unix_timestamp;

    if !(depot.borrow().owner == depot_signer.key()) {
        panic!("Depot signer is not the owner");
    }

    let mut items = items.account.clone();

    assign!(items.borrow_mut().owner, owner.key());

    assign!(items.borrow_mut().depot, depot.borrow().__account__.key());

    assign!(items.borrow_mut().name_array, Mutable::<[u16; 32]>::new(name_array));

    assign!(items.borrow_mut().info_array, Mutable::<[u16; 256]>::new(info_array));

    assign!(items.borrow_mut().coordinates_class, Mutable::<Coordinates>::new(coordinates_class));

    assign!(items.borrow_mut().cur_time, time);

    assign!(items.borrow_mut().time_created, time);
}

pub fn transfer_items_handler<'info>(
    mut clock: Sysvar<'info, Clock>,
    mut payer: SeahorseSigner<'info, '_>,
    mut old_depot_owner: SeahorseSigner<'info, '_>,
    mut new_depot_owner: SeahorseSigner<'info, '_>,
    mut old_depot: Mutable<LoadedDepot<'info, '_>>,
    mut new_depot: Mutable<LoadedDepot<'info, '_>>,
    mut items: Mutable<LoadedItems<'info, '_>>,
) -> () {
    let mut time = clock.unix_timestamp;

    if !(old_depot.borrow().owner == old_depot_owner.key()) {
        panic!("Old depot owner is not the owner of old depot");
    }

    if !(new_depot.borrow().owner == new_depot_owner.key()) {
        panic!("New depot owner is not the owner of new depot");
    }

    if !(items.borrow().depot == old_depot.borrow().__account__.key()) {
        panic!("Old depot is not same as items depot");
    }

    if !(old_depot.borrow().__account__.key() != new_depot.borrow().__account__.key()) {
        panic!("Old depot is same as new depot");
    }

    assign!(
        items.borrow_mut().depot,
        new_depot.borrow().__account__.key()
    );

    assign!(items.borrow_mut().cur_time, time);
}

pub fn update_items_handler<'info>(
    mut clock: Sysvar<'info, Clock>,
    mut payer: SeahorseSigner<'info, '_>,
    mut owner: SeahorseSigner<'info, '_>,
    mut items: Mutable<LoadedItems<'info, '_>>,
    mut coordinates_class: Coordinates,
) -> () {
    let mut time = clock.unix_timestamp;

    if !(items.borrow().owner == owner.key()) {
        panic!("Owner is not the owner");
    }

    assign!(items.borrow_mut().coordinates_class, Mutable::<Coordinates>::new(coordinates_class));

    assign!(items.borrow_mut().cur_time, time);
}
