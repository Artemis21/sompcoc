use num_bigint::BigUint;
use num_format::{Locale, ToFormattedString};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub struct ItemId(pub u32);

#[derive(Default)]
pub struct Item {
    pub id: ItemId,
    pub name: &'static str,
    pub description: &'static str,
    pub base_price: BigUint,
    pub income_per_tick: Option<BigUint>,
    pub shill_multiplier: Option<BigUint>,
    pub only_one: bool,
}

impl Item {
    pub fn show_count(&self, count: u32) -> String {
        if self.only_one {
            self.name.to_string()
        } else {
            format!("{}× {}", count.to_formatted_string(&Locale::en), self.name)
        }
    }

    pub fn price_of_nth(&self, n: u32) -> BigUint {
        if self.only_one {
            self.base_price.clone()
        } else {
            let numer = BigUint::from(23u32).pow(n);
            let denom = BigUint::from(20u32).pow(n);
            &self.base_price * numer / denom
        }
    }
}

pub const SHOP_ITEMS: &'static [ItemId] = &[
    ItemId(0),
    ItemId(1),
    ItemId(11),
    ItemId(2),
    ItemId(3),
    ItemId(4),
    ItemId(5),
    ItemId(12),
    ItemId(6),
    ItemId(7),
    ItemId(8),
    ItemId(13),
    ItemId(9),
    ItemId(10),
];

pub fn get_item(id: ItemId) -> Item {
    match id.0 {
        0 => Item {
            id: ItemId(0),
            name: "Poster",
            description: "Put a flimsy poster up in the UGSA. Hope that no-one repurposes it for their problem sheet scribbles.",
            base_price: 8u32.into(),
            income_per_tick: Some(1u32.into()),
            ..Default::default()
        },
        1 => Item {
            id: ItemId(1),
            name: "Fresher",
            description: "Pay off an enthusiastic fresher to invite their friends at every opportunity. One or two might even show up.",
            base_price: 180u32.into(),
            income_per_tick: Some(9u32.into()),
            ..Default::default()
        },
        2 => Item {
            id: ItemId(2),
            name: "JoPonting",
            description: "Occasional mentions in the footer of Jo Ponting's emails. You are breaking in to the department's consciousness.",
            base_price: 920u32.into(),
            income_per_tick: Some(35u32.into()),
            ..Default::default()
        },
        11 => Item {
            id: ItemId(11),
            name: "Glowup",
            description: "People pay more attention to attractive people. Get more attractive so you can shill better.",
            base_price: 3000u32.into(),
            shill_multiplier: Some(500u32.into()),
            only_one: true,
            ..Default::default()
        },
        3 => Item {
            id: ItemId(3),
            name: "Careers Talk",
            description: "Convince an unsuspecting corporation that SompCoc can get them the exposure they need. Convince some equally unsuspecting CSers that this will be useful.",
            base_price: 6600u32.into(),
            income_per_tick: Some(105u32.into()),
            ..Default::default()
        },
        4 => Item {
            id: ItemId(4),
            name: "PizzaSoc",
            description: "You don't need to be interesting if you're serving free dinner. Be prepared for pizza-supplier flame wars.",
            base_price: 50_000u32.into(),
            income_per_tick: Some(300u32.into()),
            ..Default::default()
        },
        5 => Item {
            id: ItemId(5),
            name: "Whiteboard Art",
            description: "More compelling than a poster could ever be. Requires redrawing frequently.",
            base_price: 120_000u32.into(),
            income_per_tick: Some(1140u32.into()),
            ..Default::default()
        },
        12 => Item {
            id: ItemId(12),
            name: "Departmental Endorsement",
            description: "Trust me, the credibility boost is worth it. Now your shilling will look a little less silly.",
            base_price: 333_333u32.into(),
            shill_multiplier: Some(50u32.into()),
            only_one: true,
            ..Default::default()
        },
        6 => Item {
            id: ItemId(6),
            name: "Boardgame Night",
            description: "Ensnare innocents who showed up for Uno in a board game with Turing-complete mechanics and uncertain termination. The perfect trap.",
            base_price: 530_000u32.into(),
            income_per_tick: Some(6183u32.into()),
            ..Default::default()
        },
        7 => Item {
            id: ItemId(7),
            name: "Stroopwafel",
            description: "The pinnacle of UGSA snacking with a price point to match. It is rumoured that non-empty packets exist, but a sighting has never been confirmed.",
            base_price: 2_110_000u32.into(),
            income_per_tick: Some(37_000u32.into()),
            ..Default::default()
        },
        8 => Item {
            id: ItemId(8),
            name: "STEM Social",
            description: "Piggyback off a more respectable science for exposure. Try not to put them off too much.",
            base_price: 34_000_000u32.into(),
            income_per_tick: Some(81_467u32.into()),
            ..Default::default()
        },
        13 => Item {
            id: ItemId(13),
            name: "Switch pizzeria",
            description: "To the better one.",
            base_price: 123_000_000u32.into(),
            shill_multiplier: Some(321u32.into()),
            only_one: true,
            ..Default::default()
        },
        9 => Item {
            id: ItemId(9),
            name: "Ice Skating",
            description: "It's the superior sport, who wouldn't show up? And one you've got them on the ice, they're helpless...",
            base_price: 240_000_000u32.into(),
            income_per_tick: Some(1_000_000u32.into()),
            ..Default::default()
        },
        10 => Item {
            id: ItemId(10),
            name: "One-way Tescalator",
            description: "What's at the bottom? Nobody knows. But those members aren't leaving any time soon.",
            base_price: 8_000_000_000u64.into(),
            income_per_tick: Some(10_000_000u32.into()),
            ..Default::default()
        },
        _ => panic!("unknown item {id:?}"),
    }
}
