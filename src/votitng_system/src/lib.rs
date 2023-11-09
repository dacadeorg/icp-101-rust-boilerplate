#[macro_use]
extern crate serde;

use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell, collections::HashMap};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[warn(unused_must_use)]
type Result<T> = std::result::Result<T, Error>;

#[derive(candid::CandidType, Deserialize, Serialize, Debug)]
enum Error {
    InsertFailed,
    VoteNotFound,
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static VOTES: RefCell<StableBTreeMap<u64, Vote, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        ));
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Vote {
    id: u64,
    candidate: String,
    voter: String,
    timestamp: u64,
}

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

#[ic_cdk::update]
fn add_vote(candidate: String, voter: String) -> Result<Vote> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment id counter");

    let vote = Vote {
        id,
        candidate: candidate.clone(),
        voter: voter.clone(),
        timestamp: time(),
    };

    if get_vote_by_candidate_voter(&candidate, &voter).is_none() {
        insert(&vote);
        Ok(vote)
    } else {
        Err(Error::InsertFailed)
    }
}

#[ic_cdk::update]
fn update_vote(id: u64, candidate: String, voter: String) -> Result<Vote> {
    let mut vote = VOTES.with(|votes| votes.borrow_mut().get(&id)).ok_or(Error::VoteNotFound)?;

    let existing_vote = get_vote_by_candidate_voter(&vote.candidate, &vote.voter);

    if let Some(existing_vote) = existing_vote {
        if existing_vote.id == id {
            vote.candidate = candidate.clone();
            vote.voter = voter.clone();
            vote.timestamp = time();
            Ok(vote)
        } else {
            Err(Error::InsertFailed)
        }
    } else {
        Err(Error::VoteNotFound)
    }
}

#[ic_cdk::update]
fn delete_vote(id: u64) -> Result<Vote> {
    let vote = VOTES.with(|votes| votes.borrow_mut().remove(&id)).ok_or(Error::VoteNotFound)?;
    Ok(vote)
}

#[ic_cdk::update]
fn clear_votes() -> Result<()> {
    VOTES.with(|votes| {
        votes.borrow_mut().clear();
        Ok(())
    })
}

#[ic_cdk::query]
fn get_votes() -> Result<Vec<Vote>> {
    VOTES.with(|votes| {
        Ok(votes.borrow().iter().map(|(_, v)| v.clone()).collect())
    })
}

#[ic_cdk::query]
fn total_votes() -> Result<u64> {
    VOTES.with(|votes| Ok(votes.borrow().len() as u64))
}

#[ic_cdk::query]
fn get_votes_by_candidate(candidate: String) -> Result<Vec<Vote>> {
    let votes = VOTES.with(|votes| {
        let mut votes = votes.borrow().iter().filter(|(_, v)| v.candidate == candidate).map(|(_, v)| v.clone()).collect::<Vec<Vote>>();
        votes.sort_by_key(|v| v.timestamp);
        Ok(votes)
    })?;
    Ok(votes)
}

#[ic_cdk::query]
fn get_votes_by_voter(voter: String) -> Result<Vec<Vote>> {
    let votes = VOTES.with(|votes| {
        let mut votes = votes.borrow().iter().filter(|(_, v)| v.voter == voter).map(|(_, v)| v.clone()).collect::<Vec<Vote>>();
        votes.sort_by_key(|v| v.timestamp);
        Ok(votes)
    })?;
    Ok(votes)
}

#[ic_cdk::query]
fn get_latest_vote_timestamp() -> Result<u64> {
    let latest_timestamp = VOTES.with(|votes| {
        let max_timestamp = votes.borrow().iter().map(|(_, v)| v.timestamp).max().unwrap_or(0);
        Ok(max_timestamp)
    })?;
    Ok(latest_timestamp)
}

#[ic_cdk::query]
fn get_candidates() -> Result<Vec<String>> {
    let candidates = VOTES.with(|votes| {
        let mut candidates: HashMap<String, bool> = HashMap::new();
        for (_, vote) in votes.borrow().iter() {
            candidates.insert(vote.candidate.clone(), true);
        }
        Ok(candidates.keys().cloned().collect())
    })?;
    Ok(candidates)
}

#[ic_cdk::query]
fn get_all_candidate_votes() -> Result<HashMap<String, u64>> {
    let candidate_votes = VOTES.with(|votes| {
        let mut candidate_votes: HashMap<String, u64> = HashMap::new();
        for (_, vote) in votes.borrow().iter() {
            let count = candidate_votes.entry(vote.candidate.clone()).or_insert(0);
            *count += 1;
        }
        Ok(candidate_votes)
    })?;
    Ok(candidate_votes)
}

#[ic_cdk::query]
fn get_votes_in_time_range(start_time: u64, end_time: u64) -> Result<Vec<Vote>> {
    if start_time > end_time {
        return Ok(vec![]);
    }

    let votes_in_range = VOTES.with(|votes| {
        Ok(votes.borrow().iter().filter(|(_, v)| v.timestamp >= start_time && v.timestamp <= end_time)
            .map(|(_, v)| v.clone())
            .collect())
    })?;
    Ok(votes_in_range)
}

#[ic_cdk::query]
fn get_most_voted_candidate() -> Result<String> {
    let most_voted_candidate = VOTES.with(|votes| {
        let mut candidate_votes: HashMap<String, u64> = HashMap::new();
        for (_, vote) in votes.borrow().iter() {
            let count = candidate_votes.entry(vote.candidate.clone()).or_insert(0);
            *count += 1;
        }
        candidate_votes.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(candidate, _)| candidate)
    })?;
    Ok(most_voted_candidate.ok_or(Error::InsertFailed)?)
}

#[ic_cdk::query]
fn get_least_voted_candidate() -> Result<String> {
    let least_voted_candidate = VOTES.with(|votes| {
        let mut candidate_votes: HashMap<String, u64> = HashMap::new();
        for (_, vote) in votes.borrow().iter() {
            let count = candidate_votes.entry(vote.candidate.clone()).or_insert(0);
            *count += 1;
        }
        candidate_votes.into_iter()
            .min_by_key(|(_, count)| *count)
            .map(|(candidate, _)| candidate)
    })?;
    Ok(least_voted_candidate.ok_or(Error::InsertFailed)?)
}

#[ic_cdk::query]
fn get_votes_sorted_by_timestamp() -> Result<Vec<Vote>> {
    let mut votes_sorted = VOTES.with(|votes| {
        let mut votes_sorted = votes.borrow().iter().map(|(_, v)| v.clone()).collect::<Vec<Vote>>();
        votes_sorted.sort_by_key(|v| v.timestamp);
        Ok(votes_sorted)
    })?;
    Ok(votes_sorted)
}

fn insert(vote: &Vote) {
    VOTES.with(|votes| votes.borrow_mut().insert(vote.id, vote.clone()));
}

fn get_vote_by_candidate_voter(candidate: &str, voter: &str) -> Option<Vote> {
    VOTES.with(|votes| votes.borrow().iter().find(|(_, v)| v.candidate == *candidate && v.voter == *voter).map(|(_, v)| v.clone()))
}

ic_cdk::export_candid!();
