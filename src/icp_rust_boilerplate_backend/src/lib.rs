#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};
use ic_cdk::api::time;

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct LotteryTicket {
    id: u64,
    owner: String,
    numbers: Vec<u32>,
    created_at: u64,
    updated_at: Option<u64>,
}

// Implement Storable and BoundedStorable traits for LotteryTicket
impl Storable for LotteryTicket {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for LotteryTicket {
    const MAX_SIZE: u32 = 1024;  // Set an appropriate max size for your struct
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct LotteryDraw {
    id: u64,
    winning_numbers: Vec<u32>,
    draw_time: u64,
    participants: Vec<String>,
}

// Implement Storable and BoundedStorable traits for LotteryDraw
impl Storable for LotteryDraw {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for LotteryDraw {
    const MAX_SIZE: u32 = 1024;  // Set an appropriate max size for your struct
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum LotteryError {
    NotFound { msg: String },
    InvalidNumbers { msg: String },
}

thread_local! {
    static LOTTERY_MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static LOTTERY_TICKET_ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(LOTTERY_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static LOTTERY_DRAW_ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(LOTTERY_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))), 0)
            .expect("Cannot create a counter")
    );

    static LOTTERY_TICKET_STORAGE: RefCell<StableBTreeMap<u64, LotteryTicket, Memory>> = RefCell::new(
        StableBTreeMap::init(LOTTERY_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))))
    );

    static LOTTERY_DRAW_STORAGE: RefCell<StableBTreeMap<u64, LotteryDraw, Memory>> = RefCell::new(
        StableBTreeMap::init(LOTTERY_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))))
    );
}

// Function to buy a lottery ticket
#[ic_cdk::update]
fn buy_lottery_ticket(owner: String, numbers: Vec<u32>) -> Result<LotteryTicket, LotteryError> {
    // Validate the numbers
    if numbers.len() != 6 || numbers.iter().any(|&num| num > 49 || num == 0) {
        return Err(LotteryError::InvalidNumbers { msg: "Invalid lottery numbers".to_string() });
    }

    let id = LOTTERY_TICKET_ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");

    let ticket = LotteryTicket {
        id,
        owner: owner.clone(),
        numbers,
        created_at: time(),
        updated_at: None,
    };

    LOTTERY_TICKET_STORAGE.with(|m| m.borrow_mut().insert(id, ticket.clone()));
    Ok(ticket)
}

// Function to check lottery ticket by ID
#[ic_cdk::query]
fn check_lottery_ticket(id: u64) -> Result<LotteryTicket, LotteryError> {
    LOTTERY_TICKET_STORAGE.with(|service| {
        service
            .borrow_mut()
            .get(&id)
            .ok_or(LotteryError::NotFound {
                msg: format!("Lottery ticket with id={} not found", id),
            })
    })
}

// Function to conduct a lottery draw
#[ic_cdk::update]
fn conduct_lottery_draw(winning_numbers: Vec<u32>) -> Result<LotteryDraw, LotteryError> {
    // Validate the winning numbers
    if winning_numbers.len() != 6 || winning_numbers.iter().any(|&num| num > 49 || num == 0) {
        return Err(LotteryError::InvalidNumbers { msg: "Invalid winning numbers".to_string() });
    }

    let id = LOTTERY_DRAW_ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");

    let draw = LotteryDraw {
        id,
        winning_numbers,
        draw_time: time(),
        participants: Vec::new(),
    };

    LOTTERY_DRAW_STORAGE.with(|m| m.borrow_mut().insert(id, draw.clone()));
    Ok(draw)
}

// Function to participate in a lottery draw
#[ic_cdk::update]
fn participate_in_lottery_draw(ticket_id: u64, draw_id: u64) -> Result<LotteryDraw, LotteryError> {
    let ticket = LOTTERY_TICKET_STORAGE.with(|service| {
        service
            .borrow_mut()
            .get(&ticket_id)
            .ok_or(LotteryError::NotFound {
                msg: format!("Lottery ticket with id={} not found", ticket_id),
            })
    })?;

    let mut draw = LOTTERY_DRAW_STORAGE.with(|service| {
        service
            .borrow_mut()
            .get(&draw_id)
            .ok_or(LotteryError::NotFound {
                msg: format!("Lottery draw with id={} not found", draw_id),
            })
    })?;

    // Add the participant to the draw
    draw.participants.push(ticket.owner.clone());

    LOTTERY_DRAW_STORAGE.with(|m| m.borrow_mut().insert(draw_id, draw.clone()));
    Ok(draw)
}

// Function to get all lottery tickets
#[ic_cdk::query]
fn get_all_lottery_tickets() -> Result<Vec<LotteryTicket>, LotteryError> {
    let tickets = LOTTERY_TICKET_STORAGE.with(|m| m.borrow().iter().map(|(_, v)| v.clone()).collect::<Vec<_>>());
    if tickets.len() == 0 {
        return Err(LotteryError::NotFound { msg: "No lottery tickets found".to_string() });
    }
    Ok(tickets)
}

// Function to get all lottery draws
#[ic_cdk::query]
fn get_all_lottery_draws() -> Result<Vec<LotteryDraw>, LotteryError> {
    let draws = LOTTERY_DRAW_STORAGE.with(|m| m.borrow().iter().map(|(_, v)| v.clone()).collect::<Vec<_>>());
    if draws.len() == 0 {
        return Err(LotteryError::NotFound { msg: "No lottery draws found".to_string() });
    }
    Ok(draws)
}

// Export the candid interface
ic_cdk::export_candid!();
