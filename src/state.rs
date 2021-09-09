use lazy_static::lazy_static;
use rand::{seq::SliceRandom, thread_rng};
use std::{cmp::min, collections::HashMap, convert::TryFrom};

#[derive(PartialEq, Eq, Hash)]
pub struct State {
    cards: [u8; 3],
    turn: Turn,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Turn {
    Player,
    Opponent,
}

impl Turn {
    pub fn next(self) -> Self {
        match self {
            Turn::Player => Turn::Opponent,
            Turn::Opponent => Turn::Player,
        }
    }
}

#[derive(Clone)]
pub struct VerboseState {
    pub player_hand: CardsHand,
    pub opponent_hand: CardsHand,
    pub table_stack: CardsHand,
    pub turn: Turn,
}

const CARDS_CNT_WHEN_TAKING_FROM_STACK: usize = 3;

impl VerboseState {
    pub fn following_states(&self) -> Vec<Self> {
        let mut res = vec![];
        let cur_hand = self.get_current_hand();

        for i in 0..CardsHand::CARD_TYPES {
            // Put 1 card
            if cur_hand.cards[i] > 0 {
                let mut s = self.clone();
                s.get_current_hand_mut().cards[i] -= 1;
                s.table_stack.cards[i] += 1;
                s.turn = s.turn.next();
                res.push(s);
            }

            // Put 4 cards (or 3 nines)
            let all_cards_cnt = u8::try_from(CardsHand::card_idx_to_cnt(i)).unwrap();
            if cur_hand.cards[i] == all_cards_cnt {
                let mut s = self.clone();
                s.get_current_hand_mut().cards[i] -= all_cards_cnt;
                s.table_stack.cards[i] += all_cards_cnt;
                s.turn = s.turn.next();
                res.push(s);
            }

            if self.table_stack.cards[i] != 0 {
                // Can't put weaker cards
                break;
            }
        }

        {
            // Take up to 3 top cards
            let mut s = self.clone();
            let cards_taken_cnt = u8::try_from(CARDS_CNT_WHEN_TAKING_FROM_STACK).unwrap();
            let mut to_remove_yet = cards_taken_cnt;
            for i in 0..CardsHand::CARD_TYPES {
                let rm_cur = min(to_remove_yet, s.table_stack.cards[i]);
                to_remove_yet -= rm_cur;
                s.table_stack.cards[i] -= rm_cur;
                s.get_current_hand_mut().cards[i] += rm_cur;
                if to_remove_yet == 0 {
                    break;
                }
            }
            s.turn = s.turn.next();
            if to_remove_yet < cards_taken_cnt {
                res.push(s);
            }
        }

        res
    }

    pub fn preceding_states(&self) -> Vec<Self> {
        let mut res = vec![];

        // Clone to easier access "previous hand"
        let selff = VerboseState {
            turn: self.turn.next(),
            ..self.clone()
        };

        let mut top_card = None;
        for i in 0..CardsHand::CARD_TYPES {
            if selff.table_stack.cards[i] > 0 {
                top_card = Some(i);
                break;
            }
        }

        if let Some(i) = top_card {
            // Put 1 card
            let mut s = selff.clone();
            s.get_current_hand_mut().cards[i] += 1;
            s.table_stack.cards[i] -= 1;
            res.push(s);

            // Put 4 cards (or 3 nines)
            let all_cards_cnt = u8::try_from(CardsHand::card_idx_to_cnt(i)).unwrap();
            if selff.table_stack.cards[i] == all_cards_cnt {
                let mut s = selff.clone();
                s.get_current_hand_mut().cards[i] += all_cards_cnt;
                s.table_stack.cards[i] -= all_cards_cnt;
                res.push(s);
            }
        }

        // Take up to 3 top cards
        {
            fn process_one_card(
                res: &mut Vec<VerboseState>,
                vs: &mut VerboseState,
                top_card: usize,
                all_cards_req: bool,
                mut cards_left: usize,
            ) {
                cards_left -= 1;
                for i in 0..=top_card {
                    if vs.get_current_hand().cards[i] == 0 {
                        continue;
                    }
                    vs.get_current_hand_mut().cards[i] -= 1;
                    vs.table_stack.cards[i] += 1;

                    if cards_left == 0 || !all_cards_req {
                        res.push(vs.clone());
                    }

                    if cards_left > 0 {
                        process_one_card(res, vs, i, all_cards_req, cards_left);
                    }

                    vs.table_stack.cards[i] -= 1;
                    vs.get_current_hand_mut().cards[i] += 1;
                }
            }

            let (top_card, all_cards_req) = match top_card {
                Some(c) => (c, true),
                None => (CardsHand::CARD_TYPES - 1, false),
            };
            process_one_card(
                &mut res,
                &mut selff.clone(),
                top_card,
                all_cards_req,
                CARDS_CNT_WHEN_TAKING_FROM_STACK,
            );
        }

        res
    }

    fn get_current_hand(&self) -> &CardsHand {
        match self.turn {
            Turn::Player => &self.player_hand,
            Turn::Opponent => &self.opponent_hand,
        }
    }

    fn get_current_hand_mut(&mut self) -> &mut CardsHand {
        match self.turn {
            Turn::Player => &mut self.player_hand,
            Turn::Opponent => &mut self.opponent_hand,
        }
    }
}

#[derive(Clone)]
pub struct CardsHand {
    /// Sorted as: [Aces, Kings, Queens, Jacks, Tens, Nines].
    /// There are up to 3 nines (assuming one is always on the table).
    pub cards: [u8; Self::CARD_TYPES],
}

impl CardsHand {
    const CARD_TYPES: usize = 6;
}

// ================ CONVERSIONS ======================

lazy_static! {
    static ref CARD_DISTR_4: Vec<(u8, u8, u8)> = gen_card_distr(4);
    static ref CARD_DISTR_3: Vec<(u8, u8, u8)> = gen_card_distr(3);
    static ref REV_CARD_DISTR_4: HashMap<(u8, u8, u8), u8> = CARD_DISTR_4
        .iter()
        .enumerate()
        .map(|(a, b)| (*b, u8::try_from(a).unwrap()))
        .collect();
    static ref REV_CARD_DISTR_3: HashMap<(u8, u8, u8), u8> = CARD_DISTR_3
        .iter()
        .enumerate()
        .map(|(a, b)| (*b, u8::try_from(a).unwrap()))
        .collect();
}

fn gen_card_distr(sum: u8) -> Vec<(u8, u8, u8)> {
    let mut res = vec![];
    for i in 0..=sum {
        for j in 0..=sum {
            for k in 0..=sum {
                if i + j + k == sum {
                    res.push((i, j, k));
                }
            }
        }
    }
    res
}

impl CardsHand {
    fn card_idx_to_distr(idx: usize) -> &'static Vec<(u8, u8, u8)> {
        if idx < Self::CARD_TYPES - 1 {
            &*CARD_DISTR_4
        } else if idx == Self::CARD_TYPES - 1 {
            &*CARD_DISTR_3
        } else {
            panic!("Invalid idx")
        }
    }

    fn card_idx_to_rev_distr(idx: usize) -> &'static HashMap<(u8, u8, u8), u8> {
        if idx < Self::CARD_TYPES - 1 {
            &*REV_CARD_DISTR_4
        } else if idx == Self::CARD_TYPES - 1 {
            &*REV_CARD_DISTR_3
        } else {
            panic!("Invalid idx")
        }
    }

    fn card_idx_to_cnt(idx: usize) -> usize {
        if idx < Self::CARD_TYPES - 1 {
            4
        } else if idx == Self::CARD_TYPES - 1 {
            3
        } else {
            panic!("Invalid idx")
        }
    }
}

impl From<State> for VerboseState {
    fn from(s: State) -> Self {
        let mut vs = VerboseState {
            player_hand: CardsHand::empty(),
            opponent_hand: CardsHand::empty(),
            table_stack: CardsHand::empty(),
            turn: s.turn,
        };

        for i in 0..CardsHand::CARD_TYPES {
            let card_code = (usize::from(s.cards[i / 2]) >> (i % 2 * 4)) & 0xF;
            let distr = CardsHand::card_idx_to_distr(i);
            assert!(card_code < distr.len(), "Invalid state");
            let card_distr = distr[card_code];
            vs.player_hand.cards[i] = card_distr.0;
            vs.opponent_hand.cards[i] = card_distr.1;
            vs.table_stack.cards[i] = card_distr.2;
        }

        vs
    }
}

impl TryFrom<VerboseState> for State {
    type Error = &'static str;

    fn try_from(mut vs: VerboseState) -> Result<Self, Self::Error> {
        let mut s = State {
            cards: [0; 3],
            turn: vs.turn,
        };

        for i in 0..CardsHand::CARD_TYPES {
            let key = (
                vs.player_hand.cards[i],
                vs.opponent_hand.cards[i],
                vs.table_stack.cards[i],
            );
            let distr = CardsHand::card_idx_to_rev_distr(i);
            if !distr.contains_key(&key) {
                return Err("Invalid state");
            }
            let card_code = distr[&key];
            s.cards[i / 2] |= card_code << (i % 2 * 4);
        }

        Ok(s)
    }
}

// ===================== CONSTRUCTORS ===================
impl CardsHand {
    pub fn empty() -> Self {
        Self { cards: [0; 6] }
    }
}

impl VerboseState {
    pub fn initial() -> Self {
        Self {
            player_hand: CardsHand {
                cards: [2, 2, 2, 2, 2, 2],
            },
            opponent_hand: CardsHand {
                cards: [2, 2, 2, 2, 2, 1],
            },
            table_stack: CardsHand {
                // cards: [0, 0, 0, 0, 0, 1],
                cards: [0, 0, 0, 0, 0, 0],
            },
            turn: Turn::Player,
        }
    }

    pub fn random() -> Self {
        let mut rng = thread_rng();
        let mut vs = Self {
            player_hand: CardsHand::empty(),
            opponent_hand: CardsHand::empty(),
            table_stack: CardsHand::empty(),
            turn: Turn::Player,
        };

        for i in 0..CardsHand::CARD_TYPES {
            let distr = CardsHand::card_idx_to_distr(i);
            let card_distr = distr.as_slice().choose(&mut rng).unwrap();
            vs.player_hand.cards[i] = card_distr.0;
            vs.opponent_hand.cards[i] = card_distr.1;
            vs.table_stack.cards[i] = card_distr.2;
        }

        vs
    }
}
