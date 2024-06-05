#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]

pub mod dot;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};

use dot::program::*;
use std::{cell::RefCell, rc::Rc};

declare_id!("91RQeXbQroq5oct6TZuFVLZJrE93q2eR8G5ntdXE69YN");

pub mod seahorse_util {
    use super::*;

    #[cfg(feature = "pyth-sdk-solana")]
    pub use pyth_sdk_solana::{load_price_feed_from_account_info, PriceFeed};
    use std::{collections::HashMap, fmt::Debug, ops::Deref};

    pub struct Mutable<T>(Rc<RefCell<T>>);

    impl<T> Mutable<T> {
        pub fn new(obj: T) -> Self {
            Self(Rc::new(RefCell::new(obj)))
        }
    }

    impl<T> Clone for Mutable<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    impl<T> Deref for Mutable<T> {
        type Target = Rc<RefCell<T>>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T: Debug> Debug for Mutable<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }

    impl<T: Default> Default for Mutable<T> {
        fn default() -> Self {
            Self::new(T::default())
        }
    }

    impl<T: Clone> Mutable<Vec<T>> {
        pub fn wrapped_index(&self, mut index: i128) -> usize {
            if index >= 0 {
                return index.try_into().unwrap();
            }

            index += self.borrow().len() as i128;

            return index.try_into().unwrap();
        }
    }

    impl<T: Clone, const N: usize> Mutable<[T; N]> {
        pub fn wrapped_index(&self, mut index: i128) -> usize {
            if index >= 0 {
                return index.try_into().unwrap();
            }

            index += self.borrow().len() as i128;

            return index.try_into().unwrap();
        }
    }

    #[derive(Clone)]
    pub struct Empty<T: Clone> {
        pub account: T,
        pub bump: Option<u8>,
    }

    #[derive(Clone, Debug)]
    pub struct ProgramsMap<'info>(pub HashMap<&'static str, AccountInfo<'info>>);

    impl<'info> ProgramsMap<'info> {
        pub fn get(&self, name: &'static str) -> AccountInfo<'info> {
            self.0.get(name).unwrap().clone()
        }
    }

    #[derive(Clone, Debug)]
    pub struct WithPrograms<'info, 'entrypoint, A> {
        pub account: &'entrypoint A,
        pub programs: &'entrypoint ProgramsMap<'info>,
    }

    impl<'info, 'entrypoint, A> Deref for WithPrograms<'info, 'entrypoint, A> {
        type Target = A;

        fn deref(&self) -> &Self::Target {
            &self.account
        }
    }

    pub type SeahorseAccount<'info, 'entrypoint, A> =
        WithPrograms<'info, 'entrypoint, Box<Account<'info, A>>>;

    pub type SeahorseSigner<'info, 'entrypoint> = WithPrograms<'info, 'entrypoint, Signer<'info>>;

    #[derive(Clone, Debug)]
    pub struct CpiAccount<'info> {
        #[doc = "CHECK: CpiAccounts temporarily store AccountInfos."]
        pub account_info: AccountInfo<'info>,
        pub is_writable: bool,
        pub is_signer: bool,
        pub seeds: Option<Vec<Vec<u8>>>,
    }

    #[macro_export]
    macro_rules! seahorse_const {
        ($ name : ident , $ value : expr) => {
            macro_rules! $name {
                () => {
                    $value
                };
            }

            pub(crate) use $name;
        };
    }

    #[macro_export]
    macro_rules! assign {
        ($ lval : expr , $ rval : expr) => {{
            let temp = $rval;

            $lval = temp;
        }};
    }

    #[macro_export]
    macro_rules! index_assign {
        ($ lval : expr , $ idx : expr , $ rval : expr) => {
            let temp_rval = $rval;
            let temp_idx = $idx;

            $lval[temp_idx] = temp_rval;
        };
    }

    pub(crate) use assign;

    pub(crate) use index_assign;

    pub(crate) use seahorse_const;
}

#[program]
mod electra_chain {
    use super::*;
    use seahorse_util::*;
    use std::collections::HashMap;

    #[derive(Accounts)]
    # [instruction (name_array: [u16; 32] , info_array: [u16; 256] , coordinates_class: Coordinates , seed_random : u128)]
    pub struct InitDepot<'info> {
        #[account()]
        pub clock: Sysvar<'info, Clock>,
        #[account(mut)]
        pub payer: Signer<'info>,
        #[account(mut)]
        pub owner: Signer<'info>,
        # [account (init , space = std :: mem :: size_of :: < dot :: program :: Depot > () + 8 , payer = payer , seeds = [owner . key () . as_ref () , "depot" . as_bytes () . as_ref () , seed_random . to_le_bytes () . as_ref ()] , bump)]
        pub depot: Box<Account<'info, dot::program::Depot>>,
        pub rent: Sysvar<'info, Rent>,
        pub system_program: Program<'info, System>,
    }

    pub fn init_depot(
        ctx: Context<InitDepot>,
        name_array: [u16; 32],
        info_array: [u16; 256],
        coordinates_class: Coordinates,
        seed_random: u128,
    ) -> Result<()> {
        let mut programs = HashMap::new();

        programs.insert(
            "system_program",
            ctx.accounts.system_program.to_account_info(),
        );

        let programs_map = ProgramsMap(programs);
        let clock = &ctx.accounts.clock.clone();
        let payer = SeahorseSigner {
            account: &ctx.accounts.payer,
            programs: &programs_map,
        };

        let owner = SeahorseSigner {
            account: &ctx.accounts.owner,
            programs: &programs_map,
        };

        let depot = Empty {
            account: dot::program::Depot::load(&mut ctx.accounts.depot, &programs_map),
            bump: Some(ctx.bumps.depot),
        };

        init_depot_handler(
            clock.clone(),
            payer.clone(),
            owner.clone(),
            depot.clone(),
            name_array,
            info_array,
            coordinates_class,
            seed_random,
        );

        dot::program::Depot::store(depot.account);

        return Ok(());
    }

    #[derive(Accounts)]
    # [instruction (name_array: [u16; 32] , info_array: [u16; 256] , coordinates_class: Coordinates , seed_random : u128)]
    pub struct InitItems<'info> {
        #[account()]
        pub clock: Sysvar<'info, Clock>,
        #[account(mut)]
        pub payer: Signer<'info>,
        #[account(mut)]
        pub owner: Signer<'info>,
        #[account(mut)]
        pub depot_signer: Signer<'info>,
        #[account(mut)]
        pub depot: Box<Account<'info, dot::program::Depot>>,
        # [account (init , space = std :: mem :: size_of :: < dot :: program :: Items > () + 8 , payer = payer , seeds = [owner . key () . as_ref () , depot_signer . key () . as_ref () , "items" . as_bytes () . as_ref () , seed_random . to_le_bytes () . as_ref ()] , bump)]
        pub items: Box<Account<'info, dot::program::Items>>,
        pub rent: Sysvar<'info, Rent>,
        pub system_program: Program<'info, System>,
    }

    pub fn init_items(
        ctx: Context<InitItems>,
        name_array: [u16; 32],
        info_array: [u16; 256],
        coordinates_class: Coordinates,
        seed_random: u128,
    ) -> Result<()> {
        let mut programs = HashMap::new();

        programs.insert(
            "system_program",
            ctx.accounts.system_program.to_account_info(),
        );

        let programs_map = ProgramsMap(programs);
        let clock = &ctx.accounts.clock.clone();
        let payer = SeahorseSigner {
            account: &ctx.accounts.payer,
            programs: &programs_map,
        };

        let owner = SeahorseSigner {
            account: &ctx.accounts.owner,
            programs: &programs_map,
        };

        let depot_signer = SeahorseSigner {
            account: &ctx.accounts.depot_signer,
            programs: &programs_map,
        };

        let depot = dot::program::Depot::load(&mut ctx.accounts.depot, &programs_map);
        let items = Empty {
            account: dot::program::Items::load(&mut ctx.accounts.items, &programs_map),
            bump: Some(ctx.bumps.items),
        };

        init_items_handler(
            clock.clone(),
            payer.clone(),
            owner.clone(),
            depot_signer.clone(),
            depot.clone(),
            items.clone(),
            name_array,
            info_array,
            coordinates_class,
            seed_random,
        );

        dot::program::Depot::store(depot);

        dot::program::Items::store(items.account);

        return Ok(());
    }

    #[derive(Accounts)]
    pub struct TransferItems<'info> {
        #[account()]
        pub clock: Sysvar<'info, Clock>,
        #[account(mut)]
        pub payer: Signer<'info>,
        #[account(mut)]
        pub old_depot_owner: Signer<'info>,
        #[account(mut)]
        pub new_depot_owner: Signer<'info>,
        #[account(mut)]
        pub old_depot: Box<Account<'info, dot::program::Depot>>,
        #[account(mut)]
        pub new_depot: Box<Account<'info, dot::program::Depot>>,
        #[account(mut)]
        pub items: Box<Account<'info, dot::program::Items>>,
    }

    pub fn transfer_items(ctx: Context<TransferItems>) -> Result<()> {
        let mut programs = HashMap::new();
        let programs_map = ProgramsMap(programs);
        let clock = &ctx.accounts.clock.clone();
        let payer = SeahorseSigner {
            account: &ctx.accounts.payer,
            programs: &programs_map,
        };

        let old_depot_owner = SeahorseSigner {
            account: &ctx.accounts.old_depot_owner,
            programs: &programs_map,
        };

        let new_depot_owner = SeahorseSigner {
            account: &ctx.accounts.new_depot_owner,
            programs: &programs_map,
        };

        let old_depot = dot::program::Depot::load(&mut ctx.accounts.old_depot, &programs_map);
        let new_depot = dot::program::Depot::load(&mut ctx.accounts.new_depot, &programs_map);
        let items = dot::program::Items::load(&mut ctx.accounts.items, &programs_map);

        transfer_items_handler(
            clock.clone(),
            payer.clone(),
            old_depot_owner.clone(),
            new_depot_owner.clone(),
            old_depot.clone(),
            new_depot.clone(),
            items.clone(),
        );

        dot::program::Depot::store(old_depot);

        dot::program::Depot::store(new_depot);

        dot::program::Items::store(items);

        return Ok(());
    }

    #[derive(Accounts)]
    # [instruction (coordinates_class: Coordinates)]
    pub struct UpdateItems<'info> {
        #[account()]
        pub clock: Sysvar<'info, Clock>,
        #[account(mut)]
        pub payer: Signer<'info>,
        #[account(mut)]
        pub owner: Signer<'info>,
        #[account(mut)]
        pub items: Box<Account<'info, dot::program::Items>>,
    }

    pub fn update_items(
        ctx: Context<UpdateItems>,
        coordinates_class: Coordinates,
    ) -> Result<()> {
        let mut programs = HashMap::new();
        let programs_map = ProgramsMap(programs);
        let clock = &ctx.accounts.clock.clone();
        let payer = SeahorseSigner {
            account: &ctx.accounts.payer,
            programs: &programs_map,
        };

        let owner = SeahorseSigner {
            account: &ctx.accounts.owner,
            programs: &programs_map,
        };

        let items = dot::program::Items::load(&mut ctx.accounts.items, &programs_map);

        update_items_handler(
            clock.clone(),
            payer.clone(),
            owner.clone(),
            items.clone(),
            coordinates_class,
        );

        dot::program::Items::store(items);

        return Ok(());
    }
}
