#[macro_use]
// region: --- IMPORTS
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell, collections::HashMap};
// endregion --- IMPORTS

// region: --- TYPES
type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[warn(unused_must_use)]
type Result<T> = std::result::Result<T, Error>;

#[derive(candid::CandidType, Deserialize, Serialize, Debug)]
enum Error {
    InsertFailed,
    VoteNotFoundError,
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static VOTES: RefCell<StableBTreeMap<u64, Vote, Memory>> = RefCell::new(StableBTreeMap::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));
}
// endregion --- TYPES

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Vote {
    id: u64,
    candidate: String,
    voter: String,
    timestamp: u64,
}

// region: --- IMPL
impl Storable for Vote {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Vote {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}
// endregion --- IMPL

// region: --- METHODS
// Function to add new vote
#[ic_cdk::update]
fn add_vote(candidate: String, voter: String) -> Result<Vote> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let vote = Vote {
        id,
        candidate,
        voter,
        timestamp: time(),
    };
    insert(&vote);
    Ok(vote)
}

// Function to update a vote by id - update candidate, voter
#[ic_cdk::update]
fn update_vote(id: u64, candidate: String, voter: String) -> Result<Vote> {
    let mut vote = VOTES
        .with(|votes| votes.borrow().get(&id))
        .ok_or(Error::VoteNotFoundError)?;
    vote.candidate = candidate.clone();
    vote.voter = voter.clone();
    vote.timestamp = time();

    insert(&vote);
    Ok(vote)
}

// Function to delete a vote by id
#[ic_cdk::update]
fn delete_vote(id: u64) -> Result<Vote> {
    let vote = VOTES
        .with(|votes| {
            let mut votes_mut = votes.borrow_mut();
            votes_mut.remove(&id)
        })
        .ok_or(Error::VoteNotFoundError)?;

    Ok(vote)
}

// Function to clear all votes
#[ic_cdk::update]
fn clear_votes() -> Result<()> {
    VOTES.with(|votes| {
        let mut votes_mut = votes.borrow_mut();
        let keys: Vec<u64> = votes_mut.iter().map(|(_, v)| v.id).collect();
        for key in keys {
            votes_mut.remove(&key);
        }
        Ok(())
    })
}

// Function to get all votes
#[ic_cdk::query]
fn get_votes() -> Result<Vec<Vote>> {
    VOTES.with(|votes| Ok(votes.borrow().iter().map(|(_, v)| v.clone()).collect()))
}

// Function to get the total number of votes
#[ic_cdk::query]
fn total_votes() -> Result<u64> {
    VOTES.with(|votes| Ok(votes.borrow().len() as u64))
}

// Function to get all votes by a specific candidate
#[ic_cdk::query]
fn get_votes_by_candidate(candidate: String) -> Result<Vec<Vote>> {
    VOTES.with(|votes| {
        Ok(votes
            .borrow()
            .iter()
            .filter(|(_, v)| v.candidate == candidate)
            .map(|(_, v)| v.clone())
            .collect())
    })
}

// Function to get all votes by a specific voter
#[ic_cdk::query]
fn get_votes_by_voter(voter: String) -> Result<Vec<Vote>> {
    VOTES.with(|votes| {
        Ok(votes
            .borrow()
            .iter()
            .filter(|(_, v)| v.voter == voter)
            .map(|(_, v)| v.clone())
            .collect())
    })
}

// Function to get the timestamp of the latest vote
#[ic_cdk::query]
fn get_latest_vote_timestamp() -> Result<u64> {
    VOTES.with(|votes| {
        Ok(votes
            .borrow()
            .iter()
            .map(|(_, vote)| vote.timestamp)
            .max()
            .unwrap_or(0))
    })
}

// Function to get the list of candidates
#[ic_cdk::query]
fn get_candidates() -> Result<Vec<String>> {
    VOTES.with(|votes| {
        let mut candidates: HashMap<String, bool> = HashMap::new();
        for (_, vote) in votes.borrow().iter() {
            candidates.insert(vote.candidate.clone(), true);
        }
        Ok(candidates.keys().cloned().collect())
    })
}

// Function to get the number of votes for all candidates
#[ic_cdk::query]
fn get_all_candidate_votes() -> Result<HashMap<String, u64>> {
    VOTES.with(|votes| {
        let mut candidate_votes: HashMap<String, u64> = HashMap::new();
        for (_, vote) in votes.borrow().iter() {
            let count = candidate_votes.entry(vote.candidate.clone()).or_insert(0);
            *count += 1;
        }
        Ok(candidate_votes)
    })
}

// Function to get all votes within a specific time range
#[ic_cdk::query]
fn get_votes_in_time_range(start_time: u64, end_time: u64) -> Result<Vec<Vote>> {
    VOTES.with(|votes| {
        Ok(votes
            .borrow()
            .iter()
            .filter(|(_, v)| v.timestamp >= start_time && v.timestamp <= end_time)
            .map(|(_, v)| v.clone())
            .collect())
    })
}

// Function to get the most voted candidate
#[ic_cdk::query]
fn get_most_voted_candidate() -> Result<String> {
    VOTES.with(|votes| {
        let mut candidate_votes: HashMap<String, u64> = HashMap::new();
        for (_, vote) in votes.borrow().iter() {
            let count = candidate_votes.entry(vote.candidate.clone()).or_insert(0);
            *count += 1;
        }
        candidate_votes
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(candidate, _)| candidate)
            .ok_or(Error::InsertFailed)
    })
}

// Function to get the least voted candidate
#[ic_cdk::query]
fn get_least_voted_candidate() -> Result<String> {
    VOTES.with(|votes| {
        let mut candidate_votes: HashMap<String, u64> = HashMap::new();
        for (_, vote) in votes.borrow().iter() {
            let count = candidate_votes.entry(vote.candidate.clone()).or_insert(0);
            *count += 1;
        }
        candidate_votes
            .into_iter()
            .min_by_key(|(_, count)| *count)
            .map(|(candidate, _)| candidate)
            .ok_or(Error::InsertFailed)
    })
}

// Function to get the votes sorted by timestamp (asc)
#[ic_cdk::query]
fn get_votes_sorted_by_timestamp() -> Result<Vec<Vote>> {
    VOTES.with(|votes| {
        let mut votes_sorted = votes.borrow().iter().map(|(_, v)| v.clone() ).collect::<Vec<Vote>>();
        votes_sorted.sort_by_key(|v| v.timestamp);
        Ok(votes_sorted)
    })
}
// endregion --- METHODS

// region: --- HELPER FN
fn insert(vote: &Vote) {
    VOTES.with(|votes| votes.borrow_mut().insert(vote.id, vote.clone()));
}
// endregion --- HELPER FN

ic_cdk::export_candid!();